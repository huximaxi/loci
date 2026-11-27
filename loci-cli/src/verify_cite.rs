//! `loci verify-cite <answer-file>`: check an answer's citations against the files, with no model.
//!
//! Every `path:line "quoted claim"` in the answer is looked up under one root: the file must be a
//! corpus file (the ones `recall` reads), the line range must exist, and the quoted string must be
//! on those lines. **Presence only.** It does not prove the string supports the sentence around it,
//! and the output says so. Experimental: the JSON is not frozen.
//!
//! Read-only and offline. A path that leaves the root (`../x.md`) is reported and never opened, a
//! file that is not a corpus file is reported without being looked at, and a symlink is never
//! followed, so the command cannot be used to learn what a file elsewhere holds or whether it
//! exists. (A symlink under the root reads as `not_in_corpus` where a missing name reads as
//! `no_file`, so that a link is there can be told; where it points never can.)
//!
//! Cite forms (every one reduces to path, first line, last line):
//! - S1 `path:N` and `path:N-M`
//! - S2 the same with an `L` before either number: `notes/x.md:L10-L20`
//! - S3 a link anchor, bare or inside a markdown link: `x.md#L10-L20`, `[Title](x.md#L10)`
//!
//! A path has no whitespace, quote, `#`, bracket, comma, semicolon or further `:`, is at most 4096
//! characters, has a letter, and either contains `/` or ends in `.md`, which keeps `10:30`, `3/4:5`,
//! `v2:1` and URLs out. The claim is the first double-quoted span after
//! the cite on the same line, with only spaces and `) ] ` : , -` between. Matching collapses runs
//! of whitespace, joins the cited lines with one space, and is a case-sensitive substring: no
//! fuzzy matching and no Unicode normalisation, so the bytes the answer quotes are the bytes the
//! file holds.

use crate::format2::Crystal;
use crate::recall::{self, Frame, Root};
use crate::{Error, Outcome};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

/// The answer file is read whole; past this it is refused.
/// A cited path longer than this is not a path: it keeps a hostile answer from making the scan or
/// the output grow with the square of its size.
const MAX_PATH_CHARS: usize = 4096;

const MAX_ANSWER_BYTES: u64 = recall::MAX_FILE_BYTES;

pub struct Args {
    pub answer_file: PathBuf,
    pub root: Option<PathBuf>,
    pub workfolder: Option<PathBuf>,
}

// ── finding the cites ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
struct Cite {
    /// The cite as written, for the report.
    text: String,
    style: &'static str,
    path: String,
    start: usize,
    end: usize,
    claim: Option<String>,
}

/// A character that can be part of a cited path. No whitespace, no quote of any kind, no `:`
/// (it separates the path from the numbers), and none of the characters that wrap a cite in prose.
fn path_char(c: char) -> bool {
    !(c.is_whitespace()
        || c.is_control()
        || matches!(
            c,
            ':' | '#'
                | '"'
                | '\''
                | '`'
                | '“'
                | '”'
                | '‘'
                | '’'
                | '('
                | ')'
                | '['
                | ']'
                | '<'
                | '>'
                | ','
                | ';'
                | '*'
        ))
}

/// Digits at `i`, as a number (saturating, so a huge one is simply out of range) and the index
/// after them.
fn number(c: &[char], mut i: usize) -> Option<(usize, usize)> {
    let from = i;
    let mut n: usize = 0;
    while i < c.len() && c[i].is_ascii_digit() {
        n = n
            .saturating_mul(10)
            .saturating_add(c[i].to_digit(10).unwrap_or(0) as usize);
        i += 1;
    }
    (i > from).then_some((n, i))
}

/// `L`? digits, then optionally `-` `L`? digits. Returns first, last and the index after.
fn line_range(c: &[char], i: usize, allow_l: bool) -> Option<(usize, usize, usize)> {
    let mut i = i;
    if allow_l && c.get(i) == Some(&'L') {
        i += 1;
    }
    let (a, j) = number(c, i)?;
    if c.get(j) == Some(&'-') {
        let mut k = j + 1;
        if c.get(k) == Some(&'L') {
            k += 1;
        }
        if let Some((b, e)) = number(c, k) {
            return Some((a, b, e));
        }
    }
    Some((a, a, j))
}

/// The path ends before `at`: take the characters that can be in a path, back to the first that
/// cannot. A path that has a `:` right in front of it is the tail of a URL or a drive, not a cite.
fn path_before(c: &[char], at: usize) -> Option<String> {
    let mut s = at;
    while s > 0 && path_char(c[s - 1]) {
        s -= 1;
        if at - s > MAX_PATH_CHARS {
            return None;
        }
    }
    if s == 0 || c[s - 1] != ':' {
        let p: String = c[s..at].iter().collect();
        // A path names something: it has a letter, so `3/4:5` is arithmetic, not a cite.
        let ok = p.chars().any(char::is_alphabetic) && (p.contains('/') || p.ends_with(".md"));
        return ok.then_some(p);
    }
    None
}

/// The first double-quoted span after `i`, with only spaces and `) ] ` : , -` between. `last` is
/// where the last `"` and the last `”` of the line are, so an opening mark with nothing to close
/// it is told at once: a line of openers costs its length once, not once per opener.
fn claim_after(c: &[char], mut i: usize, last: [Option<usize>; 2]) -> (Option<String>, usize) {
    while i < c.len()
        && (c[i] == ' ' || c[i] == '\t' || matches!(c[i], ')' | ']' | '`' | ':' | ',' | '-'))
    {
        i += 1;
    }
    let (close, kind) = match c.get(i) {
        Some('"') => ('"', 0),
        Some('“') => ('”', 1),
        _ => return (None, i.min(c.len())),
    };
    if !last[kind].is_some_and(|l| l > i) {
        return (None, i);
    }
    if let Some(len) = c[i + 1..].iter().position(|&x| x == close) {
        let claim: String = c[i + 1..i + 1 + len].iter().collect();
        (Some(claim), i + len + 2)
    } else {
        (None, i)
    }
}

fn find_cites(text: &str) -> Vec<Cite> {
    let mut out = Vec::new();
    for line in text.lines() {
        let c: Vec<char> = line.chars().collect();
        let last = [
            c.iter().rposition(|&x| x == '"'),
            c.iter().rposition(|&x| x == '”'),
        ];
        let mut i = 0;
        while i < c.len() {
            // S1 / S2: `:` then a number; S3: `#L` then a number.
            let (style, range, path_end) = match c[i] {
                ':' => ("S1", line_range(&c, i + 1, true), i),
                '#' if c.get(i + 1) == Some(&'L') => ("S3", line_range(&c, i + 1, true), i),
                _ => {
                    i += 1;
                    continue;
                }
            };
            let Some((start, end, after)) = range else {
                i += 1;
                continue;
            };
            let Some(path) = path_before(&c, path_end) else {
                i += 1;
                continue;
            };
            // S2 also covers an `L` on the second number only.
            let style = if style == "S1" && c[i + 1..after].contains(&'L') {
                "S2"
            } else {
                style
            };
            let (claim, next) = claim_after(&c, after, last);
            let from = c[..path_end].len() - path.chars().count();
            let shown: String = c[from..after].iter().collect();
            out.push(Cite {
                text: shown,
                style,
                path,
                start,
                end,
                claim,
            });
            i = next.max(after);
        }
    }
    out
}

// ── resolving one cite ───────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Status {
    Ok,
    Unclaimed,
    NoFile,
    NotInCorpus,
    OutsideWorkfolder,
    LineOutOfRange,
    BadRange,
    ClaimNotFound,
    Unreadable,
}

impl Status {
    fn name(self) -> &'static str {
        match self {
            Status::Ok => "ok",
            Status::Unclaimed => "unclaimed",
            Status::NoFile => "no_file",
            Status::NotInCorpus => "not_in_corpus",
            Status::OutsideWorkfolder => "outside_workfolder",
            Status::LineOutOfRange => "line_out_of_range",
            Status::BadRange => "bad_range",
            Status::ClaimNotFound => "claim_not_found",
            Status::Unreadable => "unreadable",
        }
    }
}

/// A corpus file that was read: its text and its crystals.
struct Loaded {
    /// The path as the disk spells it, relative to the root.
    rel: String,
    flat: Flat,
    crystals: Vec<Crystal>,
}

/// A file's lines with every run of whitespace folded to one space, made once per file so that a
/// cite costs one search and never a rebuild of the lines it cites (a hostile answer may cite the
/// whole file thousands of times).
struct Flat {
    text: String,
    /// Where line `i` (0-based) starts and ends in `text`. A blank line is empty, at the end of
    /// what came before it.
    from: Vec<usize>,
    to: Vec<usize>,
}

impl Flat {
    fn of(raw: &str) -> Flat {
        let mut text = String::new();
        let (mut from, mut to) = (Vec::new(), Vec::new());
        for line in raw.lines() {
            let folded = collapse(line);
            if !folded.is_empty() {
                if !text.is_empty() {
                    text.push(' ');
                }
                from.push(text.len());
                text.push_str(&folded);
            } else {
                from.push(text.len());
            }
            to.push(text.len());
        }
        Flat { text, from, to }
    }

    fn line_count(&self) -> usize {
        self.from.len()
    }

    /// Lines `first` to `last` (1-based, both in range), collapsed and joined by one space: what
    /// `collapse` makes of those lines joined by a space.
    fn range(&self, first: usize, last: usize) -> &str {
        self.text[self.from[first - 1]..self.to[last - 1]].trim_start()
    }
}

enum Opened {
    File(Loaded),
    Missing,
    NotCorpus,
    Unreadable(&'static str),
}

/// Fold `.` and `..` without touching the disk. `None` when the path climbs above the root.
fn fold(path: &str, root: &Root) -> Option<Vec<String>> {
    let mut rest = path;
    if path.starts_with('/') {
        // An absolute path is only in range when it names something under the root.
        let abs = Path::new(path);
        let mut roots = vec![root.path.clone()];
        if let Ok(c) = fs::canonicalize(&root.path) {
            roots.push(c);
        }
        let under = roots
            .iter()
            .find_map(|r| abs.strip_prefix(r).ok().map(Path::to_path_buf))?;
        let rel = under.to_string_lossy().to_string();
        return fold(&rel, root).filter(|_| !rel.starts_with('/'));
    }
    if let Some(r) = rest.strip_prefix("./") {
        rest = r;
    }
    let mut parts: Vec<String> = Vec::new();
    for comp in rest.split('/') {
        match comp {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            other => parts.push(other.to_string()),
        }
    }
    Some(parts)
}

/// Open the file a folded path names, if it is a corpus file. Nothing is followed: a symlink on
/// the way is `NotCorpus`, as it is for the walk `recall` does.
fn open(root: &Root, frame: &Frame, parts: &[String]) -> Opened {
    // What is in the corpus is decided from the name alone, before the disk is touched, so the
    // answer for a hidden directory or a file that is not markdown never depends on what is there.
    let Some((file, dirs)) = parts.split_last() else {
        return Opened::Missing;
    };
    // Names are compared in lower case: on a case-insensitive disk `Cron/` is the `cron/` that
    // `recall` skips, and the answer must not depend on how the cite spells it.
    if Path::new(file).extension().and_then(|x| x.to_str()) != Some("md")
        || dirs.iter().any(|d| recall::skipped_dir(&d.to_lowercase()))
    {
        return Opened::NotCorpus;
    }
    let mut cur = root.path.clone();
    for (n, part) in parts.iter().enumerate() {
        cur.push(part);
        let Ok(meta) = fs::symlink_metadata(&cur) else {
            return Opened::Missing;
        };
        if meta.file_type().is_symlink() {
            return Opened::NotCorpus;
        }
        let last = n + 1 == parts.len();
        if last != meta.is_file() {
            return Opened::Missing;
        }
        if last {
            // The name as the disk spells it, so a cite that gets the case wrong is read as the
            // file it reaches (same crystals, same stale warning) and counts as one file. It is
            // settled before the file is read: a file under a skipped directory is refused unread.
            let (Some(real), Some(base)) = (
                fs::canonicalize(&cur).ok(),
                fs::canonicalize(&root.path).ok(),
            ) else {
                return Opened::Unreadable("unreadable");
            };
            let Ok(under) = real.strip_prefix(&base) else {
                return Opened::NotCorpus;
            };
            let names: Vec<String> = under
                .components()
                .map(|c| c.as_os_str().to_string_lossy().to_string())
                .collect();
            // Directories are skipped by name, files are not: a hidden `.md` file is in the corpus
            // here exactly as `recall` reads it, and the cite `recall` prints for it must verify.
            let skipped_above = names.split_last().is_some_and(|(_, dirs)| {
                dirs.iter().any(|d| recall::skipped_dir(&d.to_lowercase()))
            });
            if skipped_above {
                return Opened::NotCorpus;
            }
            let rel = names.join("/");
            let bytes = match recall::read_capped(&cur, &meta) {
                Ok(Some(b)) => b,
                Ok(None) => return Opened::Unreadable("over 1 MiB"),
                Err(_) => return Opened::Unreadable("unreadable"),
            };
            let Ok(text) = String::from_utf8(bytes) else {
                return Opened::Unreadable("not UTF-8");
            };
            let text = text.strip_prefix('\u{feff}').unwrap_or(&text).to_string();
            let (crystals, _) = frame.crystals_in(&rel, &text);
            return Opened::File(Loaded {
                rel,
                flat: Flat::of(&text),
                crystals,
            });
        }
    }
    Opened::Missing
}

fn collapse(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// `stale: superseded` or `stale: retired` when the cited lines touch such a crystal. Any overlap
/// counts: a range that spills past the crystal must not hide the warning.
fn stale(f: &Loaded, start: usize, end: usize) -> Option<&'static str> {
    let mut found = None;
    for c in &f.crystals {
        if c.line <= end && start <= c.end_line {
            if c.is_retired() {
                return Some("stale: retired");
            }
            if c.superseded_by.is_some() {
                found = Some("stale: superseded");
            }
        }
    }
    found
}

#[derive(Serialize)]
struct CiteOut {
    cite: String,
    style: &'static str,
    root: String,
    path: String,
    line: usize,
    end_line: usize,
    claim: Option<String>,
    status: Status,
    warning: Option<&'static str>,
}

#[derive(Default, Serialize)]
struct Counts {
    total: usize,
    ok: usize,
    ungrounded: usize,
    unclaimed: usize,
}

#[derive(Default, Serialize)]
struct NotRead {
    total: usize,
    not_utf8: usize,
    over_cap: usize,
    unreadable: usize,
}

#[derive(Serialize)]
struct Scope {
    roots: Vec<RootOut>,
    files_read: usize,
    not_read: NotRead,
}

#[derive(Serialize)]
struct RootOut {
    tag: String,
    kind: &'static str,
    layout: &'static str,
}

#[derive(Serialize)]
struct Out {
    answer_file: String,
    scope: Scope,
    cites: Vec<CiteOut>,
    counts: Counts,
    verified: usize,
}

fn check(cites: Vec<Cite>, root: &Root, answer_file: &str) -> Out {
    let tag = recall::root_tag(&root.path);
    let frame = Frame::of(root);
    let mut cache: HashMap<String, Opened> = HashMap::new();
    let mut out = Vec::new();
    let mut not_read = NotRead::default();
    let mut counts = Counts::default();

    for c in cites {
        let mut warning = None;
        let parts = fold(&c.path, root);
        let shown = parts
            .as_ref()
            .map(|p| p.join("/"))
            .unwrap_or_else(|| c.path.clone());
        let status = if c.start == 0 || c.start > c.end {
            Status::BadRange
        } else if let Some(parts) = parts {
            let key = parts.join("/");
            if !cache.contains_key(&key) {
                let o = open(root, &frame, &parts);
                match &o {
                    Opened::File(_) => {}
                    Opened::Unreadable(why) => {
                        not_read.total += 1;
                        match *why {
                            "over 1 MiB" => not_read.over_cap += 1,
                            "not UTF-8" => not_read.not_utf8 += 1,
                            _ => not_read.unreadable += 1,
                        }
                    }
                    _ => {}
                }
                cache.insert(key.clone(), o);
            }
            match &cache[&key] {
                Opened::Missing => Status::NoFile,
                Opened::NotCorpus => Status::NotInCorpus,
                Opened::Unreadable(_) => Status::Unreadable,
                Opened::File(f) => {
                    if c.end > f.flat.line_count() {
                        Status::LineOutOfRange
                    } else {
                        match &c.claim {
                            None => Status::Unclaimed,
                            Some(claim) => {
                                let want = collapse(claim);
                                if want.is_empty() {
                                    // Nothing was quoted: an empty string is in every line.
                                    Status::Unclaimed
                                } else if f.flat.range(c.start, c.end).contains(&want) {
                                    warning = stale(f, c.start, c.end);
                                    Status::Ok
                                } else {
                                    Status::ClaimNotFound
                                }
                            }
                        }
                    }
                }
            }
        } else {
            Status::OutsideWorkfolder
        };
        counts.total += 1;
        match status {
            Status::Ok => counts.ok += 1,
            Status::Unclaimed => {
                counts.unclaimed += 1;
                counts.ungrounded += 1;
            }
            _ => counts.ungrounded += 1,
        }
        out.push(CiteOut {
            cite: c.text,
            style: c.style,
            root: tag.clone(),
            path: shown,
            line: c.start,
            end_line: c.end,
            claim: c.claim,
            status,
            warning,
        });
    }
    let files_read = cache
        .values()
        .filter_map(|o| match o {
            Opened::File(f) => Some(f.rel.as_str()),
            _ => None,
        })
        .collect::<std::collections::HashSet<_>>()
        .len();
    Out {
        answer_file: answer_file.to_string(),
        scope: Scope {
            roots: vec![RootOut {
                tag,
                kind: if root.palace.is_some() {
                    "palace"
                } else {
                    "plain"
                },
                layout: root
                    .palace
                    .as_ref()
                    .map(|p| recall::layout_name(&p.layout))
                    .unwrap_or("plain"),
            }],
            files_read,
            not_read,
        },
        verified: counts.ok,
        counts,
        cites: out,
    }
}

// ── output ───────────────────────────────────────────────────────────────

fn scope_line(s: &Scope) -> String {
    let mut line = format!(
        "scope: {} {} read",
        s.files_read,
        if s.files_read == 1 { "file" } else { "files" }
    );
    if s.not_read.total > 0 {
        let mut why = Vec::new();
        if s.not_read.not_utf8 > 0 {
            why.push(format!("{} not UTF-8", s.not_read.not_utf8));
        }
        if s.not_read.over_cap > 0 {
            why.push(format!("{} over 1 MiB", s.not_read.over_cap));
        }
        if s.not_read.unreadable > 0 {
            why.push(format!("{} unreadable", s.not_read.unreadable));
        }
        line.push_str(&format!(
            ", {} not read ({})",
            s.not_read.total,
            why.join(", ")
        ));
    }
    line
}

fn print_text(out: &Out) {
    // Every line goes through `plain`, so nothing from the answer or the files reaches the
    // terminal raw.
    let p = |s: String| std::println!("{}", recall::plain(&s));
    for c in &out.cites {
        let mut line = format!("{:<18} {}", c.status.name(), c.cite);
        if c.path != c.cite.split([':', '#']).next().unwrap_or("") {
            line.push_str(&format!("  (as {}:{})", c.path, c.line));
        }
        if let Some(claim) = &c.claim {
            let short: String = claim.chars().take(60).collect();
            line.push_str(&format!(
                "  \"{short}{}\"",
                if claim.chars().count() > 60 {
                    "…"
                } else {
                    ""
                }
            ));
        }
        if let Some(w) = c.warning {
            line.push_str(&format!("  [{w}]"));
        }
        p(line);
    }
    if out.counts.total == 0 {
        p("0 cites found: nothing was verified".to_string());
    } else {
        p(format!(
            "{} {}: {} ok, {} ungrounded",
            out.counts.total,
            if out.counts.total == 1 {
                "cite"
            } else {
                "cites"
            },
            out.counts.ok,
            out.counts.ungrounded
        ));
    }
    p("this checks that quoted strings are present, not that they support the answer".to_string());
    p(scope_line(&out.scope));
}

// ── entry ────────────────────────────────────────────────────────────────

pub fn run(args: Args, json: bool) -> Result<Outcome, Error> {
    let shown = args.answer_file.display().to_string();
    let meta = fs::metadata(&args.answer_file)
        .map_err(|e| Error::bad_input(format!("cannot read the answer file {shown}: {e}")))?;
    if !meta.is_file() {
        return Err(Error::bad_input(format!(
            "the answer file {shown} is not a regular file"
        )));
    }
    let mut bytes = Vec::new();
    fs::File::open(&args.answer_file)
        .and_then(|f| f.take(MAX_ANSWER_BYTES + 1).read_to_end(&mut bytes))
        .map_err(|e| Error::bad_input(format!("cannot read the answer file {shown}: {e}")))?;
    if bytes.len() as u64 > MAX_ANSWER_BYTES {
        return Err(Error::bad_input(format!(
            "the answer file {shown} is over 1 MiB"
        )));
    }
    let text = String::from_utf8(bytes)
        .map_err(|_| Error::bad_input(format!("the answer file {shown} is not UTF-8")))?;
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);

    let root = recall::resolve_root(
        args.root,
        args.workfolder,
        std::env::var_os("LOCI_WORKFOLDER"),
        std::env::var_os("LOCI_PALACE"),
        std::env::current_dir().ok(),
    )?;
    let out = check(find_cites(text), &root, &shown);
    if json {
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        print_text(&out);
    }
    Ok(if out.counts.ungrounded == 0 {
        Outcome::Clean
    } else {
        Outcome::Findings
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(text: &str) -> Cite {
        let mut v = find_cites(text);
        assert_eq!(v.len(), 1, "{text:?} gave {v:?}");
        v.remove(0)
    }

    #[test]
    fn the_three_styles_reduce_to_a_path_and_a_range() {
        let c = one("see rooms/work/decision.md:9 \"We use Postgres.\"");
        assert_eq!(
            (c.style, c.path.as_str(), c.start, c.end),
            ("S1", "rooms/work/decision.md", 9, 9)
        );
        assert_eq!(c.claim.as_deref(), Some("We use Postgres."));
        let c = one("notes/x.md:L10-L20 \"a\"");
        assert_eq!((c.style, c.start, c.end), ("S2", 10, 20));
        let c = one("[Title](x.md#L10-L12 \"a\")");
        assert_eq!(
            (c.style, c.path.as_str(), c.start, c.end),
            ("S3", "x.md", 10, 12)
        );
        assert_eq!(c.claim.as_deref(), Some("a"));
        let c = one("a/b.md:3-5 \"q\"");
        assert_eq!((c.style, c.start, c.end), ("S1", 3, 5));
    }

    #[test]
    fn times_versions_and_urls_are_not_cites() {
        for t in [
            "Standup at 10:30 daily",
            "v2:1 is out",
            "see https://example.com/a/b.md:3 \"x\"",
            "http://host:8080/x",
            "a note about nothing",
            "10:30-11:30 is the slot",
        ] {
            assert!(find_cites(t).is_empty(), "{t:?} gave {:?}", find_cites(t));
        }
    }

    #[test]
    fn a_claim_must_follow_the_cite_with_only_the_allowed_marks_between() {
        assert!(one("a.md:1").claim.is_none());
        assert!(one("a.md:1 says \"x\"").claim.is_none());
        assert_eq!(one("a.md:1 - \"x\"").claim.as_deref(), Some("x"));
        assert_eq!(one("(a.md:1): “x”").claim.as_deref(), Some("x"));
        assert!(one("\"x\" a.md:1").claim.is_none());
        assert!(one("a.md:1 \"never closed").claim.is_none());
    }

    #[test]
    fn two_cites_on_one_line_each_take_their_own_claim() {
        let v = find_cites("a.md:1 \"x\" and b/c.md:2 \"y\"");
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].claim.as_deref(), Some("x"));
        assert_eq!(v[1].path, "b/c.md");
        assert_eq!(v[1].claim.as_deref(), Some("y"));
    }

    #[test]
    fn a_cite_inside_a_quoted_claim_is_not_a_second_cite() {
        let v = find_cites("a.md:1 \"look at b.md:2 instead\"");
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn whitespace_runs_collapse_and_case_matters() {
        assert_eq!(collapse("a   b\t c"), "a b c");
        assert!(!collapse("We use postgres").contains("We use Postgres"));
    }

    #[test]
    fn a_flat_range_is_what_collapsing_the_joined_lines_gives() {
        let raw = "  a   b \n\n\tc\r\nd e\n   \nf\n";
        let lines: Vec<&str> = raw.lines().collect();
        let flat = Flat::of(raw);
        assert_eq!(flat.line_count(), lines.len());
        for first in 1..=lines.len() {
            for last in first..=lines.len() {
                assert_eq!(
                    flat.range(first, last),
                    collapse(&lines[first - 1..last].join(" ")),
                    "lines {first}-{last}"
                );
            }
        }
        assert_eq!(Flat::of("").line_count(), 0);
    }

    #[test]
    fn a_line_of_openers_that_never_close_costs_its_length_once() {
        // Each opener used to scan to the end of the line for its closer: quadratic. Both kinds.
        let t = std::time::Instant::now();
        for (open, n) in [("“", 60_000), ("\"", 1)] {
            let line = format!("{}\n", format!("a.md:1 {open}").repeat(n));
            let cites = find_cites(&line);
            assert_eq!(cites.len(), n);
            assert!(cites.iter().all(|c| c.claim.is_none()));
        }
        assert!(t.elapsed().as_secs() < 10, "took {:?}", t.elapsed());
    }

    #[test]
    fn an_opener_with_a_closer_after_it_still_takes_the_claim() {
        let v = find_cites("a.md:1 “x” b.md:2 “never closed");
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].claim.as_deref(), Some("x"));
        assert!(v[1].claim.is_none());
        // A straight quote closes only a straight quote, and a curly one only a curly one.
        assert!(one("a.md:1 “x\"").claim.is_none());
        assert!(one("a.md:1 \"x”").claim.is_none());
    }

    #[test]
    fn folding_stops_at_the_root() {
        let root = Root {
            path: std::env::temp_dir(),
            palace: None,
        };
        assert_eq!(
            fold("a/./b/../c.md", &root),
            Some(vec!["a".into(), "c.md".into()])
        );
        assert_eq!(fold("../x.md", &root), None);
        assert_eq!(fold("a/../../x.md", &root), None);
        assert_eq!(fold("/etc/passwd", &root), None);
    }
}
