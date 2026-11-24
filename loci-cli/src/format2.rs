//! Reference reader for loci format 2 (`templates/format-2.md`).
//!
//! The spec is the contract and this file is the smallest reader that follows it. `status` and
//! `crystals` count through it; the conformance tests at the bottom read the same fixture
//! workfolders under `tests/fixtures/format2/` and compare them with `expected.json`, which is
//! written by hand from the spec and never generated from this code. The spec defines what is
//! read; how it is stored here is this file's: `date:` and `label:` prefixes on `valid_until`,
//! and `struck` for a retired line with no date. The accessors below hide them from output.
//!
//! Layout detection is not repeated here: `palace::validate` already follows the spec's table.
//!
//! Not evaluated, on purpose: `superseded_by` name resolution and the `pinned_until` last-day
//! rule are stored, and a checker (rc.2) decides them. `palace::count_md_files`, which feeds the
//! legacy file counts, does not skip `cron` as the spec does; the legacy keys keep that number.

use crate::palace::{self, Layout};
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Confirmed,
    Contextual,
    Exploratory,
}

impl Tier {
    pub fn glyph(self) -> char {
        match self {
            Tier::Confirmed => '◆',
            Tier::Contextual => '◈',
            Tier::Exploratory => '◇',
        }
    }

    fn from_glyph(c: char) -> Option<Tier> {
        match c {
            '◆' => Some(Tier::Confirmed),
            '◈' => Some(Tier::Contextual),
            '◇' => Some(Tier::Exploratory),
            _ => None,
        }
    }

    /// Frontmatter `tier`: the glyph or a tier name in any case. Aliases read without a warning.
    fn from_word(s: &str) -> Option<Tier> {
        match s.trim().to_lowercase().as_str() {
            "◆" | "confirmed" => Some(Tier::Confirmed),
            "◈" | "contextual" | "working" => Some(Tier::Contextual),
            "◇" | "exploratory" | "provisional" => Some(Tier::Exploratory),
            _ => None,
        }
    }
}

/// Counts over crystals. `live` is every crystal that is not retired, superseded ones included
/// (a superseded crystal stays in place); the three tiers add up to it. `superseded`,
/// `scheduled` and `unscheduled` count within the live ones.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Tally {
    pub live: usize,
    pub confirmed: usize,
    pub contextual: usize,
    pub exploratory: usize,
    pub retired: usize,
    pub superseded: usize,
    pub scheduled: usize,
    pub unscheduled: usize,
}

impl Tally {
    pub fn of<'a>(crystals: impl IntoIterator<Item = &'a Crystal>) -> Tally {
        let mut t = Tally::default();
        for c in crystals {
            if c.is_retired() {
                t.retired += 1;
                continue;
            }
            t.live += 1;
            match c.tier {
                Tier::Confirmed => t.confirmed += 1,
                Tier::Contextual => t.contextual += 1,
                Tier::Exploratory => t.exploratory += 1,
            }
            if c.superseded_by.is_some() {
                t.superseded += 1;
            }
            if c.is_scheduled() {
                t.scheduled += 1;
            } else if c.valid_until.is_some() {
                t.unscheduled += 1;
            }
        }
        t
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Crystal {
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub fact: String,
    pub tier: Tier,
    /// `date:YYYY-MM-DD` (scheduled) or `label:<text>` (unscheduled).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurred: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mentioned: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub pinned: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned_until: Option<String>,
    /// The retired date, or `struck` for a struck-through line with no `retired:` key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retired: Option<String>,
    /// The room it was read in, if any (a root or `garden/` crystal has none). Not part of the
    /// conformance oracle.
    #[serde(skip)]
    pub room: Option<String>,
}

impl Crystal {
    fn new(file: &str, name: Option<String>, fact: String, tier: Tier) -> Crystal {
        Crystal {
            file: file.to_string(),
            name,
            fact,
            tier,
            valid_until: None,
            occurred: None,
            mentioned: None,
            superseded_by: None,
            pinned: false,
            pinned_until: None,
            retired: None,
            room: None,
        }
    }

    /// `valid_until` as the person wrote it.
    pub fn valid_until_text(&self) -> Option<&str> {
        let v = self.valid_until.as_deref()?;
        v.strip_prefix("date:").or_else(|| v.strip_prefix("label:"))
    }

    /// A real date: a checker can schedule it. Anything else is an unscheduled label.
    pub fn is_scheduled(&self) -> bool {
        self.valid_until
            .as_deref()
            .is_some_and(|v| v.starts_with("date:"))
    }

    pub fn is_retired(&self) -> bool {
        self.retired.is_some()
    }

    /// The `retired:` date, if the crystal carries one.
    pub fn retired_on(&self) -> Option<&str> {
        self.retired.as_deref().filter(|r| *r != "struck")
    }

    fn apply_key(&mut self, key: &str, val: &str) {
        let val = val.trim();
        match key {
            "valid_until" => {
                if !val.is_empty() {
                    self.valid_until = Some(classify(val));
                }
            }
            "occurred" => {
                if real_date(val) {
                    self.occurred = Some(val.to_string());
                }
            }
            "mentioned" => {
                if real_date(val) {
                    self.mentioned = Some(val.to_string());
                }
            }
            "superseded_by" => {
                if !val.is_empty() {
                    self.superseded_by = Some(val.to_string());
                }
            }
            "pinned" => {
                if val == "true" {
                    self.pinned = true;
                }
            }
            "pinned_until" => {
                if real_date(val) {
                    self.pinned_until = Some(val.to_string());
                }
            }
            "retired" => {
                // The date is the first word; the rest is a reason.
                let first = val.split_whitespace().next().unwrap_or("");
                if real_date(first) {
                    self.retired = Some(first.to_string());
                }
            }
            _ => {} // unknown and reserved keys are ignored
        }
    }
}

/// A real calendar day written `YYYY-MM-DD`. `2026-02-30` is not one.
pub fn real_date(s: &str) -> bool {
    if s.len() != 10 {
        return false;
    }
    let shaped = s.chars().enumerate().all(|(i, c)| {
        if i == 4 || i == 7 {
            c == '-'
        } else {
            c.is_ascii_digit()
        }
    });
    if !shaped {
        return false;
    }
    let (y, m, d): (u32, u32, u32) = (
        s[0..4].parse().unwrap(),
        s[5..7].parse().unwrap(),
        s[8..10].parse().unwrap(),
    );
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let days = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=days).contains(&d)
}

fn classify(val: &str) -> String {
    if real_date(val) {
        format!("date:{val}")
    } else {
        format!("label:{val}")
    }
}

/// `key: value` with a lowercase key of letters and underscores. Anything else is not a key line.
fn key_line(line: &str) -> Option<(&str, &str)> {
    let ix = line.find(": ")?;
    let key = &line[..ix];
    if key.is_empty() || !key.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
        return None;
    }
    Some((key, &line[ix + 2..]))
}

/// One crystal line: a tier glyph as the first character, a space, then name and fact. A whole
/// line struck through (`~~...~~`) is a retired crystal. The legacy suffixes ` - valid_until: X`
/// and ` · pinned` are removed before the name is split, so a nameless fact keeps its colons.
fn crystal_line(line: &str, file: &str) -> Option<Crystal> {
    let line = line.trim_end();
    let (struck, body) = if line.len() > 4 && line.starts_with("~~") && line.ends_with("~~") {
        (true, &line[2..line.len() - 2])
    } else {
        (false, line)
    };
    let mut chars = body.chars();
    let tier = Tier::from_glyph(chars.next()?)?;
    let mut rest = chars.as_str().strip_prefix(' ')?.trim_end().to_string();
    let mut pinned = false;
    let mut legacy_valid = None;
    // Either suffix may come last, so strip until neither is left.
    loop {
        if let Some(r) = rest.strip_suffix(" · pinned") {
            pinned = true;
            rest = r.trim_end().to_string();
        } else if let Some(ix) = rest.rfind(" - valid_until: ") {
            let value = rest[ix + " - valid_until: ".len()..].trim();
            if value.contains(" · pinned") || legacy_valid.is_some() {
                break;
            }
            legacy_valid = Some(value.to_string());
            rest.truncate(ix);
        } else {
            break;
        }
    }
    let (name, fact) = match rest.find(": ") {
        Some(ix) => (
            Some(rest[..ix].to_string()),
            rest[ix + 2..].trim().to_string(),
        ),
        None => (None, rest.trim().to_string()),
    };
    let mut c = Crystal::new(file, name, fact, tier);
    c.pinned = pinned;
    if let Some(v) = legacy_valid {
        c.apply_key("valid_until", &v);
    }
    if struck {
        c.retired = Some("struck".to_string());
    }
    Some(c)
}

/// Crystal lines in a container file (`WORKFOLDER.md`, `PALACE.md` or a `CLAUDE.md`). Returns the crystals and the
/// number of glyph-less bullets shaped like a legacy crystal (`- **Name** = fact`). Fenced blocks hold examples; an unclosed fence runs to
/// the end of the file. The key block ends at the first line that is not a key line.
pub fn read_lines(text: &str, file: &str) -> (Vec<Crystal>, usize) {
    let mut out: Vec<Crystal> = Vec::new();
    let mut notes = 0;
    let mut fence: Option<&str> = None;
    let mut open = false; // the last crystal still accepts key lines
    for line in text.lines() {
        let t = line.trim_start();
        if let Some(f) = fence {
            if t.starts_with(f) {
                fence = None;
            }
            continue;
        }
        if t.starts_with("```") {
            fence = Some("```");
            open = false;
            continue;
        }
        if t.starts_with("~~~") {
            fence = Some("~~~");
            open = false;
            continue;
        }
        if let Some(c) = crystal_line(line, file) {
            out.push(c);
            open = true;
            continue;
        }
        if open {
            if let Some((k, v)) = key_line(line) {
                out.last_mut().unwrap().apply_key(k, v);
                continue;
            }
        }
        open = false;
        // Only a bullet shaped like a 1.x crystal (`- **Name** = fact`) is worth a count: it is
        // the one a person may want to move to the glyph form. Plain instruction bullets are not.
        if let Some(rest) = t.strip_prefix("- ").or_else(|| t.strip_prefix("* ")) {
            if rest.trim_start().starts_with("**") {
                notes += 1;
            }
        }
    }
    (out, notes)
}

/// A file crystal: `type: crystal` in the frontmatter (case-insensitive). Name is the first `#`
/// heading or the file stem; the fact is the first paragraph after the heading.
pub fn read_file_crystal(text: &str, file: &str) -> Option<Crystal> {
    let mut lines = text.lines();
    if lines.next()?.trim_end() != "---" {
        return None;
    }
    let mut fm: Vec<(String, String)> = Vec::new();
    let mut closed = false;
    for line in lines.by_ref() {
        if line.trim_end() == "---" {
            closed = true;
            break;
        }
        if let Some(ix) = line.find(':') {
            let key = line[..ix].trim().to_lowercase();
            let val = line[ix + 1..]
                .trim()
                .trim_matches(|c| c == '"' || c == '\'')
                .to_string();
            fm.push((key, val));
        }
    }
    if !closed {
        return None;
    }
    let get = |k: &str| fm.iter().find(|(key, _)| key == k).map(|(_, v)| v.as_str());
    if !get("type")?.eq_ignore_ascii_case("crystal") {
        return None;
    }
    let tier = get("tier")
        .and_then(Tier::from_word)
        .unwrap_or(Tier::Exploratory);

    let body: Vec<&str> = lines.collect();
    let heading = body.iter().position(|l| l.starts_with("# "));
    let name = match heading {
        Some(i) => body[i][2..].trim().to_string(),
        None => Path::new(file)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default(),
    };
    let mut paragraph: Vec<&str> = Vec::new();
    for l in body.iter().skip(heading.map_or(0, |i| i + 1)) {
        if l.trim().is_empty() {
            if paragraph.is_empty() {
                continue;
            }
            break;
        }
        paragraph.push(l.trim());
    }
    let mut c = Crystal::new(file, Some(name), paragraph.join(" "), tier);
    for (k, v) in &fm {
        c.apply_key(k, v);
    }
    Some(c)
}

fn skipped(name: &str) -> bool {
    name.starts_with('.') || name == "node_modules" || name == "target" || name == "cron"
}

/// A container file may hold many crystal lines; a file crystal needs only its frontmatter and
/// first paragraph. Neither is read past these caps, so a huge file cannot stall a count.
const CONTAINER_CAP: u64 = 8 * 1024 * 1024;
const FILE_CAP: u64 = 64 * 1024;

/// Read at most `cap` bytes of a regular file as text. A leading BOM is dropped and bytes that are
/// not UTF-8 read as U+FFFD. `None` when the path is not a regular file or cannot be opened.
fn read_capped(path: &Path, cap: u64) -> Option<String> {
    use std::io::Read;
    let f = fs::File::open(path).ok()?;
    if !f.metadata().ok()?.is_file() {
        return None;
    }
    let mut buf = Vec::new();
    f.take(cap).read_to_end(&mut buf).ok()?;
    let text = String::from_utf8_lossy(&buf);
    Some(text.strip_prefix('\u{feff}').unwrap_or(&text).to_string())
}

/// Regular `.md` files under `dir`. Symlinks to files and directories are followed, as room
/// discovery and the file count do; a directory already seen (by its real path) is not entered
/// again, so a link back up the tree is read once; a file reached by two paths counts once. FIFOs, sockets and devices are never opened.
fn md_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let mut seen = std::collections::HashSet::new();
    let mut found = Vec::new();
    md_walk(dir, &mut found, &mut seen);
    // A file reached by two paths (a link and its target) is one file: keep the first by name.
    found.sort();
    let mut real = std::collections::HashSet::new();
    for f in found {
        if real.insert(fs::canonicalize(&f).unwrap_or_else(|_| f.clone())) {
            out.push(f);
        }
    }
}

fn md_walk(
    dir: &Path,
    out: &mut Vec<std::path::PathBuf>,
    seen: &mut std::collections::HashSet<std::path::PathBuf>,
) {
    if let Ok(real) = fs::canonicalize(dir) {
        if !seen.insert(real) {
            return;
        }
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.filter_map(|e| e.ok()) {
        let name = e.file_name().to_string_lossy().to_string();
        let p = e.path();
        // `metadata` follows a symlink; a dangling link has none and is skipped.
        let Ok(meta) = fs::metadata(&p) else { continue };
        if meta.is_dir() {
            if !skipped(&name) {
                md_walk(&p, out, seen);
            }
        } else if meta.is_file() && p.extension().and_then(|x| x.to_str()) == Some("md") {
            out.push(p);
        }
    }
}

/// What a workfolder holds. `layout` and `rooms` are read by the conformance tests.
#[allow(dead_code)]
pub struct Reading {
    pub layout: &'static str,
    pub rooms: Vec<String>,
    pub crystals: Vec<Crystal>,
    /// `.md` files that are not crystals, other than the marker containers (`WORKFOLDER.md`,
    /// `PALACE.md`, `CLAUDE.md`).
    pub documents: Vec<String>,
    /// Glyph-less bullets shaped like a legacy crystal (`- **Name** = fact`): read as notes.
    pub notes: usize,
}

fn rel(root: &Path, p: &Path) -> String {
    p.strip_prefix(root)
        .unwrap_or(p)
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}

/// Read a workfolder as the spec says: crystal lines in the root `WORKFOLDER.md`, `PALACE.md` and `CLAUDE.md` and
/// in each room's `CLAUDE.md`; file crystals under the scan root and under `garden/` at the root.
#[cfg(test)]
pub fn read(root: &Path) -> Option<Reading> {
    Some(read_palace(&palace::validate(root)?))
}

pub fn read_palace(p: &palace::Palace) -> Reading {
    let root = p.root.as_path();
    let layout = match p.layout {
        Layout::PalaceSubdir => "palace-subdir",
        Layout::RoomsDir => "rooms-dir",
        Layout::RoomsAtRoot => "rooms-at-root",
    };
    let rooms = palace::list_rooms(p);
    let mut containers: Vec<std::path::PathBuf> =
        palace::MARKERS.iter().map(|m| root.join(m)).collect();
    containers.extend(rooms.iter().map(|r| r.path.join("CLAUDE.md")));

    let mut crystals = Vec::new();
    let mut notes = 0;
    for c in containers.iter() {
        let Some(text) = read_capped(c, CONTAINER_CAP) else {
            continue;
        };
        let (found, n) = read_lines(&text, &rel(root, c));
        crystals.extend(found);
        notes += n;
    }

    let mut files = Vec::new();
    md_files(&p.scan_root, &mut files);
    let garden = root.join("garden");
    if !garden.starts_with(&p.scan_root) {
        md_files(&garden, &mut files);
    }
    files.sort();
    files.dedup();
    let mut documents = Vec::new();
    for f in files {
        let name = f.file_name().unwrap().to_string_lossy().to_string();
        let text = read_capped(&f, FILE_CAP).unwrap_or_default();
        match read_file_crystal(&text, &rel(root, &f)) {
            Some(c) => crystals.push(c),
            None if palace::is_marker(&name) => {}
            None => documents.push(rel(root, &f)),
        }
    }
    // A crystal belongs to the room whose directory holds its file.
    for c in &mut crystals {
        c.room = rooms
            .iter()
            .find(|r| c.file.starts_with(&format!("{}/", rel(root, &r.path))))
            .map(|r| r.name.clone());
    }
    Reading {
        layout,
        rooms: rooms.into_iter().map(|r| r.name).collect(),
        crystals,
        documents,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use std::path::PathBuf;

    const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/format2");

    fn scratch(name: &str) -> PathBuf {
        let p =
            std::env::temp_dir().join(format!("loci_cli_format2_{}_{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    /// Copy a tree. A directory named `_palace.fixture` is written as `_palace`: the repo ignores
    /// `_palace/` so a real palace can never be committed, and the fixture must not weaken that.
    fn copy_tree(from: &Path, to: &Path) {
        fs::create_dir_all(to).unwrap();
        for e in fs::read_dir(from).unwrap().filter_map(|e| e.ok()) {
            let name = e.file_name().to_string_lossy().to_string();
            let name = if name == "_palace.fixture" {
                "_palace".to_string()
            } else {
                name
            };
            let dest = to.join(name);
            if e.path().is_dir() {
                copy_tree(&e.path(), &dest);
            } else {
                fs::copy(e.path(), dest).unwrap();
            }
        }
    }

    fn materialise(fixture: &str, tag: &str) -> PathBuf {
        let to = scratch(&format!("{fixture}_{tag}"));
        copy_tree(&Path::new(FIXTURES).join(fixture), &to);
        to
    }

    fn expected() -> Value {
        let text = fs::read_to_string(Path::new(FIXTURES).join("expected.json")).unwrap();
        serde_json::from_str(&text).unwrap()
    }

    fn sorted_strings(v: Vec<Value>) -> Vec<String> {
        let mut s: Vec<String> = v.iter().map(|x| x.to_string()).collect();
        s.sort();
        s
    }

    /// Compare a materialised workfolder with its expectation. Empty Vec means it conforms.
    fn diff(root: &Path, want: &Value) -> Vec<String> {
        let Some(got) = read(root) else {
            return vec!["not detected as a workfolder".to_string()];
        };
        compare(&got, want)
    }

    fn compare(got: &Reading, want: &Value) -> Vec<String> {
        let mut problems = Vec::new();
        if want["layout"] != json!(got.layout) {
            problems.push(format!(
                "layout: want {} got {}",
                want["layout"], got.layout
            ));
        }
        if want["rooms"] != json!(got.rooms) {
            problems.push(format!("rooms: want {} got {:?}", want["rooms"], got.rooms));
        }
        let mut docs = got.documents.clone();
        docs.sort();
        if want["documents"] != json!(docs) {
            problems.push(format!(
                "documents: want {} got {:?}",
                want["documents"], docs
            ));
        }
        if want["notes"] != json!(got.notes) {
            problems.push(format!("notes: want {} got {}", want["notes"], got.notes));
        }
        let want_c = sorted_strings(want["crystals"].as_array().unwrap().clone());
        let got_c = sorted_strings(
            got.crystals
                .iter()
                .map(|c| serde_json::to_value(c).unwrap())
                .collect(),
        );
        // Multiset: a crystal read twice is a difference, not a match.
        let mut left = got_c.clone();
        for w in &want_c {
            match left.iter().position(|g| g == w) {
                Some(i) => {
                    left.remove(i);
                }
                None => problems.push(format!("missing crystal: {w}")),
            }
        }
        for g in &left {
            problems.push(format!("unexpected or repeated crystal: {g}"));
        }
        problems
    }

    fn fixture_names() -> Vec<String> {
        let mut names: Vec<String> = expected().as_object().unwrap().keys().cloned().collect();
        names.sort();
        names
    }

    #[test]
    fn fixtures_conform_to_the_spec() {
        let want = expected();
        for name in fixture_names() {
            let root = materialise(&name, "real");
            let problems = diff(&root, &want[&name]);
            let _ = fs::remove_dir_all(&root);
            assert!(
                problems.is_empty(),
                "{name} does not conform:\n{}",
                problems.join("\n")
            );
        }
    }

    #[test]
    fn every_fixture_directory_has_an_expectation() {
        let want = expected();
        for e in fs::read_dir(FIXTURES).unwrap().filter_map(|e| e.ok()) {
            if e.path().is_dir() {
                let name = e.file_name().to_string_lossy().to_string();
                assert!(
                    want.get(&name).is_some(),
                    "fixture {name} has no entry in expected.json"
                );
            }
        }
    }

    /// Selftest the check: a fixture that has drifted from its expectation must fail. Each case
    /// breaks one real fixture in one way and the diff has to notice.
    #[test]
    fn a_deliberately_broken_fixture_fails() {
        let want = expected();
        let cases: &[(&str, &str, &str, &str)] = &[
            ("rooms-dir", "CLAUDE.md", "◆ Owner: Ada", "◇ Owner: Ada"),
            (
                "rooms-dir",
                "CLAUDE.md",
                "valid_until: 2026-12-31\noccurred",
                "occurred",
            ),
            (
                "rooms-dir",
                "CLAUDE.md",
                "valid_until: 2026-12-31\noccurred",
                "valid_until: sprint-end\noccurred",
            ),
            ("rooms-dir", "CLAUDE.md", "```\n◆ Example", "◆ Example"),
            ("rooms-dir", "CLAUDE.md", "◇ Quarter:", "◇ Quarter :"),
            (
                "rooms-dir",
                "rooms/work/CLAUDE.md",
                "retired: 2026-06-01",
                "retired: 2026-06-31",
            ),
            (
                "rooms-dir",
                "rooms/work/CLAUDE.md",
                "superseded_by: Framing v2\n",
                "",
            ),
            (
                "rooms-dir",
                "garden/first.md",
                "type: crystal",
                "type: note",
            ),
            (
                "rooms-dir",
                "garden/untiered.md",
                "type: crystal\n",
                "type: crystal\ntier: \"◆\"\n",
            ),
            (
                "rooms-dir",
                "rooms/work/shard.md",
                "type: shard",
                "type: crystal",
            ),
            (
                "palace-subdir",
                "PALACE.md",
                "◆ Owner: Bo",
                "◆ Owner: Bo\n◆ Extra: a stray crystal",
            ),
            (
                "rooms-at-root",
                "garden/pea.md",
                "tier: exploratory",
                "tier: confirmed",
            ),
        ];
        for (fixture, file, from, to) in cases {
            let root = materialise(fixture, "broken");
            let path = root.join(file);
            let text = fs::read_to_string(&path).unwrap();
            assert!(
                text.contains(from),
                "selftest case is stale: {fixture}/{file} lacks {from:?}"
            );
            fs::write(&path, text.replacen(from, to, 1)).unwrap();
            let problems = diff(&root, &want[fixture]);
            let _ = fs::remove_dir_all(&root);
            assert!(
                !problems.is_empty(),
                "breaking {fixture}/{file} ({from:?} -> {to:?}) went unnoticed"
            );
        }
        // A layout change is noticed too: the same rooms under `_palace/` read as another layout.
        let root = materialise("rooms-dir", "layout");
        fs::rename(root.join("rooms"), root.join("_palace")).unwrap();
        let problems = diff(&root, &want["rooms-dir"]);
        let _ = fs::remove_dir_all(&root);
        assert!(
            problems.iter().any(|p| p.starts_with("layout: want")),
            "a changed layout went unnoticed: {problems:?}"
        );
        // A crystal read twice is noticed.
        let root = materialise("rooms-at-root", "twice");
        let mut doubled = read(&root).unwrap();
        let _ = fs::remove_dir_all(&root);
        let again = doubled.crystals[0].clone();
        doubled.crystals.push(again);
        assert!(
            !compare(&doubled, &want["rooms-at-root"]).is_empty(),
            "a crystal read twice went unnoticed"
        );
    }

    /// The fixtures must keep exercising every rule the spec states, so a tidy-up cannot quietly
    /// shrink what the conformance test proves.
    #[test]
    fn fixtures_keep_covering_the_spec() {
        let mut all = String::new();
        fn gather(dir: &Path, out: &mut String) {
            for e in fs::read_dir(dir).unwrap().filter_map(|e| e.ok()) {
                if e.path().is_dir() {
                    gather(&e.path(), out);
                } else if e.path().extension().and_then(|x| x.to_str()) == Some("md") {
                    out.push_str(&fs::read_to_string(e.path()).unwrap());
                }
            }
        }
        gather(Path::new(FIXTURES), &mut all);
        for needle in [
            "◆ ",
            "◈ ",
            "◇ ",
            "tier: Working",
            "tier: provisional",
            "tier: sparkly",
            "tier: \"◆\"",
            "valid_until: 2026-12-31",
            "valid_until: sprint-end",
            "valid_until: 2026-Q3",
            "valid_until: 2026-02-30",
            "occurred:",
            "mentioned:",
            "superseded_by:",
            "pinned: true",
            "pinned_until:",
            "retired:",
            "~~◈",
            " - valid_until: ",
            " · pinned",
            "Valid_until:",
            "shard_hint:",
            "address:",
            "signed_by:",
            "type: shard",
            "type: Crystal",
            "```",
            "~~~",
            "- **",
            "→ Promoted",
        ] {
            assert!(all.contains(needle), "no fixture exercises {needle:?}");
        }
        let want = expected();
        let tiers: String = want.to_string();
        for t in ["confirmed", "contextual", "exploratory"] {
            assert!(
                tiers.contains(&format!("\"tier\":\"{t}\"")),
                "no expected crystal has tier {t}"
            );
        }
        for layout in ["palace-subdir", "rooms-dir", "rooms-at-root"] {
            assert!(want.get(layout).is_some(), "no fixture for layout {layout}");
        }
    }

    fn tree(name: &str, dirs_and_rooms: &[&str]) -> PathBuf {
        let root = scratch(name);
        fs::write(root.join("PALACE.md"), "# p\n").unwrap();
        for d in dirs_and_rooms {
            let p = root.join(d);
            fs::create_dir_all(&p).unwrap();
            if d.ends_with("/r") || *d == "r" {
                fs::write(p.join("CLAUDE.md"), "# r\n").unwrap();
            }
        }
        root
    }

    #[test]
    fn layout_precedence_follows_the_spec_table() {
        // 1: `_palace/` wins when it exists, even with no room in it and a rooms/ beside it.
        let root = tree("order1", &["_palace", "rooms/r", "r"]);
        assert_eq!(read(&root).unwrap().layout, "palace-subdir");
        let _ = fs::remove_dir_all(&root);
        // 2: `rooms/` wins when it holds a room, over a room at the root.
        let root = tree("order2", &["rooms/r", "r"]);
        assert_eq!(read(&root).unwrap().layout, "rooms-dir");
        let _ = fs::remove_dir_all(&root);
        // 3: `rooms/` with no room falls through to rooms at the root.
        let root = tree("order3", &["rooms", "r"]);
        assert_eq!(read(&root).unwrap().layout, "rooms-at-root");
        let _ = fs::remove_dir_all(&root);
        // Nothing to scan: not a workfolder.
        let root = tree("order4", &["rooms", "notes"]);
        assert!(read(&root).is_none());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn skipped_directories_are_not_scanned() {
        let root = tree(
            "skip",
            &[
                "rooms/r",
                "rooms/node_modules",
                "rooms/target",
                "rooms/cron",
                "rooms/.old",
            ],
        );
        for d in ["node_modules", "target", "cron", ".old"] {
            fs::write(
                root.join("rooms").join(d).join("x.md"),
                "---\ntype: crystal\n---\n# Hidden\n\nnever read\n",
            )
            .unwrap();
        }
        let r = read(&root).unwrap();
        let _ = fs::remove_dir_all(&root);
        assert!(
            r.crystals.is_empty(),
            "skipped directories were read: {:?}",
            r.crystals
        );
    }

    #[test]
    fn calendar_dates_are_real_or_labels() {
        for ok in ["2026-12-31", "2024-02-29", "2000-02-29", "2026-01-01"] {
            assert!(real_date(ok), "{ok} should be a date");
        }
        for bad in [
            "2026-02-30",
            "2026-02-29",
            "1900-02-29",
            "2026-13-01",
            "2026-00-10",
            "2026-1-01",
            "2026-Q3",
            "sprint-end",
            "2026-12-31 (review)",
            "",
        ] {
            assert!(!real_date(bad), "{bad:?} should not be a date");
        }
    }

    #[test]
    fn a_legacy_suffix_is_removed_before_the_name_is_split() {
        let c = crystal_line("◈ Standup at 10:30 daily - valid_until: 2026-12-31", "f.md").unwrap();
        assert_eq!(c.name, None);
        assert_eq!(c.fact, "Standup at 10:30 daily");
        assert_eq!(c.valid_until.as_deref(), Some("date:2026-12-31"));
        let c = crystal_line("◆ Style: bullet points · pinned", "f.md").unwrap();
        assert_eq!(c.name.as_deref(), Some("Style"));
        assert_eq!(c.fact, "bullet points");
        assert!(c.pinned);
    }

    #[test]
    fn a_glyph_must_be_the_first_character() {
        assert!(crystal_line(" ◆ Indented: no", "f.md").is_none());
        assert!(crystal_line("- ◆ Bulleted: no", "f.md").is_none());
        assert!(crystal_line("◆No space: no", "f.md").is_none());
        assert!(crystal_line("◆ Space: yes", "f.md").is_some());
    }

    #[test]
    fn an_unclosed_fence_runs_to_the_end_of_the_file() {
        let (c, _) = read_lines(
            "◆ Before: counted\n```\n◆ Inside: not\n◈ Also inside: not\n",
            "f.md",
        );
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].name.as_deref(), Some("Before"));
    }

    #[test]
    fn a_name_ends_at_the_first_colon_and_a_suffix_at_the_last_marker() {
        let c = crystal_line("◆ Ratio: 3: 1 at the gate", "f.md").unwrap();
        assert_eq!(c.name.as_deref(), Some("Ratio"));
        assert_eq!(c.fact, "3: 1 at the gate");
        let c = crystal_line(
            "◈ Note: write \" - valid_until: x\" later - valid_until: 2026-12-31",
            "f.md",
        )
        .unwrap();
        assert_eq!(c.fact, "write \" - valid_until: x\" later");
        assert_eq!(c.valid_until.as_deref(), Some("date:2026-12-31"));
    }

    #[test]
    fn both_legacy_suffixes_read_in_either_order() {
        for line in [
            "◆ Keep: fact · pinned - valid_until: 2026-12-31",
            "◆ Keep: fact - valid_until: 2026-12-31 · pinned",
        ] {
            let c = crystal_line(line, "f.md").unwrap();
            assert!(c.pinned, "{line}");
            assert_eq!(c.fact, "fact", "{line}");
            assert_eq!(c.valid_until.as_deref(), Some("date:2026-12-31"), "{line}");
        }
    }

    #[test]
    fn a_struck_line_is_retired_even_with_trailing_space() {
        let c = crystal_line("~~◈ Old: rhythm~~  ", "f.md").unwrap();
        assert_eq!(c.retired.as_deref(), Some("struck"));
        assert!(crystal_line("~~◈ Half: struck", "f.md").is_none());
    }

    #[test]
    fn keys_are_read_only_as_the_spec_says() {
        let (c, _) = read_lines(
            "◆ A: b\n  valid_until: 2026-12-31\n◆ C: d\noccurred: 2026-02-30\nmentioned: soon\n\
             pinned_until: later\nretired: someday\npinned: false\n◆ E: f\npinned: True\n",
            "f.md",
        );
        assert_eq!(c[0].valid_until, None, "an indented line is not a key");
        assert_eq!(c[1].occurred, None);
        assert_eq!(c[1].mentioned, None);
        assert_eq!(c[1].pinned_until, None);
        assert_eq!(c[1].retired, None);
        assert!(!c[1].pinned, "pinned is exactly true");
        assert!(!c[2].pinned, "True is not true");
    }

    #[test]
    fn a_fence_closes_only_on_its_own_marker() {
        let (c, _) = read_lines(
            "```\n~~~\n◆ In: no\n```\n◆ Out: yes\n~~~\n```\n◆ Still in: no\n~~~\n◆ Out2: yes\n",
            "f.md",
        );
        let names: Vec<_> = c.iter().filter_map(|c| c.name.as_deref()).collect();
        assert_eq!(names, ["Out", "Out2"]);
    }

    #[test]
    fn frontmatter_keys_and_type_match_in_any_case() {
        let c = read_file_crystal(
            "---\nType: Crystal\nTier: Working\nValid_Until: 2026-12-31\nPinned: true\n---\n# N\n\nfact\n",
            "n.md",
        )
        .unwrap();
        assert_eq!(c.tier as u8, Tier::Contextual as u8);
        assert_eq!(c.valid_until.as_deref(), Some("date:2026-12-31"));
        assert!(c.pinned);
    }

    #[cfg(unix)]
    #[test]
    fn a_linked_file_is_read_once_and_a_loop_ends() {
        let root = tree("link", &["rooms/r"]);
        fs::write(
            root.join("rooms/r/c.md"),
            "---\ntype: crystal\ntier: \"◆\"\n---\n# One\n\nonly once\n",
        )
        .unwrap();
        std::os::unix::fs::symlink(".", root.join("rooms/r/loop")).unwrap();
        std::os::unix::fs::symlink("c.md", root.join("rooms/r/alias.md")).unwrap();
        let r = read(&root).unwrap();
        let _ = fs::remove_dir_all(&root);
        assert_eq!(r.crystals.len(), 1, "{:?}", r.crystals);
    }

    fn tally(
        (live, c, x, e): (usize, usize, usize, usize),
        (retired, superseded, scheduled, unscheduled): (usize, usize, usize, usize),
    ) -> Tally {
        Tally {
            live,
            confirmed: c,
            contextual: x,
            exploratory: e,
            retired,
            superseded,
            scheduled,
            unscheduled,
        }
    }

    /// `status` counts tiered crystals the way the spec says. The numbers are tallied by hand from
    /// `expected.json`: live crystals by tier, then retired, superseded, scheduled, unscheduled.
    #[test]
    fn status_counts_tiered_crystals_per_the_spec() {
        let cases = [
            ("rooms-dir", tally((21, 5, 9, 7), (3, 2, 5, 4)), 1, 2),
            ("palace-subdir", tally((4, 1, 1, 2), (0, 0, 0, 1)), 0, 0),
            ("rooms-at-root", tally((6, 1, 3, 2), (0, 0, 2, 0)), 0, 2),
        ];
        for (name, want, notes, docs) in cases {
            let root = materialise(name, "status");
            let p = palace::validate(&root).unwrap();
            let out = crate::status_out(&p);
            let files = palace::count_md_files(&p.scan_root);
            let _ = fs::remove_dir_all(&root);
            assert_eq!(out.tiered, want, "{name}: tiered");
            assert_eq!(out.legacy_lines_not_counted, notes, "{name}: legacy lines");
            assert_eq!(out.documents, docs, "{name}: documents");
            // The old keys keep their old meaning: every `.md` file, the same number as before.
            assert_eq!(out.crystal_total, files, "{name}: crystal_total");
            assert_eq!(out.files_total, files, "{name}: files_total");
            for r in &out.rooms {
                assert_eq!(r.crystals, r.files, "{name}/{}: legacy room count", r.name);
            }
        }
    }

    #[test]
    fn status_attributes_crystals_to_their_room() {
        let root = materialise("rooms-dir", "rooms");
        let out = crate::status_out(&palace::validate(&root).unwrap());
        let _ = fs::remove_dir_all(&root);
        let work = out.rooms.iter().find(|r| r.name == "work").unwrap();
        assert_eq!(work.tiered, tally((9, 3, 4, 2), (2, 2, 3, 0)));
        let general = out.rooms.iter().find(|r| r.name == "general").unwrap();
        assert_eq!(general.tiered, Tally::default());
        // The rest (root CLAUDE.md, garden/) belongs to no room but is in the total.
        assert_eq!(out.tiered.live, 21);

        let root = materialise("palace-subdir", "rooms2");
        let out = crate::status_out(&palace::validate(&root).unwrap());
        let _ = fs::remove_dir_all(&root);
        let ideas = out.rooms.iter().find(|r| r.name == "ideas").unwrap();
        assert_eq!(ideas.tiered, tally((2, 0, 0, 2), (0, 0, 0, 0)));
    }

    #[test]
    fn a_retired_or_superseded_crystal_is_counted_apart() {
        let (c, _) = read_lines(
            "◆ Live: yes\n◈ Old: gone\nsuperseded_by: Live\n~~◇ Dead: bye~~\nretired: 2026-06-01\n",
            "f.md",
        );
        let t = Tally::of(&c);
        assert_eq!((t.live, t.retired, t.superseded), (2, 1, 1));
        assert_eq!(t.confirmed + t.contextual + t.exploratory, t.live);
    }

    #[test]
    fn output_hides_the_stored_encoding() {
        let (c, _) = read_lines(
            "◈ A: b\nvalid_until: 2026-12-31\n◈ C: d\nvalid_until: sprint-end\n~~◇ E: f~~\n",
            "CLAUDE.md",
        );
        assert_eq!(c[0].valid_until_text(), Some("2026-12-31"));
        assert!(c[0].is_scheduled());
        assert_eq!(c[1].valid_until_text(), Some("sprint-end"));
        assert!(!c[1].is_scheduled());
        assert_eq!(c[2].retired_on(), None);
        assert!(c[2].is_retired());
        let line = crate::entry_line(&crate::entry_out(&c[1]));
        assert_eq!(line, "◈ C (unscheduled sprint-end)");
    }

    #[test]
    fn a_fifo_is_never_opened_and_a_huge_file_is_read_only_to_its_cap() {
        let root = tree("hazards", &["rooms", "rooms/work"]);
        fs::write(root.join("CLAUDE.md"), "# p\n").unwrap();
        fs::write(root.join("rooms/work/CLAUDE.md"), "# w\n").unwrap();
        let fifo = root.join("rooms/work/pipe.md");
        let made = std::process::Command::new("mkfifo").arg(&fifo).status();
        let have_fifo = matches!(made, Ok(s) if s.success());
        // A crystal whose frontmatter is followed by far more than the cap.
        let mut big = String::from("---\ntype: crystal\ntier: \"◆\"\n---\n# Big\nThe fact.\n\n");
        big.push_str(&"x".repeat(FILE_CAP as usize * 4));
        fs::write(root.join("rooms/work/big.md"), &big).unwrap();
        let r = read(&root).unwrap();
        let _ = fs::remove_dir_all(&root);
        assert_eq!(r.crystals.len(), 1);
        assert_eq!(r.crystals[0].fact, "The fact.");
        if have_fifo {
            assert!(!r.documents.iter().any(|d| d.ends_with("pipe.md")));
        }
    }

    #[test]
    fn workfolder_md_is_read_as_a_container_and_is_not_a_document() {
        // `WORKFOLDER.md` as the only marker, at a rooms-at-root workfolder (the scan root is the
        // root, so the file is also walked as a .md file: it must not become a document).
        let root = tree("wfmd", &["work"]);
        fs::write(root.join("WORKFOLDER.md"), "# w\n◆ Owner: Ada\n").unwrap();
        fs::write(root.join("work/CLAUDE.md"), "# w\n").unwrap();
        let r = read(&root).unwrap();
        let _ = fs::remove_dir_all(&root);
        assert_eq!(r.crystals.len(), 1);
        assert_eq!(r.crystals[0].name.as_deref(), Some("Owner"));
        assert!(r.documents.is_empty(), "documents: {:?}", r.documents);
    }

    #[test]
    fn a_bom_does_not_hide_a_crystal() {
        let root = tree("bom", &["rooms", "rooms/work"]);
        fs::write(root.join("CLAUDE.md"), "\u{feff}◆ First: yes\n").unwrap();
        fs::write(root.join("rooms/work/CLAUDE.md"), "# w\n").unwrap();
        fs::write(
            root.join("rooms/work/c.md"),
            "\u{feff}---\ntype: crystal\ntier: \"◈\"\n---\n# C\nfact\n",
        )
        .unwrap();
        let r = read(&root).unwrap();
        let _ = fs::remove_dir_all(&root);
        assert_eq!(r.crystals.len(), 2);
        assert!(r.documents.is_empty());
    }

    #[test]
    fn a_symlinked_room_is_read_once_and_a_loop_ends() {
        let root = tree("links", &["rooms"]);
        let outside = scratch("links_outside");
        fs::write(root.join("CLAUDE.md"), "# p\n").unwrap();
        fs::write(outside.join("CLAUDE.md"), "◇ Linked: seen\n").unwrap();
        std::os::unix::fs::symlink(&outside, root.join("rooms/linked")).unwrap();
        std::os::unix::fs::symlink(&root, outside.join("back")).unwrap();
        let r = read(&root).unwrap();
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&outside);
        assert_eq!(r.crystals.len(), 1);
        assert_eq!(r.crystals[0].name.as_deref(), Some("Linked"));
    }

    #[test]
    fn a_file_named_like_a_container_keeps_its_slug_and_labels_are_one_clean_line() {
        let (c, _) = read_lines("◆ A: b\n", "rooms/w/MYCLAUDE.md");
        let e = crate::entry_out(&c[0]);
        assert_eq!(e.slug.as_deref(), Some("MYCLAUDE"));
        let (c, _) = read_lines("◆ A: b\n", "rooms/w/CLAUDE.md");
        assert_eq!(crate::entry_out(&c[0]).slug, None);

        assert_eq!(crate::clean_label("a\u{1b}[31m\nb", 20), "a [31m b");
        let long = "x".repeat(300);
        assert_eq!(crate::clean_label(&long, 120).chars().count(), 120);
        assert_eq!(crate::plural(1, "crystal", "crystals"), "1 crystal");
        assert_eq!(crate::plural(0, "crystal", "crystals"), "0 crystals");
    }

    #[test]
    fn only_legacy_shaped_bullets_count_as_notes() {
        let text = "- Ask before you write\n* Keep it short\n- **Gardening** = old style\n* **Other** = too\n";
        let (c, notes) = read_lines(text, "CLAUDE.md");
        assert!(c.is_empty());
        assert_eq!(notes, 2);
    }
}
