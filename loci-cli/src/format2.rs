//! Reference reader for loci format 2 (`templates/format-2.md`).
//!
//! The spec is the contract and this file is the smallest reader that follows it. It is test-only
//! until P4 wires crystal counting into `status` and `crystals`; the conformance tests at the
//! bottom read the fixture workfolders under `tests/fixtures/format2/` and compare them with
//! `expected.json`, which is written by hand from the spec and never generated from this code.
//!
//! Layout detection is not repeated here: `palace::validate` already follows the spec's table.
#![allow(dead_code)]

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
        }
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
    let (struck, body) = if line.len() > 4 && line.starts_with("~~") && line.ends_with("~~") {
        (true, &line[2..line.len() - 2])
    } else {
        (false, line)
    };
    let mut chars = body.chars();
    let tier = Tier::from_glyph(chars.next()?)?;
    let mut rest = chars.as_str().strip_prefix(' ')?.trim_end().to_string();
    let mut pinned = false;
    if let Some(r) = rest.strip_suffix(" · pinned") {
        pinned = true;
        rest = r.to_string();
    }
    let mut legacy_valid = None;
    if let Some(ix) = rest.rfind(" - valid_until: ") {
        legacy_valid = Some(rest[ix + " - valid_until: ".len()..].trim().to_string());
        rest.truncate(ix);
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

/// Crystal lines in a container file (`PALACE.md` or a `CLAUDE.md`). Returns the crystals and the
/// number of glyph-less bullets (notes). Fenced blocks hold examples; an unclosed fence runs to
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
        if t.starts_with("- ") || t.starts_with("* ") {
            notes += 1;
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

fn md_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.filter_map(|e| e.ok()) {
        let name = e.file_name().to_string_lossy().to_string();
        let p = e.path();
        if p.is_dir() {
            if !skipped(&name) {
                md_files(&p, out);
            }
        } else if p.extension().and_then(|x| x.to_str()) == Some("md") {
            out.push(p);
        }
    }
}

pub struct Reading {
    pub layout: &'static str,
    pub rooms: Vec<String>,
    pub crystals: Vec<Crystal>,
    /// `.md` files that are not crystals, other than the `CLAUDE.md` and `PALACE.md` containers.
    pub documents: Vec<String>,
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

/// Read a workfolder as the spec says: crystal lines in the root `PALACE.md` and `CLAUDE.md` and
/// in each room's `CLAUDE.md`; file crystals under the scan root and under `garden/` at the root.
pub fn read(root: &Path) -> Option<Reading> {
    let p = palace::validate(root)?;
    let layout = match p.layout {
        Layout::PalaceSubdir => "palace-subdir",
        Layout::RoomsDir => "rooms-dir",
        Layout::RoomsAtRoot => "rooms-at-root",
    };
    let rooms = palace::list_rooms(&p);
    let mut containers = vec![root.join("PALACE.md"), root.join("CLAUDE.md")];
    containers.extend(rooms.iter().map(|r| r.path.join("CLAUDE.md")));

    let mut crystals = Vec::new();
    let mut notes = 0;
    for c in containers.iter().filter(|c| c.exists()) {
        let text = fs::read_to_string(c).unwrap_or_default();
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
        let text = fs::read_to_string(&f).unwrap_or_default();
        match read_file_crystal(&text, &rel(root, &f)) {
            Some(c) => crystals.push(c),
            None if name == "CLAUDE.md" || name == "PALACE.md" => {}
            None => documents.push(rel(root, &f)),
        }
    }
    Some(Reading {
        layout,
        rooms: rooms.into_iter().map(|r| r.name).collect(),
        crystals,
        documents,
        notes,
    })
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
        for w in want_c.iter().filter(|w| !got_c.contains(w)) {
            problems.push(format!("missing crystal: {w}"));
        }
        for g in got_c.iter().filter(|g| !want_c.contains(g)) {
            problems.push(format!("unexpected crystal: {g}"));
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
        // A layout change is noticed too.
        let root = materialise("rooms-dir", "layout");
        fs::rename(root.join("rooms"), root.join("roomz")).unwrap();
        let problems = diff(&root, &want["rooms-dir"]);
        let _ = fs::remove_dir_all(&root);
        assert!(!problems.is_empty(), "renaming rooms/ went unnoticed");
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
        let root = tree("skip", &["rooms/r", "rooms/node_modules", "rooms/target"]);
        for d in ["node_modules", "target"] {
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
}
