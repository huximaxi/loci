//! `loci check`: ask a workfolder whether it still agrees with itself.
//!
//! Read-only. It reads the workfolder the way `status` does, runs every registered rule over what
//! it read, and reports findings as `path:line` locations. It never edits a file, uses no model
//! and no network, and reads the clock only to fill in the default of `--as-of`. A finding is a
//! proposal; the person decides.
//!
//! **A rule** is a record in [`RULES`]: an id, a severity, a one-line summary and a function that
//! returns hits. A rule never writes its own id or severity into a finding: the engine stamps
//! both from the record, so a finding cannot claim to be another rule's. The engine also refuses a
//! finding that has no usable `path:line`. Rule ids and the keys of the `--json` output are
//! frozen from 2.0; new rule ids and new keys are additive. A `fail` finding makes the run exit 4;
//! a `warn` is reported and the run still exits 0.
//!
//! **A rule must prove it can see.** Each rule ships a workfolder that makes it fire
//! (`tests/fixtures/check/<id>/planted/`), one that does not (`.../clean/`) and an `expected.txt`
//! naming the `path:line` the planted one must point at. [`pair_problems`] runs every registered
//! rule over both and names every rule that cannot tell them apart or fires for the wrong reason,
//! so a rule that always passes, always fires or points nowhere fails CI. Silence is never read
//! as health either: the output always says how many rules ran, how many crystals were read, and
//! which files it could not read in full.

use crate::format2::{self, Reading};
use crate::palace::Palace;
use crate::recall::plain;
use crate::tokens;
use crate::{Error, Outcome};
use serde::Serialize;
use std::path::PathBuf;

/// How serious a finding is, and whether it fails the run (`CONTRACT.md`). A `fail` makes the run
/// exit 4. A `warn` is printed and the run still exits 0, so a rule that may be wrong can ship
/// without turning an unchanged workfolder from 0 to 4. More values may be added; each one must
/// say which side of the exit it is on, in [`Severity::fails_the_run`], which has no wildcard arm.
// No rule is registered yet, so neither variant is built outside the tests.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Likely a real problem; a false positive is plausible.
    Warn,
    /// A problem by the format's own definition.
    Fail,
}

impl Severity {
    fn word(self) -> &'static str {
        match self {
            Severity::Warn => "warn",
            Severity::Fail => "fail",
        }
    }

    /// Whether a finding of this severity makes the run exit 4.
    fn fails_the_run(self) -> bool {
        match self {
            Severity::Warn => false,
            Severity::Fail => true,
        }
    }
}

/// How wide a rule compares crystals (`--scope`). A rule that does not compare across rooms
/// ignores it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Scope {
    /// Within one room (the default): the same name in two rooms is not a clash. Crystals that
    /// live outside any room (the root files, `garden/`) are one group of their own.
    Room,
    /// Across every room of the workfolder.
    Workfolder,
}

impl Scope {
    fn word(self) -> &'static str {
        match self {
            Scope::Room => "room",
            Scope::Workfolder => "workfolder",
        }
    }
}

/// One place a finding points at. `path` is relative to the workfolder root, `/`-separated;
/// `line` is 1-based.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Location {
    pub path: String,
    pub line: usize,
}

/// What a rule returns. The engine turns it into a [`Finding`] by adding the rule's id and severity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    /// One sentence that says what is wrong and how to clear it.
    pub message: String,
    /// At least one, each a root-relative path and a line from 1; the first is the one a person
    /// opens. The engine refuses a hit without them.
    pub locations: Vec<Location>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    pub rule: &'static str,
    pub severity: Severity,
    pub message: String,
    pub locations: Vec<Location>,
}

/// What a rule can see: the workfolder as `status` reads it, plus the two flags.
// No rule is registered yet, so these are read only by the tests' demo rules; the tests check
// that each flag reaches the rules.
#[allow(dead_code)]
pub struct Ctx<'a> {
    /// For the rules that read beside the crystals (the latest handover, a rule's own files).
    pub palace: &'a Palace,
    pub reading: &'a Reading,
    pub scope: Scope,
    /// Today, or `--as-of`: `YYYY-MM-DD`. A rule that compares against a date uses this, never the clock.
    pub as_of: &'a str,
}

pub struct Rule {
    /// Stable, dotted, lower-case: `family.name`. Frozen once released.
    pub id: &'static str,
    pub severity: Severity,
    /// One line, shown in the rule list of the output.
    pub summary: &'static str,
    pub check: fn(&Ctx) -> Vec<Hit>,
}

/// The registered rules, in the order they run. Empty until the rules land one by one; each
/// brings its planted and clean fixture, and [`pair_problems`] holds it to that.
pub const RULES: &[Rule] = &[];

// ── engine ───────────────────────────────────────────────────────────────

/// How many findings one rule produced, for the rule list.
#[derive(Debug, Clone, Serialize)]
pub struct RuleRun {
    pub id: &'static str,
    pub severity: Severity,
    pub summary: &'static str,
    pub finding_count: usize,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub rules: Vec<RuleRun>,
    pub findings: Vec<Finding>,
}

/// A location a person can open: a root-relative path made only of ordinary names, and a line
/// from 1.
fn usable(l: &Location) -> bool {
    use std::path::{Component, Path};
    l.line >= 1
        && !l.path.is_empty()
        && Path::new(&l.path)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
}

/// Run every rule. Findings come back in a fixed order (all their locations, then rule id, then
/// message), so the same files and flags give the same output. A hit without a usable location is
/// a bug in the rule: the run stops and names it, rather than print a finding nobody can open.
pub fn evaluate(rules: &[Rule], ctx: &Ctx) -> Result<Report, Error> {
    let mut findings = Vec::new();
    let mut runs = Vec::new();
    for rule in rules {
        let mut n = 0;
        for hit in (rule.check)(ctx) {
            if hit.locations.is_empty() || !hit.locations.iter().all(usable) {
                return Err(Error::io(format!(
                    "rule {} returned a finding with no usable path:line. \
                     That is a bug in loci, not in your files.",
                    rule.id
                )));
            }
            n += 1;
            findings.push(Finding {
                rule: rule.id,
                severity: rule.severity,
                message: hit.message,
                locations: hit.locations,
            });
        }
        runs.push(RuleRun {
            id: rule.id,
            severity: rule.severity,
            summary: rule.summary,
            finding_count: n,
        });
    }
    findings.sort_by(|a, b| {
        (&a.locations, a.rule, &a.message).cmp(&(&b.locations, b.rule, &b.message))
    });
    Ok(Report {
        rules: runs,
        findings,
    })
}

// ── command ──────────────────────────────────────────────────────────────

pub struct Args {
    pub workfolder: Option<PathBuf>,
    pub scope: Scope,
    pub as_of: Option<String>,
}

/// Everything one run found out, owned, so a test can look at what the rules were given.
struct Gathered {
    palace: String,
    layout: &'static str,
    scope: Scope,
    as_of: String,
    crystals: usize,
    legacy: usize,
    unread: Vec<String>,
    report: Report,
}

#[derive(Serialize)]
struct Out<'a> {
    palace: &'a str,
    layout: &'a str,
    scope: &'a str,
    as_of: &'a str,
    rules_run: usize,
    crystals_considered: usize,
    legacy_lines_not_counted: usize,
    files_unread: &'a [String],
    rules: &'a [RuleRun],
    findings: &'a [Finding],
}

impl Gathered {
    fn out(&self) -> Out<'_> {
        Out {
            palace: &self.palace,
            layout: self.layout,
            scope: self.scope.word(),
            as_of: &self.as_of,
            rules_run: self.report.rules.len(),
            crystals_considered: self.crystals,
            legacy_lines_not_counted: self.legacy,
            files_unread: &self.unread,
            rules: &self.report.rules,
            findings: &self.report.findings,
        }
    }
}

pub fn run(args: Args, json: bool) -> Result<Outcome, Error> {
    run_with(RULES, args, json)
}

fn gather(rules: &[Rule], args: Args) -> Result<Gathered, Error> {
    let as_of = match &args.as_of {
        Some(d) if format2::real_date(d) => d.clone(),
        Some(d) => {
            return Err(Error::bad_input(format!(
                "--as-of must be a real date written YYYY-MM-DD, got {d}"
            )))
        }
        None => tokens::epoch_to_iso(tokens::epoch_now())[..10].to_string(),
    };
    let p = crate::require_palace(args.workfolder)?;
    let reading = format2::read_palace(&p);
    let ctx = Ctx {
        palace: &p,
        reading: &reading,
        scope: args.scope,
        as_of: &as_of,
    };
    let report = evaluate(rules, &ctx)?;
    Ok(Gathered {
        palace: p.root.display().to_string(),
        layout: reading.layout,
        scope: args.scope,
        as_of: as_of.clone(),
        crystals: reading.crystals.len(),
        legacy: reading.notes,
        unread: reading.unread.clone(),
        report,
    })
}

fn run_with(rules: &[Rule], args: Args, json: bool) -> Result<Outcome, Error> {
    let g = gather(rules, args)?;
    let out = g.out();
    if json {
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        print!("{}", render_text(&out));
    }
    Ok(
        if g.report.findings.iter().any(|f| f.severity.fails_the_run()) {
            Outcome::Findings
        } else {
            Outcome::Clean
        },
    )
}

fn render_text(o: &Out) -> String {
    use std::fmt::Write;
    let mut t = String::new();
    for f in o.findings {
        let _ = writeln!(
            t,
            "{:<5} {}  {}",
            f.severity.word(),
            f.rule,
            plain(&f.message)
        );
        for l in &f.locations {
            let _ = writeln!(t, "      {}:{}", plain(&l.path), l.line);
        }
    }
    if !o.findings.is_empty() {
        t.push('\n');
    }
    let rules = match o.rules_run {
        0 => "0 rules ran: nothing was checked".to_string(),
        1 => "1 rule ran".to_string(),
        n => format!("{n} rules ran"),
    };
    let crystals = match o.crystals_considered {
        1 => "1 crystal".to_string(),
        n => format!("{n} crystals"),
    };
    let findings = match o.findings.len() {
        0 => "no findings".to_string(),
        1 => "1 finding".to_string(),
        n => format!("{n} findings"),
    };
    let _ = writeln!(t, "{rules}, {crystals} considered, {findings}.");
    if o.legacy_lines_not_counted > 0 {
        let _ = writeln!(
            t,
            "{} legacy lines not counted (bullets without a tier glyph).",
            o.legacy_lines_not_counted
        );
    }
    if !o.files_unread.is_empty() {
        let files = match o.files_unread.len() {
            1 => "1 file".to_string(),
            n => format!("{n} files"),
        };
        let names: Vec<String> = o.files_unread.iter().map(|f| plain(f)).collect();
        let _ =
            writeln!(
            t,
            "{files} could not be read in full, so nothing in {} was checked past that point: {}.",
            if o.files_unread.len() == 1 { "it" } else { "them" },
            names.join(", ")
        );
    }
    let _ = writeln!(
        t,
        "layout {}, scope {}, as of {}. A check reads; it never edits.",
        o.layout, o.scope, o.as_of
    );
    t
}

// ── the guard: a rule must see ───────────────────────────────────────────

/// Where a rule's fixtures live, under the crate's `tests/fixtures/check/`.
#[cfg(test)]
const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/check");

/// The date the guard runs every rule at. A fixture that depends on a date is written against it.
#[cfg(test)]
const GUARD_AS_OF: &str = "2026-11-24";

/// `family.name`: two or more dot-separated words, each starting with a lower-case letter and made
/// of lower-case letters, digits, `_` and `-`. This also keeps an id from escaping the fixtures
/// directory (`..`, `/`) or sharing a directory with another id on a case-blind disk.
#[cfg(test)]
fn valid_rule_id(id: &str) -> bool {
    let words: Vec<&str> = id.split('.').collect();
    words.len() >= 2
        && words.iter().all(|w| {
            let mut ch = w.chars();
            matches!(ch.next(), Some(c) if c.is_ascii_lowercase())
                && ch.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
        })
}

/// Every file under `dir` with its bytes, in path order: a rule that writes changes this.
#[cfg(test)]
fn tree_bytes(dir: &std::path::Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn walk(dir: &std::path::Path, out: &mut Vec<(PathBuf, Vec<u8>)>) {
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .collect();
        entries.sort();
        for p in entries {
            if p.is_dir() {
                walk(&p, out);
            } else {
                let bytes = std::fs::read(&p).unwrap_or_default();
                out.push((p, bytes));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, &mut out);
    out
}

/// Why a location does not point at a real line of the fixture, or `None` when it does.
#[cfg(test)]
fn location_problem(root: &std::path::Path, l: &Location) -> Option<&'static str> {
    if !usable(l) {
        return Some("is not a root-relative path and a line from 1");
    }
    let Ok(bytes) = std::fs::read(root.join(&l.path)) else {
        return Some("names a file that is not in the fixture");
    };
    if l.line > String::from_utf8_lossy(&bytes).lines().count() {
        return Some("names a line past the end of its file");
    }
    None
}

/// The `path:line` an `expected.txt` lists: one per line, blank lines and `#` comments ignored.
#[cfg(test)]
fn expected_locations(file: &std::path::Path) -> Option<std::collections::BTreeSet<String>> {
    let text = std::fs::read_to_string(file).ok()?;
    Some(
        text.lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(str::to_string)
            .collect(),
    )
}

/// Every way the registered rules fail to prove they can see. Empty means each rule has a clean
/// id and summary, fires on its planted fixture at exactly the places `expected.txt` names, stays
/// quiet on its clean one, gives the same answer twice, points only at real lines, and leaves the
/// fixture as it found it. Each fixture is run under both scopes: a rule that only compares
/// across rooms fires on the planted one under the wide scope, and no rule may fire on the clean
/// one under either.
#[cfg(test)]
fn pair_problems(rules: &[Rule], fixtures: &std::path::Path) -> Vec<String> {
    use crate::palace;
    let mut problems = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for rule in rules {
        if !seen.insert(rule.id) {
            problems.push(format!("{}: registered twice", rule.id));
            continue;
        }
        if !valid_rule_id(rule.id) {
            // Not safe to join onto the fixtures directory, so not looked up at all.
            problems.push(format!(
                "{}: the id must be family.name in lower case (letters, digits, `_`, `-`)",
                rule.id
            ));
            continue;
        }
        if rule.summary.trim().is_empty() {
            problems.push(format!("{}: needs a summary", rule.id));
        }
        let mut pointed_at = std::collections::BTreeSet::new();
        for (kind, must_fire) in [("planted", true), ("clean", false)] {
            let dir = fixtures.join(rule.id).join(kind);
            let Some(p) = palace::validate(&dir) else {
                problems.push(format!(
                    "{}: no {kind} fixture (a workfolder at {})",
                    rule.id,
                    dir.display()
                ));
                continue;
            };
            let reading = format2::read_palace(&p);
            let before = tree_bytes(&dir);
            let mut fired = false;
            for scope in [Scope::Room, Scope::Workfolder] {
                let ctx = Ctx {
                    palace: &p,
                    reading: &reading,
                    scope,
                    as_of: GUARD_AS_OF,
                };
                let hits = (rule.check)(&ctx);
                if hits != (rule.check)(&ctx) {
                    problems.push(format!(
                        "{}: gave different findings on two runs over the {kind} fixture",
                        rule.id
                    ));
                }
                fired |= !hits.is_empty();
                if !must_fire && !hits.is_empty() {
                    problems.push(format!(
                        "{}: fires on its clean fixture (scope {})",
                        rule.id,
                        scope.word()
                    ));
                }
                for h in &hits {
                    if h.locations.is_empty() {
                        problems.push(format!(
                            "{}: a finding on the {kind} fixture has no path:line",
                            rule.id
                        ));
                    }
                    for l in &h.locations {
                        if let Some(why) = location_problem(&dir, l) {
                            problems.push(format!(
                                "{}: a finding on the {kind} fixture points at {}:{}, which {why}",
                                rule.id, l.path, l.line
                            ));
                        }
                    }
                    if must_fire {
                        if let Some(l) = h.locations.first() {
                            pointed_at.insert(format!("{}:{}", l.path, l.line));
                        }
                    }
                }
            }
            if must_fire && !fired {
                problems.push(format!("{}: silent on its planted fixture", rule.id));
            }
            if before != tree_bytes(&dir) {
                problems.push(format!(
                    "{}: changed a file of its {kind} fixture; a rule only reads",
                    rule.id
                ));
            }
        }
        match expected_locations(&fixtures.join(rule.id).join("expected.txt")) {
            None => problems.push(format!(
                "{}: no expected.txt (the path:line its planted fixture must point at)",
                rule.id
            )),
            Some(want) if want != pointed_at => problems.push(format!(
                "{}: the planted fixture is pointed at {:?} but expected.txt lists {:?}",
                rule.id, pointed_at, want
            )),
            Some(_) => {}
        }
    }
    problems
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::fs;
    use std::path::Path;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn scratch(name: &str) -> PathBuf {
        let p =
            std::env::temp_dir().join(format!("loci_cli_check_{}_{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    /// A tiny workfolder: a marker and one room holding `body` as its CLAUDE.md.
    fn workfolder(root: &Path, body: &str) {
        fs::create_dir_all(root.join("rooms/work")).unwrap();
        fs::write(root.join("WORKFOLDER.md"), "# workfolder\n").unwrap();
        fs::write(root.join("rooms/work/CLAUDE.md"), body).unwrap();
    }

    fn at(path: &str, line: usize) -> Location {
        Location {
            path: path.into(),
            line,
        }
    }

    /// A rule that fires when the room file says `BAD`, pointing at it.
    fn sees_bad(ctx: &Ctx) -> Vec<Hit> {
        ctx.reading
            .crystals
            .iter()
            .filter(|c| c.fact.contains("BAD"))
            .map(|c| Hit {
                message: format!("{} says BAD", c.file),
                locations: vec![at(&c.file, c.line)],
            })
            .collect()
    }

    /// The same rule, but it only compares across rooms: it fires under the wide scope only.
    fn sees_bad_wide(ctx: &Ctx) -> Vec<Hit> {
        if ctx.scope == Scope::Workfolder {
            sees_bad(ctx)
        } else {
            Vec::new()
        }
    }

    fn never(_: &Ctx) -> Vec<Hit> {
        Vec::new()
    }

    fn always(_: &Ctx) -> Vec<Hit> {
        vec![Hit {
            message: "always".into(),
            locations: vec![at("x.md", 1)],
        }]
    }

    fn unlocated(_: &Ctx) -> Vec<Hit> {
        vec![Hit {
            message: "nowhere".into(),
            locations: Vec::new(),
        }]
    }

    fn at_line_zero(_: &Ctx) -> Vec<Hit> {
        vec![Hit {
            message: "line zero".into(),
            locations: vec![at("rooms/work/CLAUDE.md", 0)],
        }]
    }

    /// Fires on the planted fixture, but at a line that is not the defect.
    fn wrong_reason(ctx: &Ctx) -> Vec<Hit> {
        if sees_bad(ctx).is_empty() {
            Vec::new()
        } else {
            vec![Hit {
                message: "something else".into(),
                locations: vec![at("WORKFOLDER.md", 1)],
            }]
        }
    }

    fn points_past_the_end(ctx: &Ctx) -> Vec<Hit> {
        sees_bad(ctx)
            .into_iter()
            .map(|h| Hit {
                locations: vec![at("rooms/work/CLAUDE.md", 999)],
                ..h
            })
            .collect()
    }

    fn points_at_a_missing_file(ctx: &Ctx) -> Vec<Hit> {
        sees_bad(ctx)
            .into_iter()
            .map(|h| Hit {
                locations: vec![at("nonexistent/file.md", 1)],
                ..h
            })
            .collect()
    }

    static CALLS: AtomicUsize = AtomicUsize::new(0);

    /// Answers differently every other call.
    fn flaky(ctx: &Ctx) -> Vec<Hit> {
        if CALLS.fetch_add(1, Ordering::SeqCst) & 1 == 0 {
            sees_bad(ctx)
        } else {
            Vec::new()
        }
    }

    /// Finds the defect, and leaves a file behind.
    fn writes(ctx: &Ctx) -> Vec<Hit> {
        let _ = fs::write(ctx.palace.root.join("stray.txt"), "x");
        sees_bad(ctx)
    }

    /// Reports the two flags it was given, pointing at the marker.
    fn reports_flags(ctx: &Ctx) -> Vec<Hit> {
        vec![Hit {
            message: format!("scope={} as_of={}", ctx.scope.word(), ctx.as_of),
            locations: vec![at("WORKFOLDER.md", 1)],
        }]
    }

    const GOOD: &str = "◆ fine = a fact that holds\n";
    const BAD: &str = "◆ broken = BAD fact\n";

    fn rule(id: &'static str, check: fn(&Ctx) -> Vec<Hit>) -> Rule {
        Rule {
            id,
            severity: Severity::Warn,
            summary: "test rule",
            check,
        }
    }

    /// Fixtures for one synthetic rule: planted holds `BAD`, clean holds `GOOD`, and `expected.txt`
    /// names the line of the defect.
    fn pair(base: &Path, id: &str) {
        workfolder(&base.join(id).join("planted"), BAD);
        workfolder(&base.join(id).join("clean"), GOOD);
        fs::write(
            base.join(id).join("expected.txt"),
            "# the defect\nrooms/work/CLAUDE.md:1\n",
        )
        .unwrap();
    }

    fn problems_of(base: &Path, rules: &[Rule]) -> Vec<String> {
        pair_problems(rules, base)
    }

    fn said(problems: &[String], needle: &str) -> bool {
        problems.iter().any(|p| p.contains(needle))
    }

    fn only_about(problems: &[String], id: &str) -> Vec<String> {
        problems
            .iter()
            .filter(|p| p.starts_with(&format!("{id}:")))
            .cloned()
            .collect()
    }

    #[test]
    fn the_shipped_rules_each_see_their_planted_case_and_pass_their_clean_one() {
        let problems = pair_problems(RULES, Path::new(FIXTURES));
        assert!(problems.is_empty(), "{problems:#?}");
    }

    #[test]
    fn the_guard_passes_a_rule_that_tells_planted_from_clean() {
        let base = scratch("guard_ok");
        pair(&base, "demo.sees");
        pair(&base, "demo.wide");
        let problems = problems_of(
            &base,
            &[
                rule("demo.sees", sees_bad),
                // A rule that only compares across rooms is held to the wide scope, not blamed.
                rule("demo.wide", sees_bad_wide),
            ],
        );
        assert!(problems.is_empty(), "{problems:#?}");
        let _ = fs::remove_dir_all(&base);
    }

    /// The guard is itself a sensor, so it is tested against rules built to fail it: a check that
    /// always passes, one that always fires, one with no fixtures, one that points nowhere.
    #[test]
    fn the_guard_fails_a_rule_that_cannot_see() {
        let base = scratch("guard_bad");
        for id in [
            "demo.sees",
            "demo.never",
            "demo.always",
            "demo.nowhere",
            "demo.zero",
        ] {
            pair(&base, id);
        }
        workfolder(&base.join("demo.half/planted"), BAD); // no clean fixture, no expected.txt

        let problems = problems_of(
            &base,
            &[
                rule("demo.sees", sees_bad),
                rule("demo.never", never),
                rule("demo.always", always),
                rule("demo.nowhere", unlocated),
                rule("demo.zero", at_line_zero),
                rule("demo.half", sees_bad),
                rule("demo.missing", sees_bad),
                rule("demo.sees", sees_bad),
            ],
        );
        assert!(
            said(&problems, "demo.never: silent on its planted fixture"),
            "{problems:#?}"
        );
        assert!(
            said(&problems, "demo.always: fires on its clean fixture"),
            "{problems:#?}"
        );
        assert!(
            said(
                &problems,
                "demo.nowhere: a finding on the planted fixture has no path:line"
            ),
            "{problems:#?}"
        );
        assert!(
            said(
                &problems,
                "demo.zero: a finding on the planted fixture points at"
            ),
            "{problems:#?}"
        );
        assert!(
            said(&problems, "demo.half: no clean fixture"),
            "{problems:#?}"
        );
        assert!(
            said(&problems, "demo.missing: no planted fixture"),
            "{problems:#?}"
        );
        assert!(
            said(&problems, "demo.sees: registered twice"),
            "{problems:#?}"
        );
        // The one honest rule is not blamed for anything.
        assert_eq!(
            only_about(&problems, "demo.sees"),
            vec!["demo.sees: registered twice"]
        );
        let _ = fs::remove_dir_all(&base);
    }

    /// A rule can fire on the planted fixture for the wrong reason, point at a line that is not
    /// there, or be unstable or untidy. The guard holds a rule to the fixture, not to "some hit".
    #[test]
    fn the_guard_fails_a_rule_that_fires_for_the_wrong_reason_or_points_at_nothing() {
        let base = scratch("guard_reason");
        for id in [
            "demo.wrong",
            "demo.past",
            "demo.missing-file",
            "demo.flaky",
            "demo.writes",
        ] {
            pair(&base, id);
        }
        let problems = problems_of(
            &base,
            &[
                rule("demo.wrong", wrong_reason),
                rule("demo.past", points_past_the_end),
                rule("demo.missing-file", points_at_a_missing_file),
                rule("demo.flaky", flaky),
                rule("demo.writes", writes),
            ],
        );
        assert!(
            said(&problems, "demo.wrong: the planted fixture is pointed at"),
            "{problems:#?}"
        );
        assert!(
            said(&problems, "demo.past: a finding on the planted fixture points at rooms/work/CLAUDE.md:999, which names a line past the end"),
            "{problems:#?}"
        );
        assert!(
            said(&problems, "demo.missing-file: a finding on the planted fixture points at nonexistent/file.md:1, which names a file that is not in the fixture"),
            "{problems:#?}"
        );
        assert!(
            said(&problems, "demo.flaky: gave different findings on two runs"),
            "{problems:#?}"
        );
        assert!(
            said(
                &problems,
                "demo.writes: changed a file of its planted fixture"
            ),
            "{problems:#?}"
        );
        let _ = fs::remove_dir_all(&base);
    }

    /// An id is a path segment and a name in the output: it must be one clean `family.name`, and
    /// it must not borrow another rule's fixtures or point the guard at another directory.
    #[test]
    fn the_guard_refuses_an_id_that_could_escape_or_collide_and_a_blank_summary_and_no_expectation()
    {
        let base = scratch("guard_id");
        pair(&base, "real.rule");
        pair(&base, "demo.blank");
        // Fixtures exist for the borrowed and absolute ids, so only the id check can stop them.
        let problems = problems_of(
            &base,
            &[
                rule("real.rule", sees_bad),
                rule("real.rule/../real.rule", sees_bad),
                rule("/nonexistent", sees_bad),
                rule("Real.Rule", sees_bad),
                rule("norules", sees_bad),
                rule(".lead", sees_bad),
                rule("a..b", sees_bad),
                Rule {
                    id: "demo.blank",
                    severity: Severity::Warn,
                    summary: "   ",
                    check: sees_bad,
                },
            ],
        );
        for bad in [
            "real.rule/../real.rule",
            "/nonexistent",
            "Real.Rule",
            "norules",
            ".lead",
            "a..b",
        ] {
            assert!(
                said(&problems, &format!("{bad}: the id must be family.name")),
                "{bad}: {problems:#?}"
            );
        }
        assert!(
            said(&problems, "demo.blank: needs a summary"),
            "{problems:#?}"
        );
        assert!(
            only_about(&problems, "real.rule").is_empty(),
            "{problems:#?}"
        );
        // A rule whose fixtures are fine but whose expectation is missing is not finished.
        workfolder(&base.join("demo.noexp/planted"), BAD);
        workfolder(&base.join("demo.noexp/clean"), GOOD);
        let problems = problems_of(&base, &[rule("demo.noexp", sees_bad)]);
        assert!(
            said(&problems, "demo.noexp: no expected.txt"),
            "{problems:#?}"
        );
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn ids_are_family_dot_name() {
        assert!(valid_rule_id("note.lags-handover"));
        assert!(valid_rule_id("supersession.same_name.v2"));
        assert!(!valid_rule_id("single"));
        assert!(!valid_rule_id("Upper.case"));
        assert!(!valid_rule_id("a.1b"));
        assert!(!valid_rule_id("a..b"));
        assert!(!valid_rule_id("a.b/"));
        assert!(!valid_rule_id(""));
    }

    #[test]
    fn the_engine_stamps_id_and_severity_and_orders_findings_the_same_way_every_time() {
        let base = scratch("engine");
        workfolder(&base, "◆ b = BAD one\n◆ a = BAD two\n◆ ok = fine\n");
        let p = crate::palace::validate(&base).unwrap();
        let reading = format2::read_palace(&p);
        let ctx = Ctx {
            palace: &p,
            reading: &reading,
            scope: Scope::Room,
            as_of: "2026-11-24",
        };
        let rules = [Rule {
            id: "demo.sees",
            severity: Severity::Fail,
            summary: "demo",
            check: sees_bad,
        }];
        let r = evaluate(&rules, &ctx).unwrap();
        assert_eq!(r.rules.len(), 1);
        assert_eq!(r.rules[0].finding_count, 2);
        assert_eq!(r.rules[0].severity, Severity::Fail);
        assert_eq!(r.rules[0].summary, "demo");
        assert!(r
            .findings
            .iter()
            .all(|f| f.rule == "demo.sees" && f.severity == Severity::Fail));
        let lines: Vec<usize> = r.findings.iter().map(|f| f.locations[0].line).collect();
        assert_eq!(lines, vec![1, 2], "ordered by location");
        let again = evaluate(&rules, &ctx).unwrap();
        assert_eq!(
            serde_json::to_string(&r).unwrap(),
            serde_json::to_string(&again).unwrap()
        );
        let _ = fs::remove_dir_all(&base);
    }

    /// Emitted in the worst order: later files first, and a tie on first location, rule and message
    /// that only the later locations can break.
    fn jumbled(_: &Ctx) -> Vec<Hit> {
        let hit = |locs: &[(&str, usize)]| Hit {
            message: "m".into(),
            locations: locs.iter().map(|(p, l)| at(p, *l)).collect(),
        };
        vec![
            hit(&[("z.md", 1)]),
            hit(&[("x.md", 1), ("z.md", 1)]),
            hit(&[("x.md", 1), ("a.md", 1)]),
            hit(&[("a.md", 2)]),
            hit(&[("a.md", 1)]),
        ]
    }

    #[test]
    fn findings_sort_on_every_location_then_rule_then_message_whatever_the_emission_order() {
        let base = scratch("order");
        workfolder(&base, GOOD);
        let p = crate::palace::validate(&base).unwrap();
        let reading = format2::read_palace(&p);
        let ctx = Ctx {
            palace: &p,
            reading: &reading,
            scope: Scope::Room,
            as_of: "2026-11-24",
        };
        let r = evaluate(&[rule("b.rule", jumbled), rule("a.rule", jumbled)], &ctx).unwrap();
        let key = |f: &Finding| {
            let locs: Vec<String> = f
                .locations
                .iter()
                .map(|l| format!("{}:{}", l.path, l.line))
                .collect();
            format!("{} {}", locs.join(","), f.rule)
        };
        let got: Vec<String> = r.findings.iter().map(key).collect();
        let want = [
            "a.md:1 a.rule",
            "a.md:1 b.rule",
            "a.md:2 a.rule",
            "a.md:2 b.rule",
            "x.md:1,a.md:1 a.rule",
            "x.md:1,a.md:1 b.rule",
            "x.md:1,z.md:1 a.rule",
            "x.md:1,z.md:1 b.rule",
            "z.md:1 a.rule",
            "z.md:1 b.rule",
        ];
        assert_eq!(got, want);
        let _ = fs::remove_dir_all(&base);
    }

    /// A finding nobody can open is a bug in the rule: the run stops and names the rule.
    #[test]
    fn a_finding_without_a_usable_location_stops_the_run_and_names_the_rule() {
        let base = scratch("unusable");
        workfolder(&base, GOOD);
        let p = crate::palace::validate(&base).unwrap();
        let reading = format2::read_palace(&p);
        let ctx = Ctx {
            palace: &p,
            reading: &reading,
            scope: Scope::Room,
            as_of: "2026-11-24",
        };
        fn absolute(_: &Ctx) -> Vec<Hit> {
            vec![Hit {
                message: "m".into(),
                locations: vec![at("/etc/passwd", 1)],
            }]
        }
        fn parent(_: &Ctx) -> Vec<Hit> {
            vec![Hit {
                message: "m".into(),
                locations: vec![at("../outside.md", 1)],
            }]
        }
        fn empty_path(_: &Ctx) -> Vec<Hit> {
            vec![Hit {
                message: "m".into(),
                locations: vec![at("", 1)],
            }]
        }
        for (name, check) in [
            ("unlocated", unlocated as fn(&Ctx) -> Vec<Hit>),
            ("line_zero", at_line_zero),
            ("absolute", absolute),
            ("parent", parent),
            ("empty_path", empty_path),
        ] {
            let err = evaluate(&[rule("demo.bad", check)], &ctx)
                .err()
                .unwrap_or_else(|| panic!("{name}: the engine accepted an unusable location"));
            assert!(matches!(err.kind(), crate::ErrKind::Io), "{name}");
            assert!(err.to_string().contains("rule demo.bad"), "{name}: {err}");
        }
        // A hit that is fine in the same run as another rule's is not collateral.
        assert!(evaluate(&[rule("demo.ok", always)], &ctx).is_ok());
        let _ = fs::remove_dir_all(&base);
    }

    fn demo_out<'a>(report: &'a Report, crystals: usize) -> Out<'a> {
        Out {
            palace: "/w",
            layout: "rooms-dir",
            scope: "room",
            as_of: "2026-11-24",
            rules_run: report.rules.len(),
            crystals_considered: crystals,
            legacy_lines_not_counted: 0,
            files_unread: &[],
            rules: &report.rules,
            findings: &report.findings,
        }
    }

    fn demo_run(id: &'static str, findings: usize) -> RuleRun {
        RuleRun {
            id,
            severity: Severity::Warn,
            summary: "demo",
            finding_count: findings,
        }
    }

    #[test]
    fn the_text_says_how_many_rules_ran_and_how_many_crystals_were_read_even_when_quiet() {
        let none = Report {
            rules: vec![],
            findings: vec![],
        };
        let t = render_text(&demo_out(&none, 0));
        assert!(t.contains("0 rules ran: nothing was checked, 0 crystals considered, no findings."));
        let one = Report {
            rules: vec![demo_run("demo.sees", 0)],
            findings: vec![],
        };
        let t = render_text(&demo_out(&one, 24));
        assert!(
            t.contains("1 rule ran, 24 crystals considered, no findings."),
            "{t}"
        );
        let t = render_text(&demo_out(&one, 1));
        assert!(
            t.contains("1 rule ran, 1 crystal considered, no findings."),
            "{t}"
        );
    }

    #[test]
    fn the_text_names_every_file_it_could_not_read_in_full() {
        let r = Report {
            rules: vec![demo_run("demo.sees", 0)],
            findings: vec![],
        };
        let unread = vec![
            "CLAUDE.md".to_string(),
            "rooms/a\u{1b}[31m/CLAUDE.md".to_string(),
        ];
        let mut o = demo_out(&r, 3);
        o.files_unread = &unread;
        let t = render_text(&o);
        assert!(
            t.contains("2 files could not be read in full, so nothing in them was checked past that point: CLAUDE.md, rooms/a [31m/CLAUDE.md."),
            "{t:?}"
        );
        assert!(!t.contains('\u{1b}'));
        o.files_unread = &unread[..1];
        let t = render_text(&o);
        assert!(
            t.contains("1 file could not be read in full, so nothing in it was checked past that point: CLAUDE.md."),
            "{t}"
        );
        assert!(!render_text(&demo_out(&r, 3)).contains("could not be read"));
    }

    #[test]
    fn a_finding_prints_severity_rule_message_and_every_location_and_hides_control_characters() {
        let r = Report {
            rules: vec![demo_run("demo.sees", 1)],
            findings: vec![Finding {
                rule: "demo.sees",
                severity: Severity::Warn,
                message: "two things\u{1b}[31m clash".into(),
                locations: vec![at("rooms/work/CLAUDE.md", 3), at("rooms/work/x.md", 1)],
            }],
        };
        let t = render_text(&demo_out(&r, 5));
        assert!(t.starts_with("warn  demo.sees  two things [31m clash\n      rooms/work/CLAUDE.md:3\n      rooms/work/x.md:1\n\n"), "{t:?}");
        assert!(!t.contains('\u{1b}'));
        assert!(t.contains("1 rule ran, 5 crystals considered, 1 finding."));
    }

    fn keys(v: &Value) -> Vec<&str> {
        let mut k: Vec<&str> = v.as_object().unwrap().keys().map(|k| k.as_str()).collect();
        k.sort();
        k
    }

    /// The frozen shape, down to the entries: the nested keys are part of the contract too.
    #[test]
    fn the_envelope_keys_are_the_frozen_ones_at_every_level() {
        let r = Report {
            rules: vec![demo_run("demo.sees", 1)],
            findings: vec![Finding {
                rule: "demo.sees",
                severity: Severity::Fail,
                message: "m".into(),
                locations: vec![at("a.md", 2)],
            }],
        };
        let v = serde_json::to_value(demo_out(&r, 1)).unwrap();
        assert_eq!(
            keys(&v),
            vec![
                "as_of",
                "crystals_considered",
                "files_unread",
                "findings",
                "layout",
                "legacy_lines_not_counted",
                "palace",
                "rules",
                "rules_run",
                "scope"
            ]
        );
        let rule = &v["rules"][0];
        assert_eq!(
            keys(rule),
            vec!["finding_count", "id", "severity", "summary"]
        );
        assert_eq!(rule["id"], "demo.sees");
        assert_eq!(rule["severity"], "warn");
        assert_eq!(rule["summary"], "demo");
        assert_eq!(rule["finding_count"], 1);
        let f = &v["findings"][0];
        assert_eq!(keys(f), vec!["locations", "message", "rule", "severity"]);
        assert_eq!(f["rule"], "demo.sees");
        assert_eq!(f["severity"], "fail");
        assert_eq!(f["message"], "m");
        assert_eq!(keys(&f["locations"][0]), vec!["line", "path"]);
        assert_eq!(f["locations"][0]["path"], "a.md");
        assert_eq!(f["locations"][0]["line"], 2);
        assert!(v["files_unread"].as_array().unwrap().is_empty());
    }

    /// The flags a person gives are the flags the rules get: a rule that reads `ctx.scope` or
    /// `ctx.as_of` sees what was asked for, and the defaults are today and the narrow scope.
    #[test]
    fn the_flags_reach_the_rules() {
        let base = scratch("flags");
        workfolder(&base, GOOD);
        let ask = |scope, as_of: Option<&str>| {
            let g = gather(
                &[rule("demo.flags", reports_flags)],
                Args {
                    workfolder: Some(base.clone()),
                    scope,
                    as_of: as_of.map(str::to_string),
                },
            )
            .unwrap();
            (g.report.findings[0].message.clone(), g)
        };
        let (m, g) = ask(Scope::Workfolder, Some("2027-01-02"));
        assert_eq!(m, "scope=workfolder as_of=2027-01-02");
        assert_eq!(g.scope, Scope::Workfolder);
        assert_eq!(g.as_of, "2027-01-02");
        let (m, _) = ask(Scope::Room, Some("2026-11-24"));
        assert_eq!(m, "scope=room as_of=2026-11-24");
        // No `--as-of`: today. Not the epoch, not a fixed day.
        let (m, g) = ask(Scope::Room, None);
        assert!(m.starts_with("scope=room as_of=20"), "{m}");
        assert!(format2::real_date(&g.as_of), "{}", g.as_of);
        assert!(g.as_of.as_str() >= "2026-11-26", "{}", g.as_of);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn only_a_fail_finding_is_the_findings_outcome_a_warn_or_a_quiet_run_is_clean() {
        let base = scratch("outcome");
        workfolder(&base, "◆ fine = ok\n");
        let args = || Args {
            workfolder: Some(base.clone()),
            scope: Scope::Room,
            as_of: Some("2026-11-24".into()),
        };
        let failing = |id: &'static str| Rule {
            severity: Severity::Fail,
            ..rule(id, always)
        };
        // A `fail` finding fails the run.
        let found = run_with(&[failing("demo.fails")], args(), true).unwrap();
        assert_eq!(found, Outcome::Findings);
        // A `warn` finding is reported and the run is still clean.
        let g = gather(&[rule("demo.warns", always)], args()).unwrap();
        assert_eq!(g.report.findings.len(), 1, "the warn is reported");
        let warned = run_with(&[rule("demo.warns", always)], args(), true).unwrap();
        assert_eq!(warned, Outcome::Clean, "a warn alone does not fail the run");
        // One `fail` among warns still fails it, in either rule order.
        for rules in [
            [rule("demo.warns", always), failing("demo.fails")],
            [failing("demo.fails"), rule("demo.warns", always)],
        ] {
            let mixed = run_with(&rules, args(), true).unwrap();
            assert_eq!(mixed, Outcome::Findings);
        }
        // A `fail` rule that finds nothing is quiet.
        let silent = Rule {
            severity: Severity::Fail,
            ..rule("demo.silent", never)
        };
        assert_eq!(run_with(&[silent], args(), true).unwrap(), Outcome::Clean);
        let quiet = run_with(&[rule("demo.never", never)], args(), true).unwrap();
        assert_eq!(quiet, Outcome::Clean);
        let bad = run_with(
            &[],
            Args {
                as_of: Some("2026-02-30".into()),
                ..args()
            },
            true,
        );
        assert!(bad.is_err(), "a date that is not real is bad input");
        let _ = fs::remove_dir_all(&base);
    }
}
