//! reads: does the agent read the palace at all? Counts, from local Claude
//! Code transcripts (the Claude Code JSONL format only), how many user turns
//! were followed by at least one read of a file inside the palace before the
//! next user turn. Read-only, no network.
//!
//! Why: writes are easy to see; reads are not. A memory that is written every
//! session and read on almost no turn is a diary, not a memory. This counter
//! makes the read rate visible so the palace's own instructions can be judged
//! by what the agent does, not by what they say.
//!
//! Privacy: counts only. No transcript text, path or command is printed,
//! returned or stored. Lines are streamed with the same bounds and symlink
//! rules as `tokens`.
//!
//! What counts:
//! - a **user turn** is a main-thread user entry carrying text (a string, or a
//!   text block with no tool result). Meta entries, compaction summaries,
//!   local-command echoes and subagent (sidechain) entries are not turns.
//! - a **palace read** is a Read / Grep / Glob / NotebookRead call whose path
//!   resolves inside the palace root. Relative paths resolve against the
//!   entry's `cwd`; `..` is folded before the check, so it cannot escape.
//! - a **palace write** is a Write / Edit / MultiEdit / NotebookEdit call
//!   whose path resolves inside the palace root.
//! - a **shell touch** is a Bash call whose command names the palace root.
//!   Counted apart: a command string cannot say whether it read or wrote, and
//!   a command that spells the palace as a relative path, with `~` or `$HOME`,
//!   or with backslash-escaped spaces is not seen. So
//!   two rates: the read rate (tool reads only, the floor) and the touch rate
//!   (reads or shell touches, a wider count). The truth usually sits between
//!   them; shell reads by relative path from inside the palace are not seen
//!   and can lift it above the touch rate.
//! - a user turn copied into a new transcript on resume or fork counts once:
//!   turns are deduped by their entry `uuid`, as tool calls are by tool id.
//! - subagent calls count as **delegated** reads and writes, outside the rate.

use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::tokens;

pub const DEFAULT_DAYS: u64 = 7;
pub const MAX_DAYS: u64 = 90;

#[derive(Serialize, Clone, Default, Debug, PartialEq)]
pub struct ReadRate {
    /// The palace root that was measured (the user's own setting, not transcript data).
    pub root: String,
    pub days: u64,
    pub user_turns: usize,
    /// User turns followed by at least one palace read before the next turn.
    pub turns_with_read: usize,
    /// turns_with_read / user_turns, in percent (None when there are no turns).
    pub read_rate_pct: Option<f64>,
    /// User turns followed by a palace read OR a shell touch: a wider count,
    /// since a shell command naming the palace may have read it or written it.
    /// Not an upper bound: shell commands that spell the palace as a relative
    /// path, with `~` or `$HOME`, or with escaped spaces are still missed.
    pub turns_with_touch: usize,
    pub touch_rate_pct: Option<f64>,
    pub palace_reads: usize,
    pub palace_writes: usize,
    pub shell_touches: usize,
    pub delegated_reads: usize,
    pub delegated_writes: usize,
    pub sessions: usize,
    /// Sessions with at least one main-thread palace read.
    pub sessions_with_read: usize,
    pub files_scanned: usize,
    pub note: String,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind {
    Turn,
    Read,
    Write,
    Shell,
}

/// The palace root in every spelling a transcript may use: as given, and
/// canonicalized (macOS `/var` vs `/private/var`, symlinked home folders).
pub fn roots_for(root: &Path) -> Vec<PathBuf> {
    // Absolute first: a relative root (`--palace palace`) would otherwise
    // match any command that merely contains the word.
    let abs = if root.is_absolute() {
        root.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|c| c.join(root))
            .unwrap_or_else(|_| root.to_path_buf())
    };
    let mut v = vec![normalize(&abs)];
    if let Ok(c) = std::fs::canonicalize(root) {
        let c = normalize(&c);
        if !v.contains(&c) {
            v.push(c);
        }
    }
    v
}

/// Lexical normalization: fold `.` and `..` without touching the filesystem
/// (the file may no longer exist). A `..` above the root stays at the root.
fn normalize(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn inside(path: &str, cwd: Option<&str>, roots: &[PathBuf]) -> bool {
    if path.is_empty() {
        return false;
    }
    let p = Path::new(path);
    let abs = if p.is_absolute() {
        p.to_path_buf()
    } else {
        match cwd {
            Some(c) => Path::new(c).join(p),
            None => return false,
        }
    };
    let n = normalize(&abs);
    roots.iter().any(|r| n.starts_with(r))
}

fn is_user_turn(v: &serde_json::Value) -> bool {
    if v.get("isMeta").and_then(|x| x.as_bool()).unwrap_or(false)
        || v.get("isCompactSummary")
            .and_then(|x| x.as_bool())
            .unwrap_or(false)
    {
        return false;
    }
    // A person's prompt, not harness traffic. Newer transcripts say so directly:
    // origin.kind is "human" for a typed prompt and something else (for example
    // "task-notification") for what the harness injects. Older entries carry no
    // origin, so fall back to the shapes the harness is known to inject.
    let origin_human = match v.pointer("/origin/kind").and_then(|x| x.as_str()) {
        Some("human") => true,
        Some(_) => return false,
        None => false,
    };
    let keep = |t: &str| {
        let t = t.trim_start();
        !t.is_empty() && !t.starts_with("<local-command-") && (origin_human || !harness_shaped(t))
    };
    match v.pointer("/message/content") {
        Some(serde_json::Value::String(s)) => keep(s),
        Some(serde_json::Value::Array(blocks)) => {
            let ty =
                |b: &serde_json::Value| b.get("type").and_then(|x| x.as_str()).map(str::to_owned);
            let has_result = blocks
                .iter()
                .any(|b| ty(b).as_deref() == Some("tool_result"));
            let first_text = blocks
                .iter()
                .find(|b| ty(b).as_deref() == Some("text"))
                .and_then(|b| b.get("text").and_then(|x| x.as_str()));
            !has_result && first_text.is_some_and(keep)
        }
        _ => false,
    }
}

/// Content shapes the harness injects as user entries (used only when an entry
/// carries no `origin`): notifications, reminders, slash and bash-mode echoes,
/// scheduled prompts, and the interrupt marker.
fn harness_shaped(t: &str) -> bool {
    const SHAPES: &[&str] = &[
        "<task-notification",
        "<system-reminder",
        "<command-name",
        "<command-message",
        "<command-args",
        "<bash-input",
        "<bash-stdout",
        "<bash-stderr",
        "<scheduled-task",
        "<ci-monitor-event",
        "<preview-annotation",
        "[Request interrupted",
    ];
    SHAPES.iter().any(|p| t.starts_with(p))
}

fn classify(
    name: &str,
    input: &serde_json::Value,
    cwd: Option<&str>,
    roots: &[PathBuf],
) -> Option<Kind> {
    let s = |k: &str| input.get(k).and_then(|x| x.as_str()).unwrap_or("");
    match name {
        "Read" => inside(s("file_path"), cwd, roots).then_some(Kind::Read),
        "Grep" | "Glob" => {
            // No path means the cwd: a search run from inside the palace reads it.
            let p = s("path");
            let hit = if p.is_empty() {
                cwd.is_some_and(|c| inside(c, None, roots))
            } else {
                inside(p, cwd, roots)
            };
            hit.then_some(Kind::Read)
        }
        "NotebookRead" => inside(s("notebook_path"), cwd, roots).then_some(Kind::Read),
        "Write" | "Edit" | "MultiEdit" => inside(s("file_path"), cwd, roots).then_some(Kind::Write),
        "NotebookEdit" => inside(s("notebook_path"), cwd, roots).then_some(Kind::Write),
        "Bash" => {
            let cmd = s("command");
            roots
                .iter()
                .any(|r| names_root(cmd, &r.to_string_lossy()))
                .then_some(Kind::Shell)
        }
        _ => None,
    }
}

/// Streaming tally: feed lines one at a time, then `finish`. Holds event
/// kinds, session ids and the ids of palace tool calls (none of which reach
/// the output), never the line text.
pub struct Tally<'r> {
    r: ReadRate,
    roots: &'r [PathBuf],
    horizon: u64,
    seen_tools: HashSet<String>,
    // turn entry uuids already counted (ids only, never text): a resumed or
    // forked session copies earlier entries into a new transcript file
    seen_turns: HashSet<String>,
    // session -> [(timestamp, order, kind)] on the main thread
    events: HashMap<String, Vec<(u64, usize, Kind)>>,
    order: usize,
}

impl<'r> Tally<'r> {
    pub fn new(roots: &'r [PathBuf], horizon: u64) -> Self {
        Tally {
            r: ReadRate::default(),
            roots,
            horizon,
            seen_tools: HashSet::new(),
            seen_turns: HashSet::new(),
            events: HashMap::new(),
            order: 0,
        }
    }

    /// `fi` only keeps sessions apart when a line has no `sessionId`.
    pub fn feed(&mut self, fi: usize, line: &str) {
        let roots = self.roots;
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            return;
        };
        let Some(t) = v
            .get("timestamp")
            .and_then(|x| x.as_str())
            .and_then(tokens::parse_rfc3339)
        else {
            return;
        };
        if t < self.horizon {
            return;
        }
        let side = v
            .get("isSidechain")
            .and_then(|x| x.as_bool())
            .unwrap_or(false);
        let session = v
            .get("sessionId")
            .and_then(|x| x.as_str())
            .map(str::to_owned)
            .unwrap_or_else(|| format!("file-{fi}"));
        let cwd = v.get("cwd").and_then(|x| x.as_str());
        self.order += 1;
        let order = self.order;
        match v.get("type").and_then(|x| x.as_str()) {
            Some("user") if !side && is_user_turn(&v) => {
                if let Some(id) = v.get("uuid").and_then(|x| x.as_str()) {
                    if !self.seen_turns.insert(id.to_owned()) {
                        return;
                    }
                }
                self.events
                    .entry(session)
                    .or_default()
                    .push((t, order, Kind::Turn));
            }
            Some("assistant") => {
                let Some(blocks) = v.pointer("/message/content").and_then(|x| x.as_array()) else {
                    return;
                };
                for b in blocks {
                    if b.get("type").and_then(|x| x.as_str()) != Some("tool_use") {
                        continue;
                    }
                    let name = b.get("name").and_then(|x| x.as_str()).unwrap_or("");
                    let input = b.get("input").unwrap_or(&serde_json::Value::Null);
                    let Some(k) = classify(name, input, cwd, roots) else {
                        continue;
                    };
                    // Dedupe only palace hits, so memory grows with palace events,
                    // not with every tool call in the window.
                    if let Some(id) = b.get("id").and_then(|x| x.as_str()) {
                        if !self.seen_tools.insert(id.to_owned()) {
                            continue;
                        }
                    }
                    let r = &mut self.r;
                    if side {
                        match k {
                            Kind::Read => r.delegated_reads += 1,
                            Kind::Write => r.delegated_writes += 1,
                            _ => {}
                        }
                        continue;
                    }
                    match k {
                        Kind::Read => r.palace_reads += 1,
                        Kind::Write => r.palace_writes += 1,
                        Kind::Shell => r.shell_touches += 1,
                        Kind::Turn => {}
                    }
                    self.events
                        .entry(session.clone())
                        .or_default()
                        .push((t, order, k));
                }
            }
            _ => {}
        }
    }

    pub fn finish(self) -> ReadRate {
        let Tally {
            mut r, mut events, ..
        } = self;
        for ev in events.values_mut() {
            ev.sort_by_key(|e| (e.0, e.1));
            let mut turns = 0usize;
            let mut open_read = false; // a turn awaiting its first read
            let mut open_touch = false; // a turn awaiting its first read or shell touch
            let mut any_read = false;
            for &(_, _, k) in ev.iter() {
                match k {
                    Kind::Turn => {
                        turns += 1;
                        open_read = true;
                        open_touch = true;
                    }
                    Kind::Read | Kind::Shell => {
                        if k == Kind::Read {
                            any_read = true;
                            if open_read {
                                r.turns_with_read += 1;
                                open_read = false;
                            }
                        }
                        if open_touch {
                            r.turns_with_touch += 1;
                            open_touch = false;
                        }
                    }
                    Kind::Write => {}
                }
            }
            if turns > 0 {
                r.sessions += 1;
                r.user_turns += turns;
                if any_read {
                    r.sessions_with_read += 1;
                }
            }
        }
        let pct = |n: usize| {
            (r.user_turns > 0).then(|| (n as f64 * 1000.0 / r.user_turns as f64).round() / 10.0)
        };
        r.read_rate_pct = pct(r.turns_with_read);
        r.touch_rate_pct = pct(r.turns_with_touch);
        r
    }
}

/// True when `cmd` names `root` as a whole path: the match must not sit inside
/// a longer path (`/mnt/home/u/palace`) or be a prefix of a sibling
/// (`/home/u/palace-old`). The character after it must end the path or
/// continue into it: `/`, whitespace, a quote, `;`, `)`, a backtick or the end.
fn names_root(cmd: &str, root: &str) -> bool {
    if root.is_empty() || root == "/" {
        return false;
    }
    let path_char = |c: char| c.is_alphanumeric() || matches!(c, '.' | '-' | '_' | '/' | '~');
    cmd.match_indices(root).any(|(i, _)| {
        let before = cmd[..i].chars().next_back();
        let after = cmd[i + root.len()..].chars().next();
        before.is_none_or(|c| !path_char(c))
            && after.is_none_or(|c| {
                matches!(c, '/' | '"' | '\'' | ';' | ')' | '`') || c.is_whitespace()
            })
    })
}

/// Pure tally over `(file_index, line)` pairs (the test entry point).
#[cfg(test)]
pub fn tally<'a>(
    lines: impl IntoIterator<Item = (usize, &'a str)>,
    roots: &[PathBuf],
    horizon: u64,
) -> ReadRate {
    let mut t = Tally::new(roots, horizon);
    for (fi, l) in lines {
        t.feed(fi, l);
    }
    t.finish()
}

/// Scan local Claude Code transcripts touched in the last `days` and tally them against
/// the palace at `root`.
pub fn read_rate(root: &Path, days: u64) -> ReadRate {
    let days = days.clamp(1, MAX_DAYS);
    let now = tokens::epoch_now();
    let horizon = now.saturating_sub(days * 86_400);
    let roots = roots_for(root);

    // Collect first, then feed in a fixed order: when a resumed or forked
    // transcript repeats ids, the first file seen wins, so the order must not
    // depend on how the filesystem lists a folder.
    let mut paths: Vec<PathBuf> = Vec::new();
    for dir in tokens::data_dirs() {
        let mut stack = vec![dir];
        while let Some(d) = stack.pop() {
            let Ok(rd) = std::fs::read_dir(&d) else {
                continue;
            };
            for e in rd.filter_map(|e| e.ok()) {
                let Ok(ft) = e.file_type() else { continue };
                if ft.is_symlink() {
                    continue; // never traverse a symlink (same rule as tokens)
                }
                let p = e.path();
                if ft.is_dir() {
                    stack.push(p);
                    continue;
                }
                if p.extension().and_then(|x| x.to_str()) != Some("jsonl") {
                    continue;
                }
                let fresh = e
                    .metadata()
                    .and_then(|m| m.modified())
                    .ok()
                    .and_then(|m| m.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() >= horizon)
                    .unwrap_or(false);
                if fresh {
                    paths.push(p);
                }
            }
        }
    }
    paths.sort();
    paths.dedup();

    let mut t = Tally::new(&roots, horizon);
    let mut files = 0usize;
    for p in &paths {
        let Ok(f) = std::fs::File::open(p) else {
            continue;
        };
        let fi = files;
        files += 1;
        tokens::for_each_bounded_line(f, |line| {
            // Cheap prefilter: only lines that can be a turn or a tool call.
            if line.contains("\"type\":\"user\"") || line.contains("\"tool_use\"") {
                t.feed(fi, line);
            }
        });
    }
    let mut r = t.finish();
    r.root = root.display().to_string();
    r.days = days;
    r.files_scanned = files;
    r.note = "counts only; no transcript text, transcript path or command is printed or kept (root is the palace you measured)".into();
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    const TS: &str = "2026-11-19T10:00:00Z";

    fn roots() -> Vec<PathBuf> {
        vec![PathBuf::from("/home/u/palace")]
    }
    fn turn(sess: &str, ts: &str) -> String {
        format!(
            r#"{{"type":"user","sessionId":"{sess}","timestamp":"{ts}","cwd":"/home/u/palace","message":{{"role":"user","content":"hello"}}}}"#
        )
    }
    fn tool(sess: &str, ts: &str, id: &str, name: &str, input: &str) -> String {
        format!(
            r#"{{"type":"assistant","sessionId":"{sess}","timestamp":"{ts}","cwd":"/home/u/palace","message":{{"role":"assistant","content":[{{"type":"tool_use","id":"{id}","name":"{name}","input":{input}}}]}}}}"#
        )
    }
    fn run(lines: &[String]) -> ReadRate {
        tally(lines.iter().map(|l| (0usize, l.as_str())), &roots(), 0)
    }

    #[test]
    fn read_inside_palace_counts_and_marks_the_turn() {
        let r = run(&[
            turn("s", TS),
            tool(
                "s",
                "2026-11-19T10:00:05Z",
                "t1",
                "Read",
                r#"{"file_path":"/home/u/palace/memory/a.md"}"#,
            ),
        ]);
        assert_eq!((r.user_turns, r.turns_with_read, r.palace_reads), (1, 1, 1));
        assert_eq!(r.read_rate_pct, Some(100.0));
        assert_eq!((r.sessions, r.sessions_with_read), (1, 1));
    }

    #[test]
    fn read_outside_palace_does_not_count() {
        let r = run(&[
            turn("s", TS),
            tool(
                "s",
                "2026-11-19T10:00:05Z",
                "t1",
                "Read",
                r#"{"file_path":"/home/u/other/a.md"}"#,
            ),
        ]);
        assert_eq!((r.user_turns, r.turns_with_read, r.palace_reads), (1, 0, 0));
        assert_eq!(r.read_rate_pct, Some(0.0));
    }

    #[test]
    fn dotdot_cannot_escape_or_sneak_in() {
        let r = run(&[
            turn("s", TS),
            tool(
                "s",
                "2026-11-19T10:00:01Z",
                "t1",
                "Read",
                r#"{"file_path":"/home/u/palace/../other/a.md"}"#,
            ),
            tool(
                "s",
                "2026-11-19T10:00:02Z",
                "t2",
                "Read",
                r#"{"file_path":"/home/u/palace-old/a.md"}"#,
            ),
        ]);
        assert_eq!(
            r.palace_reads, 0,
            "neither a folded escape nor a prefix sibling is the palace"
        );
    }

    #[test]
    fn shell_touch_needs_a_whole_path() {
        let r = run(&[
            turn("s", TS),
            tool(
                "s",
                "2026-11-19T10:00:01Z",
                "t1",
                "Bash",
                r#"{"command":"cat /home/u/palace-old/a.md"}"#,
            ),
            tool(
                "s",
                "2026-11-19T10:00:02Z",
                "t2",
                "Bash",
                r#"{"command":"ls /mnt/home/u/palace/x"}"#,
            ),
        ]);
        assert_eq!(
            r.shell_touches, 0,
            "a sibling prefix or a longer path is not the palace"
        );
        let r = run(&[
            turn("s", TS),
            tool(
                "s",
                "2026-11-19T10:00:01Z",
                "t1",
                "Bash",
                r#"{"command":"cd \"/home/u/palace\" && ls"}"#,
            ),
            tool(
                "s",
                "2026-11-19T10:00:02Z",
                "t2",
                "Bash",
                r#"{"command":"grep x /home/u/palace"}"#,
            ),
        ]);
        assert_eq!(
            r.shell_touches, 2,
            "quoted or at the end of the command, it is"
        );
    }

    #[test]
    fn relative_root_is_made_absolute() {
        let v = roots_for(Path::new("palace"));
        assert!(v.iter().all(|r| r.is_absolute()));
    }

    fn user(sess: &str, origin: Option<&str>, content: &str) -> String {
        let o = origin.map_or(String::new(), |k| format!(r#""origin":{{"kind":"{k}"}},"#));
        format!(
            r#"{{"type":"user",{o}"sessionId":"{sess}","timestamp":"{TS}","cwd":"/home/u/palace","message":{{"role":"user","content":{content}}}}}"#
        )
    }

    #[test]
    fn only_human_origin_counts_when_origin_is_present() {
        let r = run(&[
            user("s", Some("human"), r#""a typed prompt""#),
            user(
                "s",
                Some("task-notification"),
                r#""<task-notification>done</task-notification>""#,
            ),
            user(
                "s",
                Some("human"),
                r#"[{"type":"text","text":"also typed"}]"#,
            ),
        ]);
        assert_eq!(r.user_turns, 2, "a task notification is not a turn");
    }

    #[test]
    fn harness_shapes_are_not_turns_when_origin_is_absent() {
        let shapes = [
            r#""<task-notification>x</task-notification>""#,
            r#""<system-reminder>x</system-reminder>""#,
            r#""[Request interrupted by user]""#,
            r#""<command-name>/clear</command-name>""#,
            r#""<bash-input>ls</bash-input>""#,
            r#""<scheduled-task name=\"x\">go</scheduled-task>""#,
            r#"[{"type":"text","text":"<system-reminder>x</system-reminder>"}]"#,
        ];
        let mut lines: Vec<String> = shapes.iter().map(|c| user("s", None, c)).collect();
        lines.push(user("s", None, r#""plain words from a person""#));
        assert_eq!(run(&lines).user_turns, 1);
    }

    #[test]
    fn a_notification_does_not_close_the_human_turns_read_window() {
        let r = run(&[
            user("s", Some("human"), r#""check the palace""#),
            user(
                "s",
                Some("task-notification"),
                r#""<task-notification>x</task-notification>""#,
            ),
            tool(
                "s",
                "2026-11-19T10:00:05Z",
                "t1",
                "Read",
                r#"{"file_path":"/home/u/palace/a.md"}"#,
            ),
        ]);
        assert_eq!((r.user_turns, r.turns_with_read), (1, 1));
    }

    #[test]
    fn pathless_search_outside_the_palace_is_not_a_read() {
        let line = r#"{"type":"assistant","sessionId":"s","timestamp":"2026-11-19T10:00:05Z","cwd":"/home/u/other","message":{"content":[{"type":"tool_use","id":"t1","name":"Glob","input":{"pattern":"**/*.md"}}]}}"#.to_string();
        let r = run(&[turn("s", TS), line]);
        assert_eq!(r.palace_reads, 0);
    }

    #[test]
    fn relative_path_resolves_against_cwd() {
        let r = run(&[
            turn("s", TS),
            tool(
                "s",
                "2026-11-19T10:00:05Z",
                "t1",
                "Grep",
                r#"{"pattern":"x","path":"memory"}"#,
            ),
        ]);
        assert_eq!(r.palace_reads, 1);
    }

    #[test]
    fn search_with_no_path_reads_the_cwd() {
        let r = run(&[
            turn("s", TS),
            tool(
                "s",
                "2026-11-19T10:00:05Z",
                "t1",
                "Glob",
                r#"{"pattern":"**/*.md"}"#,
            ),
        ]);
        assert_eq!(r.palace_reads, 1);
    }

    #[test]
    fn writes_and_shell_touches_are_counted_apart() {
        let r = run(&[
            turn("s", TS),
            tool(
                "s",
                "2026-11-19T10:00:01Z",
                "t1",
                "Edit",
                r#"{"file_path":"/home/u/palace/a.md"}"#,
            ),
            tool(
                "s",
                "2026-11-19T10:00:02Z",
                "t2",
                "Bash",
                r#"{"command":"cat /home/u/palace/a.md"}"#,
            ),
        ]);
        assert_eq!(
            (
                r.palace_writes,
                r.shell_touches,
                r.palace_reads,
                r.turns_with_read
            ),
            (1, 1, 0, 0)
        );
        assert_eq!(
            (r.turns_with_touch, r.touch_rate_pct),
            (1, Some(100.0)),
            "a shell touch lifts only the touch rate"
        );
    }

    #[test]
    fn tool_results_meta_and_command_echoes_are_not_turns() {
        let lines = vec![
            r#"{"type":"user","sessionId":"s","timestamp":"2026-11-19T10:00:00Z","message":{"content":[{"type":"tool_result","content":"x"}]}}"#.to_string(),
            r#"{"type":"user","isMeta":true,"sessionId":"s","timestamp":"2026-11-19T10:00:00Z","message":{"content":"x"}}"#.to_string(),
            r#"{"type":"user","isCompactSummary":true,"sessionId":"s","timestamp":"2026-11-19T10:00:00Z","message":{"content":"x"}}"#.to_string(),
            r#"{"type":"user","sessionId":"s","timestamp":"2026-11-19T10:00:00Z","message":{"content":"<local-command-stdout>ok</local-command-stdout>"}}"#.to_string(),
            r#"{"type":"user","sessionId":"s","timestamp":"2026-11-19T10:00:00Z","message":{"content":[{"type":"text","text":"real"}]}}"#.to_string(),
        ];
        assert_eq!(run(&lines).user_turns, 1);
    }

    #[test]
    fn a_turn_counts_once_however_many_reads_follow() {
        let r = run(&[
            turn("s", TS),
            tool(
                "s",
                "2026-11-19T10:00:01Z",
                "t1",
                "Read",
                r#"{"file_path":"/home/u/palace/a.md"}"#,
            ),
            tool(
                "s",
                "2026-11-19T10:00:02Z",
                "t2",
                "Read",
                r#"{"file_path":"/home/u/palace/b.md"}"#,
            ),
            turn("s", "2026-11-19T10:01:00Z"),
        ]);
        assert_eq!((r.user_turns, r.turns_with_read, r.palace_reads), (2, 1, 2));
        assert_eq!(r.read_rate_pct, Some(50.0));
        assert_eq!(r.turns_with_touch, 1, "a read is also a touch");
    }

    #[test]
    fn duplicate_tool_ids_count_once() {
        let l = tool(
            "s",
            "2026-11-19T10:00:01Z",
            "t1",
            "Read",
            r#"{"file_path":"/home/u/palace/a.md"}"#,
        );
        let r = run(&[turn("s", TS), l.clone(), l]);
        assert_eq!(r.palace_reads, 1);
    }

    #[test]
    fn sidechain_reads_are_delegated_not_rated() {
        let side = r#"{"type":"assistant","isSidechain":true,"sessionId":"s","timestamp":"2026-11-19T10:00:01Z","cwd":"/home/u/palace","message":{"content":[{"type":"tool_use","id":"t9","name":"Read","input":{"file_path":"/home/u/palace/a.md"}}]}}"#.to_string();
        let side_turn = r#"{"type":"user","isSidechain":true,"sessionId":"s","timestamp":"2026-11-19T10:00:00Z","message":{"content":"brief"}}"#.to_string();
        let r = run(&[turn("s", TS), side_turn, side]);
        assert_eq!(
            (
                r.user_turns,
                r.turns_with_read,
                r.palace_reads,
                r.delegated_reads
            ),
            (1, 0, 0, 1)
        );
    }

    #[test]
    fn entries_before_the_horizon_are_ignored() {
        let lines = [turn("s", "2026-01-01T00:00:00Z")];
        let horizon = tokens::parse_rfc3339("2026-06-01T00:00:00Z").unwrap();
        let r = tally(
            lines.iter().map(|l| (0usize, l.as_str())),
            &roots(),
            horizon,
        );
        assert_eq!(r.user_turns, 0);
        assert_eq!(r.read_rate_pct, None);
    }

    #[test]
    fn sessions_are_kept_apart() {
        let r = run(&[
            turn("a", TS),
            turn("b", TS),
            tool(
                "a",
                "2026-11-19T10:00:05Z",
                "t1",
                "Read",
                r#"{"file_path":"/home/u/palace/a.md"}"#,
            ),
        ]);
        assert_eq!(
            (
                r.sessions,
                r.sessions_with_read,
                r.user_turns,
                r.turns_with_read
            ),
            (2, 1, 2, 1)
        );
    }

    #[test]
    fn a_turn_copied_into_a_second_file_counts_once() {
        // Resume or fork: the same entries (same uuid, same tool id) land in a
        // new transcript file under a new sessionId.
        let t1 = r#"{"type":"user","uuid":"u-1","sessionId":"a","timestamp":"2026-11-19T10:00:00Z","cwd":"/home/u/palace","message":{"role":"user","content":"hello"}}"#;
        let r1 = tool(
            "a",
            "2026-11-19T10:00:01Z",
            "t1",
            "Read",
            r#"{"file_path":"/home/u/palace/a.md"}"#,
        );
        let t2 = t1.replace(r#""sessionId":"a""#, r#""sessionId":"b""#);
        let r2 = r1.replace(r#""sessionId":"a""#, r#""sessionId":"b""#);
        let lines = [
            (0usize, t1),
            (0, r1.as_str()),
            (1, t2.as_str()),
            (1, r2.as_str()),
        ];
        let r = tally(lines, &roots(), 0);
        assert_eq!((r.user_turns, r.turns_with_read), (1, 1));
        assert_eq!(r.sessions, 1, "a copied history is not a second session");
    }
}
