//! `loci recall <query>`: ask a workfolder a question without a model.
//!
//! A lexical search over every `.md` file under one root: BM25 (k1 1.2, b 0.75) over paragraph
//! chunks and crystals, one statistics set per run, nothing persisted, no network, no clock but
//! `--as-of`. A result carries its state (current, due, superseded, retired, none) so a belief
//! that was replaced never ranks above the one that replaced it, and a cite (`path:line`) that
//! `verify-cite` will accept. Experimental: the JSON is not frozen.
//!
//! The tokenizer is `unicode-alnum-v1`: Unicode lowercase, maximal runs of alphanumeric
//! characters, and a dotted or hyphenated compound (`2.5.0`, `well-known`) also yields itself
//! next to its parts. No stemming, no stopwords, no diacritic folding, no script segmentation.
//! Any change to that rule bumps the id, so a result can be reproduced elsewhere.
//!
//! Nothing a folder holds may crash the command or vanish quietly: every `.md` file that was not read
//! is counted in `scope`, and those counts never change the exit code.

use crate::format2::{self, Crystal};
use crate::palace::{self, Layout, Palace};
use crate::tokens;
use crate::Error;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

pub const TOKENIZER: &str = "unicode-alnum-v1";
pub const SCORING: &str = "bm25(k1=1.2,b=0.75)";
const K1: f64 = 1.2;
const B: f64 = 0.75;

/// A file over this is counted as not read, never half-read.
const MAX_FILE_BYTES: u64 = 1024 * 1024;
/// The walk stops after this many `.md` files, in walk order, and says so.
const MAX_FILES: usize = 20_000;
/// The walk stops after this many chunks, so a folder of one-character paragraphs cannot
/// exhaust memory, and says so.
const MAX_CHUNKS: usize = 500_000;
/// A paragraph chunk never spans more than this many lines.
const MAX_CHUNK_LINES: usize = 40;
/// Frontmatter keys passed through on a hit, and the length each value is cut to.
const MAX_FM_KEYS: usize = 20;
const MAX_FM_VALUE: usize = 200;
/// Frontmatter keys whose values are searchable text. Dates and everything else are not.
const FM_TEXT_KEYS: [&str; 5] = ["title", "name", "description", "aliases", "tags"];
/// Directory names the walk skips besides hidden ones (the names the workfolder scan skips).
const SKIP_DIRS: [&str; 3] = ["node_modules", "target", "cron"];

pub struct Args {
    pub query: String,
    pub root: Option<PathBuf>,
    pub workfolder: Option<PathBuf>,
    pub limit: usize,
    pub room: Option<String>,
    pub as_of: Option<String>,
}

// ── roots ────────────────────────────────────────────────────────────────

/// The folder a run reads, and the palace reading of it when it is one.
struct Root {
    path: PathBuf,
    palace: Option<Palace>,
}

/// Which folder to read, first rule that applies: `--root` (any directory), `--workfolder`
/// (a palace), `$LOCI_WORKFOLDER` then `$LOCI_PALACE` (any directory; the CLI-wide rule, so the
/// first one set wins), then a walk up from `cwd` for a palace marker. A set variable that names
/// no directory is an error: the run does not fall back to the current directory. A plain folder
/// is only ever read when it is named, never found by walking, so `$HOME` is not indexed by
/// accident.
fn resolve_root(
    root_flag: Option<PathBuf>,
    workfolder_flag: Option<PathBuf>,
    env_workfolder: Option<OsString>,
    env_palace: Option<OsString>,
    cwd: Option<PathBuf>,
) -> Result<Root, Error> {
    if let Some(path) = root_flag {
        if !path.is_dir() {
            return Err(Error::not_found(format!(
                "--root {} is not a directory",
                path.display()
            )));
        }
        let palace = palace::validate(&path);
        return Ok(Root { path, palace });
    }
    if let Some(path) = workfolder_flag {
        return match palace::validate(&path) {
            Some(p) => Ok(Root {
                path: p.root.clone(),
                palace: Some(p),
            }),
            None => Err(Error::not_found(format!(
                "no workfolder at {}. To read a plain folder, pass --root <path>.",
                path.display()
            ))),
        };
    }
    let env = env_workfolder
        .filter(|e| !e.is_empty())
        .map(|e| ("LOCI_WORKFOLDER", e))
        .or_else(|| {
            env_palace
                .filter(|e| !e.is_empty())
                .map(|e| ("LOCI_PALACE", e))
        });
    if let Some((var, value)) = env {
        let path = PathBuf::from(value);
        if !path.is_dir() {
            return Err(Error::not_found(format!(
                "{var} is set to {} but no directory is there. Fix or unset it; \
                 loci does not fall back to the current directory.",
                path.display()
            )));
        }
        let palace = palace::validate(&path);
        return Ok(Root { path, palace });
    }
    let mut dir = cwd;
    while let Some(d) = dir {
        if let Some(p) = palace::validate(&d) {
            return Ok(Root {
                path: p.root.clone(),
                palace: Some(p),
            });
        }
        dir = d.parent().map(Path::to_path_buf);
    }
    Err(Error::not_found(
        "no workfolder found. Pass --root <path> (any folder of markdown), pass --workfolder \
         <path>, set LOCI_WORKFOLDER, or run from inside a workfolder."
            .to_string(),
    ))
}

// ── tokenizer ────────────────────────────────────────────────────────────

/// `unicode-alnum-v1`. See the module comment.
pub(crate) fn tokenize(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.to_lowercase().chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if !chars[i].is_alphanumeric() {
            i += 1;
            continue;
        }
        let mut parts: Vec<String> = Vec::new();
        let mut compound = String::new();
        loop {
            let start = i;
            while i < chars.len() && chars[i].is_alphanumeric() {
                i += 1;
            }
            let run: String = chars[start..i].iter().collect();
            compound.push_str(&run);
            parts.push(run);
            let joined = i + 1 < chars.len()
                && (chars[i] == '.' || chars[i] == '-')
                && chars[i + 1].is_alphanumeric();
            if joined {
                compound.push(chars[i]);
                i += 1;
            } else {
                break;
            }
        }
        if parts.len() > 1 {
            out.push(compound);
        }
        out.extend(parts);
    }
    out
}

/// The query's tokens, each once, in the order they first appear.
fn query_tokens(query: &str) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    tokenize(query)
        .into_iter()
        .filter(|t| seen.insert(t.clone()))
        .collect()
}

// ── the corpus ───────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Crystal,
    Note,
    Document,
}

impl Kind {
    fn as_str(self) -> &'static str {
        match self {
            Kind::Crystal => "crystal",
            Kind::Note => "note",
            Kind::Document => "document",
        }
    }
}

struct Chunk {
    file: usize,
    line: usize,
    end_line: usize,
    kind: Kind,
    crystal: Option<usize>,
    /// What the text output shows for a document chunk: its first line.
    snippet: String,
    /// Tokens in the chunk (document length for BM25).
    len: u32,
    /// Occurrences of each query token, aligned with the query's tokens.
    tf: Vec<u32>,
}

struct FileMeta {
    path: String,
    room: Option<String>,
    frontmatter: BTreeMap<String, String>,
}

#[derive(Default, Serialize)]
struct NotRead {
    total: usize,
    not_utf8: usize,
    over_cap: usize,
    unreadable: usize,
}

#[derive(Default, Serialize)]
struct Scope {
    roots: Vec<RootOut>,
    files_read: usize,
    chunks: usize,
    crystals_read: usize,
    not_read: NotRead,
    skipped_dirs: usize,
    skipped_symlinks: usize,
    malformed_frontmatter: usize,
    legacy_lines_not_counted: usize,
    truncated: bool,
}

#[derive(Serialize)]
struct RootOut {
    tag: String,
    kind: &'static str,
    layout: &'static str,
}

struct Corpus {
    files: Vec<FileMeta>,
    chunks: Vec<Chunk>,
    crystals: Vec<Crystal>,
    scope: Scope,
}

fn rel_path(root: &Path, p: &Path) -> String {
    p.strip_prefix(root)
        .unwrap_or(p)
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}

fn layout_name(l: &Layout) -> &'static str {
    match l {
        Layout::PalaceSubdir => "palace-subdir",
        Layout::RoomsDir => "rooms-dir",
        Layout::RoomsAtRoot => "rooms-at-root",
    }
}

fn root_tag(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .or_else(|| {
            fs::canonicalize(path)
                .ok()
                .and_then(|c| c.file_name().map(|n| n.to_string_lossy().to_string()))
        })
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| "root".to_string())
}

/// Read what a root holds. One walk reads everything, so nothing outside the root is ever
/// opened and no symlink is followed. A crystal is a crystal line in a container file (a root
/// marker, and in a workfolder each room's `CLAUDE.md`) or a `type: crystal` file; in a workfolder
/// a file crystal counts only under the folder the layout scans (and `garden/`), as the reference
/// reader does. The reference reader itself is not used here because it follows symlinks.
fn build_corpus(root: &Root, qtokens: &[String]) -> Corpus {
    let mut scope = Scope {
        roots: vec![RootOut {
            tag: root_tag(&root.path),
            kind: if root.palace.is_some() {
                "palace"
            } else {
                "plain"
            },
            layout: root
                .palace
                .as_ref()
                .map(|p| layout_name(&p.layout))
                .unwrap_or("plain"),
        }],
        ..Scope::default()
    };

    // Palace crystals, grouped by the file that holds them. Room membership comes with them.
    let mut crystals: Vec<Crystal> = Vec::new();
    let mut room_dirs: Vec<(String, String)> = Vec::new(); // (relative dir with trailing /, name)
    let mut scan_prefix = String::new(); // "" when the whole root is scanned
    if let Some(p) = &root.palace {
        for r in palace::list_rooms(p) {
            room_dirs.push((format!("{}/", rel_path(&p.root, &r.path)), r.name));
        }
        let s = rel_path(&p.root, &p.scan_root);
        if !s.is_empty() {
            scan_prefix = format!("{s}/");
        }
    }

    let mut files: Vec<FileMeta> = Vec::new();
    let mut chunks: Vec<Chunk> = Vec::new();
    let mut seen_md = 0usize;

    let mut stack: Vec<PathBuf> = vec![root.path.clone()];
    'walk: while let Some(dir) = stack.pop() {
        let Ok(rd) = fs::read_dir(&dir) else {
            scope.not_read.unreadable += 1;
            scope.not_read.total += 1;
            continue;
        };
        let mut entries: Vec<_> = Vec::new();
        for e in rd {
            match e {
                Ok(e) => entries.push(e),
                Err(_) => {
                    scope.not_read.unreadable += 1;
                    scope.not_read.total += 1;
                }
            }
        }
        entries.sort_by_key(|e| e.file_name());
        // Files in this directory are read now; subdirectories are pushed so the walk visits
        // them in name order after it.
        let mut subdirs: Vec<PathBuf> = Vec::new();
        for e in entries {
            let name = e.file_name().to_string_lossy().to_string();
            let path = e.path();
            let Ok(meta) = fs::symlink_metadata(&path) else {
                scope.not_read.unreadable += 1;
                scope.not_read.total += 1;
                continue;
            };
            if meta.file_type().is_symlink() {
                scope.skipped_symlinks += 1;
                continue;
            }
            if meta.is_dir() {
                if name.starts_with('.') || SKIP_DIRS.contains(&name.as_str()) {
                    scope.skipped_dirs += 1;
                } else {
                    subdirs.push(path);
                }
                continue;
            }
            if !meta.is_file() || path.extension().and_then(|x| x.to_str()) != Some("md") {
                continue;
            }
            if seen_md >= MAX_FILES || chunks.len() >= MAX_CHUNKS {
                scope.truncated = true;
                break 'walk;
            }
            seen_md += 1;
            if meta.len() > MAX_FILE_BYTES {
                scope.not_read.over_cap += 1;
                scope.not_read.total += 1;
                continue;
            }
            let bytes = match fs::read(&path) {
                Ok(b) => b,
                Err(_) => {
                    scope.not_read.unreadable += 1;
                    scope.not_read.total += 1;
                    continue;
                }
            };
            let text = match String::from_utf8(bytes) {
                Ok(t) => t,
                Err(_) => {
                    scope.not_read.not_utf8 += 1;
                    scope.not_read.total += 1;
                    continue;
                }
            };
            let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
            scope.files_read += 1;
            let rel = rel_path(&root.path, &path);

            // The room whose directory holds this file.
            let room = room_dirs
                .iter()
                .find(|(prefix, _)| rel.starts_with(prefix.as_str()))
                .map(|(_, n)| n.clone());

            // The crystals this file holds.
            let container = if root.palace.is_some() {
                (!rel.contains('/') && palace::is_marker(&name))
                    || room_dirs
                        .iter()
                        .any(|(prefix, _)| rel == format!("{prefix}CLAUDE.md"))
            } else {
                !rel.contains('/') && palace::is_marker(&name)
            };
            let scanned = root.palace.is_none()
                || rel.starts_with(&scan_prefix)
                || rel.starts_with("garden/");
            let mut found: Vec<Crystal> = Vec::new();
            if container {
                let (c, n) = format2::read_lines(text, &rel);
                found.extend(c);
                scope.legacy_lines_not_counted += n;
            }
            if scanned {
                if let Some(c) = format2::read_file_crystal(text, &rel) {
                    found.push(c);
                }
            }
            let mut mine: Vec<usize> = Vec::new();
            for mut c in found {
                c.room = room.clone();
                mine.push(crystals.len());
                crystals.push(c);
            }

            let file_ix = files.len();
            let (fm, malformed) = chunk_file(text, file_ix, &mine, &crystals, qtokens, &mut chunks);
            if malformed {
                scope.malformed_frontmatter += 1;
            }
            files.push(FileMeta {
                path: rel,
                room,
                frontmatter: fm,
            });
        }
        // Visit subdirectories in name order: the stack pops from the end.
        subdirs.reverse();
        stack.extend(subdirs);
    }

    scope.chunks = chunks.len();
    scope.crystals_read = crystals.len();
    Corpus {
        files,
        chunks,
        crystals,
        scope,
    }
}

/// Count the query tokens in a chunk's text.
fn counted(text: &str, qtokens: &[String]) -> (u32, Vec<u32>) {
    let toks = tokenize(text);
    let mut tf = vec![0u32; qtokens.len()];
    for t in &toks {
        if let Some(ix) = qtokens.iter().position(|q| q == t) {
            tf[ix] += 1;
        }
    }
    (toks.len() as u32, tf)
}

/// A leading `---` opens frontmatter only when the next line is a `key:` line (or the fence
/// closes at once). A horizontal rule above prose is not frontmatter, so its text stays searchable.
fn opens_like_frontmatter(lines: &[&str]) -> bool {
    match lines.get(1) {
        None => false,
        Some(l) if l.trim_end() == "---" => true,
        Some(l) => {
            let Some(ix) = l.find(':') else { return false };
            let key = &l[..ix];
            !key.is_empty()
                && !key.starts_with(char::is_whitespace)
                && key
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == ' ')
        }
    }
}

/// The line of a paragraph to show: the first one that holds a query word, else the first.
fn shown_line<'a>(lines: &[&'a str], qtokens: &[String]) -> &'a str {
    lines
        .iter()
        .find(|l| tokenize(l).iter().any(|t| qtokens.contains(t)))
        .or_else(|| lines.first())
        .map(|l| l.trim())
        .unwrap_or("")
}

/// Chunk one file. Returns its frontmatter pass-through and whether the frontmatter was
/// malformed (an opening fence that never closes: the file is then read as plain text).
fn chunk_file(
    text: &str,
    file: usize,
    mine: &[usize],
    crystals: &[Crystal],
    qtokens: &[String],
    chunks: &mut Vec<Chunk>,
) -> (BTreeMap<String, String>, bool) {
    let lines: Vec<&str> = text.lines().collect();
    let mut fm: BTreeMap<String, String> = BTreeMap::new();
    let mut fm_end = 0usize; // 1-based line of the closing fence; 0 when there is none
    let mut malformed = false;
    if lines.first().is_some_and(|l| l.trim_end() == "---") && opens_like_frontmatter(&lines) {
        match lines
            .iter()
            .skip(1)
            .position(|l| l.trim_end() == "---")
            .map(|p| p + 2)
        {
            Some(end) => fm_end = end,
            None => malformed = true,
        }
    }

    // A file crystal is the one crystal that starts on line 1 of a file with frontmatter.
    let file_crystal: Option<usize> = if fm_end > 0 {
        mine.iter().copied().find(|&ix| crystals[ix].line == 1)
    } else {
        None
    };

    if fm_end > 0 {
        let mut texts: Vec<String> = Vec::new();
        // Whether the last top-level key holds searchable text, so its indented lines and its
        // `- item` list lines (an Obsidian tag list) are text too.
        let mut in_text_key = false;
        for raw in &lines[1..fm_end - 1] {
            if raw.starts_with(char::is_whitespace) || raw.trim_start().starts_with("- ") {
                if in_text_key {
                    let item = raw.trim().trim_start_matches("- ").trim();
                    let item = item.trim_matches(|c| c == '"' || c == '\'');
                    if !item.is_empty() {
                        texts.push(item.to_string());
                    }
                }
                continue; // top-level keys only below
            }
            in_text_key = false;
            let Some(ix) = raw.find(':') else { continue };
            let key = raw[..ix].trim().to_lowercase();
            let val = raw[ix + 1..]
                .trim()
                .trim_matches(|c| c == '"' || c == '\'')
                .to_string();
            if key.is_empty() {
                continue;
            }
            if FM_TEXT_KEYS.contains(&key.as_str()) {
                in_text_key = true;
                // A block scalar (`>` or `|`) has its text on the lines below.
                if !val.is_empty() && !val.starts_with(['>', '|']) {
                    texts.push(val.clone());
                }
            }
            if val.is_empty() || val.starts_with(['>', '|']) {
                continue;
            }
            if fm.len() < MAX_FM_KEYS {
                fm.entry(key)
                    .or_insert_with(|| val.chars().take(MAX_FM_VALUE).collect());
            }
        }
        if !texts.is_empty() {
            let joined = texts.join(" ");
            let (len, tf) = counted(&joined, qtokens);
            chunks.push(Chunk {
                file,
                line: 1,
                end_line: fm_end,
                kind: if file_crystal.is_some() {
                    Kind::Crystal
                } else {
                    Kind::Document
                },
                crystal: file_crystal,
                snippet: joined.chars().take(160).collect(),
                len,
                tf,
            });
        }
    }

    // Crystal lines in a container file: one chunk each, name and fact.
    let mut covered = vec![false; lines.len() + 2];
    for &ix in mine {
        if Some(ix) == file_crystal {
            continue;
        }
        let c = &crystals[ix];
        let body = format!("{} {}", c.name.as_deref().unwrap_or(""), c.fact);
        let (len, tf) = counted(&body, qtokens);
        chunks.push(Chunk {
            file,
            line: c.line,
            end_line: c.end_line,
            kind: Kind::Crystal,
            crystal: Some(ix),
            snippet: String::new(),
            len,
            tf,
        });
        let hi = c.end_line.min(lines.len());
        if c.line <= hi {
            covered[c.line..=hi].fill(true);
        }
    }

    // Paragraphs: runs of non-blank lines outside the crystal ranges, at most 40 lines each.
    let mut start: Option<usize> = None; // 1-based first line of the open paragraph
    let flush = |from: usize, to: usize, chunks: &mut Vec<Chunk>| {
        let body = lines[from - 1..to].join(" ");
        let first = lines[from - 1].trim();
        let shown = shown_line(&lines[from - 1..to], qtokens);
        let note = first
            .strip_prefix("- ")
            .or_else(|| first.strip_prefix("* "))
            .is_some_and(|r| r.trim_start().starts_with("**"));
        let (len, tf) = counted(&body, qtokens);
        chunks.push(Chunk {
            file,
            line: from,
            end_line: to,
            kind: if file_crystal.is_some() {
                Kind::Crystal
            } else if note {
                Kind::Note
            } else {
                Kind::Document
            },
            crystal: file_crystal,
            snippet: shown.chars().take(160).collect(),
            len,
            tf,
        });
    };
    for n in (fm_end + 1)..=lines.len() {
        let blank = lines[n - 1].trim().is_empty() || covered[n];
        match (start, blank) {
            (None, false) => start = Some(n),
            (Some(s), true) => {
                flush(s, n - 1, chunks);
                start = None;
            }
            (Some(s), false) if n - s + 1 > MAX_CHUNK_LINES => {
                flush(s, n - 1, chunks);
                start = Some(n);
            }
            _ => {}
        }
    }
    if let Some(s) = start {
        flush(s, lines.len(), chunks);
    }
    (fm, malformed)
}

// ── scoring and state ────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum State {
    Current,
    None,
    Due,
    Superseded,
    Retired,
}

impl State {
    fn as_str(self) -> &'static str {
        match self {
            State::Current => "current",
            State::None => "none",
            State::Due => "due",
            State::Superseded => "superseded",
            State::Retired => "retired",
        }
    }

    /// Current and none rank together, then due, then superseded, then retired.
    fn group(self) -> u8 {
        match self {
            State::Current | State::None => 0,
            State::Due => 1,
            State::Superseded => 2,
            State::Retired => 3,
        }
    }
}

/// First match wins: retired, superseded (resolved or not), due (a real `valid_until` date that
/// `as_of` has reached), else current. A label never makes a crystal due.
fn state_of(c: &Crystal, as_of: &str) -> State {
    if c.is_retired() {
        State::Retired
    } else if c.superseded_by.is_some() {
        State::Superseded
    } else if c.is_scheduled() && c.valid_until_text().is_some_and(|d| d <= as_of) {
        State::Due
    } else {
        State::Current
    }
}

fn norm_name(s: &str) -> String {
    s.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Serialize)]
struct HitOut {
    rank: usize,
    root: String,
    path: String,
    line: usize,
    end_line: usize,
    cite: String,
    kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    state: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    room: Option<String>,
    valid_until: Option<String>,
    valid_until_kind: Option<&'static str>,
    superseded_by: Option<String>,
    superseded_by_found: Option<bool>,
    same_name_unresolved: Option<usize>,
    frontmatter: BTreeMap<String, String>,
    score: f64,
    text: String,
}

#[derive(Serialize)]
struct RecallOut {
    query: String,
    query_tokens: Vec<String>,
    unmatched_tokens: Vec<String>,
    tokenizer: &'static str,
    scoring: &'static str,
    as_of: String,
    matching_chunks: usize,
    scope: Scope,
    results: Vec<HitOut>,
}

fn cite(path: &str, line: usize, end_line: usize) -> String {
    if end_line > line {
        format!("{path}:{line}-{end_line}")
    } else {
        format!("{path}:{line}")
    }
}

fn search(corpus: &Corpus, qtokens: &[String], args: &Args, as_of: &str, tag: &str) -> RecallOut {
    let n = corpus.chunks.len() as f64;
    let total_len: f64 = corpus.chunks.iter().map(|c| c.len as f64).sum();
    let avgdl = if n > 0.0 { total_len / n } else { 0.0 };
    let mut df = vec![0usize; qtokens.len()];
    for c in &corpus.chunks {
        for (i, &t) in c.tf.iter().enumerate() {
            if t > 0 {
                df[i] += 1;
            }
        }
    }
    let idf: Vec<f64> = df
        .iter()
        .map(|&d| (1.0 + (n - d as f64 + 0.5) / (d as f64 + 0.5)).ln())
        .collect();

    // How many unresolved crystals share a normalised name, per name.
    let mut same_name: HashMap<String, usize> = HashMap::new();
    for c in &corpus.crystals {
        if c.retired.is_none() && c.superseded_by.is_none() {
            if let Some(name) = c.name.as_deref() {
                *same_name.entry(norm_name(name)).or_default() += 1;
            }
        }
    }

    struct Scored<'a> {
        chunk: &'a Chunk,
        state: State,
        centi: i64,
    }
    let mut scored: Vec<Scored> = Vec::new();
    for c in &corpus.chunks {
        let mut score = 0.0;
        for (i, &tf) in c.tf.iter().enumerate() {
            if tf == 0 {
                continue;
            }
            let tf = tf as f64;
            let norm = 1.0 - B + B * (c.len as f64 / avgdl.max(1e-9));
            score += idf[i] * tf * (K1 + 1.0) / (tf + K1 * norm);
        }
        if score <= 0.0 {
            continue;
        }
        if let Some(room) = &args.room {
            if corpus.files[c.file].room.as_deref() != Some(room.as_str()) {
                continue;
            }
        }
        let state = match c.crystal {
            Some(ix) => state_of(&corpus.crystals[ix], as_of),
            None => State::None,
        };
        scored.push(Scored {
            chunk: c,
            state,
            centi: (score * 100.0).round() as i64,
        });
    }
    let matching_chunks = scored.len();

    let recency = |s: &Scored| -> Option<&str> {
        let c = &corpus.crystals[s.chunk.crystal?];
        c.occurred.as_deref().or(c.mentioned.as_deref())
    };
    scored.sort_by(|a, b| {
        a.state
            .group()
            .cmp(&b.state.group())
            .then(b.centi.cmp(&a.centi))
            // Newest belief first; no date last.
            .then_with(|| match (recency(a), recency(b)) {
                (Some(x), Some(y)) => y.cmp(x),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            })
            .then_with(|| {
                corpus.files[a.chunk.file]
                    .path
                    .cmp(&corpus.files[b.chunk.file].path)
            })
            .then(a.chunk.line.cmp(&b.chunk.line))
    });

    let results: Vec<HitOut> = scored
        .iter()
        .take(args.limit)
        .enumerate()
        .map(|(i, s)| {
            let file = &corpus.files[s.chunk.file];
            let crystal = s.chunk.crystal.map(|ix| &corpus.crystals[ix]);
            let superseded_by = crystal.and_then(|c| c.superseded_by.clone());
            let found = crystal.zip(superseded_by.as_deref()).map(|(me, target)| {
                let want = norm_name(target);
                let hit = |other: &&Crystal| {
                    !std::ptr::eq(*other, me)
                        && other.name.as_deref().map(norm_name).as_deref() == Some(want.as_str())
                };
                // The same file, then the same room, then the workfolder.
                corpus
                    .crystals
                    .iter()
                    .filter(hit)
                    .any(|o| o.file == me.file)
                    || (me.room.is_some()
                        && corpus
                            .crystals
                            .iter()
                            .filter(hit)
                            .any(|o| o.room == me.room))
                    || corpus.crystals.iter().any(|o| hit(&o))
            });
            let (name, text) = match crystal {
                Some(c) if s.chunk.kind == Kind::Crystal && s.chunk.line == c.line => (
                    c.name.clone(),
                    format!(
                        "{} {}{}",
                        c.tier.glyph(),
                        c.name
                            .as_deref()
                            .map(|n| format!("{n}: "))
                            .unwrap_or_default(),
                        c.fact
                    ),
                ),
                Some(c) => (c.name.clone(), s.chunk.snippet.clone()),
                None => (None, s.chunk.snippet.clone()),
            };
            HitOut {
                rank: i + 1,
                root: tag.to_string(),
                path: file.path.clone(),
                line: s.chunk.line,
                end_line: s.chunk.end_line,
                cite: cite(&file.path, s.chunk.line, s.chunk.end_line),
                kind: s.chunk.kind.as_str(),
                name,
                state: s.state.as_str(),
                room: file.room.clone(),
                valid_until: crystal.and_then(|c| c.valid_until_text()).map(String::from),
                valid_until_kind: crystal.and_then(|c| {
                    c.valid_until.as_ref().map(|_| {
                        if c.is_scheduled() {
                            "scheduled"
                        } else {
                            "unscheduled"
                        }
                    })
                }),
                superseded_by,
                superseded_by_found: found,
                same_name_unresolved: crystal
                    .and_then(|c| c.name.as_deref())
                    .and_then(|nm| same_name.get(&norm_name(nm)).copied()),
                frontmatter: file.frontmatter.clone(),
                score: s.centi as f64 / 100.0,
                text,
            }
        })
        .collect();

    let unmatched: Vec<String> = qtokens
        .iter()
        .zip(&df)
        .filter(|(_, &d)| d == 0)
        .map(|(t, _)| t.clone())
        .collect();

    RecallOut {
        query: args.query.clone(),
        query_tokens: qtokens.to_vec(),
        unmatched_tokens: unmatched,
        tokenizer: TOKENIZER,
        scoring: SCORING,
        as_of: as_of.to_string(),
        matching_chunks,
        scope: Scope::default(),
        results,
    }
}

// ── output ───────────────────────────────────────────────────────────────

fn scope_line(s: &Scope) -> String {
    let mut parts = vec![
        format!("{} files read", s.files_read),
        format!("{} chunks", s.chunks),
        format!("{} crystals read", s.crystals_read),
    ];
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
        parts.push(format!(
            "{} not read ({})",
            s.not_read.total,
            why.join(", ")
        ));
    }
    if s.skipped_dirs > 0 {
        parts.push(format!("{} directories skipped by rule", s.skipped_dirs));
    }
    if s.skipped_symlinks > 0 {
        parts.push(format!("{} symlinks skipped", s.skipped_symlinks));
    }
    if s.malformed_frontmatter > 0 {
        parts.push(format!("{} malformed frontmatter", s.malformed_frontmatter));
    }
    if s.legacy_lines_not_counted > 0 {
        parts.push(format!(
            "{} legacy lines not counted as crystals (bullets without a tier glyph)",
            s.legacy_lines_not_counted
        ));
    }
    let mut line = format!("scope: {}", parts.join(", "));
    if s.truncated {
        line.push_str(&format!(
            "\nthe walk stopped at {MAX_FILES} files; later files were not read"
        ));
    }
    line
}

/// A folder's text may hold terminal control sequences; they must not reach the terminal.
fn plain(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}

fn print_text(out: &RecallOut) {
    // Every line below goes through `plain`, so no folder text reaches the terminal raw.
    macro_rules! println {
        () => {
            std::println!()
        };
        ($($a:tt)*) => {
            std::println!("{}", plain(&format!($($a)*)))
        };
    }
    println!(
        "recall: {} of {} matching chunks (tokenizer {}, scoring {}, as of {})",
        out.results.len(),
        out.matching_chunks,
        out.tokenizer,
        out.scoring,
        out.as_of
    );
    println!("query tokens: {}", out.query_tokens.join(", "));
    if !out.unmatched_tokens.is_empty() {
        println!(
            "found in no file: {} (no stemming, no accent folding)",
            out.unmatched_tokens.join(", ")
        );
    }
    println!();
    if out.results.is_empty() {
        println!("no matches");
    }
    for h in &out.results {
        println!(
            "{}. [{}] {}  {}  score {:.2}",
            h.rank,
            h.root,
            h.cite,
            if h.kind == "crystal" {
                h.state.to_string()
            } else {
                format!("{} ({})", h.state, h.kind)
            },
            h.score
        );
        if !h.text.is_empty() {
            println!("   {}", h.text);
        }
        if let Some(v) = &h.valid_until {
            println!(
                "   valid_until: {v}{}",
                if h.valid_until_kind == Some("unscheduled") {
                    " (a label, not a date)"
                } else {
                    ""
                }
            );
        }
        if let Some(s) = &h.superseded_by {
            println!(
                "   superseded_by: {s}{}",
                if h.superseded_by_found == Some(false) {
                    " (no crystal by that name)"
                } else {
                    ""
                }
            );
        }
        if let Some(n) = h.same_name_unresolved.filter(|&n| n > 1) {
            println!("   same_name_unresolved: {n}");
        }
        for (k, v) in &h.frontmatter {
            println!("   {k}: {v}");
        }
        println!("   cite: {}", h.cite);
    }
    println!();
    println!("{}", scope_line(&out.scope));
}

// ── entry ────────────────────────────────────────────────────────────────

pub fn run(args: Args, json: bool) -> Result<(), Error> {
    let qtokens = query_tokens(&args.query);
    if qtokens.is_empty() {
        return Err(Error::bad_input(
            "the query has no searchable words (letters or digits)".to_string(),
        ));
    }
    let as_of = match &args.as_of {
        Some(d) if format2::real_date(d) => d.clone(),
        Some(d) => {
            return Err(Error::bad_input(format!(
                "--as-of must be a real date written YYYY-MM-DD, got {d}"
            )))
        }
        None => tokens::epoch_to_iso(tokens::epoch_now())[..10].to_string(),
    };
    let root = resolve_root(
        args.root.clone(),
        args.workfolder.clone(),
        std::env::var_os("LOCI_WORKFOLDER"),
        std::env::var_os("LOCI_PALACE"),
        std::env::current_dir().ok(),
    )?;
    if let Some(room) = &args.room {
        let known = root
            .palace
            .as_ref()
            .is_some_and(|p| palace::list_rooms(p).iter().any(|r| r.name == *room));
        if !known {
            return Err(Error::not_found(format!("room not found: {room}")));
        }
    }
    let corpus = build_corpus(&root, &qtokens);
    let tag = root_tag(&root.path);
    let mut out = search(&corpus, &qtokens, &args, &as_of, &tag);
    out.scope = corpus.scope;
    if json {
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        print_text(&out);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_are_maximal_alphanumeric_runs_lowercased() {
        assert_eq!(tokenize("Hello, World!"), vec!["hello", "world"]);
        assert_eq!(tokenize("ÁRVÍZTŰRŐ café"), vec!["árvíztűrő", "café"]);
        assert!(tokenize("!!! ???").is_empty());
    }

    #[test]
    fn a_compound_yields_itself_and_its_parts() {
        let t = tokenize("well-known 2.5.0");
        for want in ["well-known", "well", "known", "2.5.0", "2", "5", "0"] {
            assert!(t.contains(&want.to_string()), "{want} missing from {t:?}");
        }
    }

    #[test]
    fn the_query_is_deduplicated_in_order() {
        assert_eq!(query_tokens("b a b a"), vec!["b", "a"]);
    }

    #[test]
    fn an_explicit_root_wins_and_a_missing_one_is_not_found() {
        let dir = std::env::temp_dir();
        let r = resolve_root(Some(dir.clone()), None, None, None, None).unwrap();
        assert_eq!(r.path, dir);
        let gone = dir.join("loci_recall_unit_no_such_dir");
        let e = resolve_root(Some(gone), None, None, None, None)
            .err()
            .unwrap();
        assert!(matches!(e.kind(), crate::ErrKind::NotFound), "{e:?}");
    }
}
