//! The CLI contract (`CONTRACT.md`), run against the real binary: exit codes, output streams,
//! and the `--json` key set of every command that takes it, on the format 2 fixture workfolder.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_loci");
const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/format2/rooms-dir"
);

fn scratch(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("loci_contract_{}_{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap().flatten() {
        let dst = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_tree(&e.path(), &dst);
        } else {
            fs::copy(e.path(), &dst).unwrap();
        }
    }
}

/// A fixture workfolder with a handover in it, and an empty home so nothing reads the real one.
struct World {
    root: PathBuf,
    home: PathBuf,
    cwd: PathBuf,
}

impl World {
    fn new(name: &str) -> World {
        let base = scratch(name);
        let root = base.join("wf");
        copy_tree(Path::new(FIXTURE), &root);
        fs::create_dir_all(root.join("handovers")).unwrap();
        fs::write(
            root.join("handovers/HANDOVER_2026-11-24.md"),
            "# Handover\n\nnext: ship\n",
        )
        .unwrap();
        let home = base.join("home");
        let cwd = base.join("cwd");
        fs::create_dir_all(&home).unwrap();
        fs::create_dir_all(&cwd).unwrap();
        World { root, home, cwd }
    }

    /// Run `loci` from an empty directory with an empty home and no `LOCI_PALACE`.
    fn run(&self, args: &[&str]) -> Output {
        self.run_env(args, &[])
    }

    /// As `run`, with these environment variables set.
    fn run_env(&self, args: &[&str], env: &[(&str, &str)]) -> Output {
        let mut c = Command::new(BIN);
        c.args(args)
            .current_dir(&self.cwd)
            .env("HOME", &self.home)
            .env_remove("LOCI_PALACE")
            .env_remove("LOCI_WORKFOLDER")
            .env_remove("CLAUDE_CONFIG_DIR");
        for (k, v) in env {
            c.env(k, v);
        }
        c.output().expect("run loci")
    }

    fn with_palace(&self, args: &[&str]) -> Output {
        let root = self.root.to_str().unwrap();
        let mut full = vec!["--palace", root];
        full.extend_from_slice(args);
        self.run(&full)
    }
}

impl Drop for World {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(self.root.parent().unwrap());
    }
}

fn code(o: &Output) -> i32 {
    o.status.code().expect("exit code")
}

fn keys(o: &Output) -> Vec<String> {
    let text = String::from_utf8_lossy(&o.stdout);
    let v: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("stdout is not one JSON document: {e}\n{text}"));
    let mut k: Vec<String> = v
        .as_object()
        .unwrap_or_else(|| panic!("top level is not an object: {text}"))
        .keys()
        .cloned()
        .collect();
    k.sort();
    k
}

fn want(list: &[&str]) -> Vec<String> {
    let mut v: Vec<String> = list.iter().map(|s| s.to_string()).collect();
    v.sort();
    v
}

#[test]
fn every_json_command_emits_one_document_with_the_contract_keys() {
    let w = World::new("json");
    let new_target = w.cwd.join("fresh");
    let new_target = new_target.to_str().unwrap().to_string();
    let cases: Vec<(Vec<&str>, Vec<&str>)> = vec![
        (
            vec!["status"],
            vec![
                "palace",
                "layout",
                "rooms",
                "crystal_total",
                "files_total",
                "tiered",
                "documents",
                "legacy_lines_not_counted",
            ],
        ),
        (vec!["crystals"], vec!["palace", "rooms", "unrooted"]),
        (vec!["read", "decision"], vec!["slug", "path", "content"]),
        (vec!["handover"], vec!["path", "content"]),
        (
            vec!["check"],
            vec![
                "palace",
                "layout",
                "scope",
                "as_of",
                "rules_run",
                "crystals_considered",
                "legacy_lines_not_counted",
                "files_unread",
                "rules",
                "findings",
            ],
        ),
        (
            vec!["tokens"],
            vec![
                "state",
                "signal",
                "tokens",
                "messages",
                "files_scanned",
                "window_start_utc",
                "window_reset_utc",
                "minutes_remaining",
                "note",
            ],
        ),
        (
            vec!["reads"],
            vec![
                "root",
                "days",
                "sessions",
                "sessions_with_read",
                "files_scanned",
                "user_turns",
                "turns_with_read",
                "turns_with_touch",
                "palace_reads",
                "palace_writes",
                "delegated_reads",
                "delegated_writes",
                "shell_touches",
                "read_rate_pct",
                "touch_rate_pct",
                "note",
            ],
        ),
        (vec!["rain"], vec!["window", "garden", "start_phrase"]),
        (
            vec!["recall", "plain labels"],
            vec![
                "query",
                "query_tokens",
                "unmatched_tokens",
                "tokenizer",
                "scoring",
                "as_of",
                "matching_chunks",
                "scope",
                "results",
            ],
        ),
        (
            vec!["audit"],
            vec![
                "wal",
                "since",
                "frames",
                "classes",
                "chain_ok",
                "chain_break_seq",
                "degraded",
                "degraded_latest",
                "disabled_at",
                "unreadable_lines",
                "experimental",
                "note",
            ],
        ),
        (
            vec!["overview"],
            vec!["tool", "version", "palace", "groups", "doors"],
        ),
    ];
    for (args, expected) in &cases {
        let mut full = vec!["--json"];
        full.extend(args.iter().copied());
        let o = w.with_palace(&full);
        assert_eq!(
            code(&o),
            0,
            "{args:?}: {}",
            String::from_utf8_lossy(&o.stderr)
        );
        assert_eq!(keys(&o), want(expected), "{args:?}: --json keys");
    }
    // `new` takes a path, not a workfolder.
    let o = w.run(&["--json", "new", &new_target]);
    assert_eq!(code(&o), 0, "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(keys(&o), want(&["palace", "created"]));
}

#[test]
fn usage_errors_are_exit_3_on_stderr_with_empty_stdout() {
    let w = World::new("usage");
    for args in [
        vec!["--bogus"],
        vec!["bogus"],
        vec!["read"],
        vec!["reads", "--days", "x"],
        vec!["recall"],
        vec!["recall", "x", "--limit", "0"],
        vec!["check", "--scope", "galaxy"],
        vec!["check", "--as-of"],
        vec!["status", "--nope"],
        vec![],
    ] {
        let o = w.run(&args);
        assert_eq!(code(&o), 3, "{args:?}");
        assert!(o.stdout.is_empty(), "{args:?}: stdout must stay empty");
        assert!(!o.stderr.is_empty(), "{args:?}: stderr must say why");
    }
}

#[test]
fn help_and_version_are_exit_0_on_stdout() {
    let w = World::new("help");
    for args in [vec!["--help"], vec!["--version"], vec!["status", "--help"]] {
        let o = w.run(&args);
        assert_eq!(code(&o), 0, "{args:?}");
        assert!(!o.stdout.is_empty(), "{args:?}: stdout");
        assert!(o.stderr.is_empty(), "{args:?}: stderr");
    }
}

#[test]
fn not_found_is_exit_2_and_nothing_else_is() {
    let w = World::new("notfound");
    // No workfolder anywhere (empty cwd, no env). The walk climbs through the system temp dir, so
    // this assumes no ancestor of it is itself a workfolder.
    let o = w.run(&["status"]);
    assert_eq!(code(&o), 2);
    assert!(o.stdout.is_empty());
    // `check` needs a workfolder too: it is not a plain-folder command.
    let o = w.run(&["check"]);
    assert_eq!(code(&o), 2);
    assert!(o.stdout.is_empty());
    assert!(String::from_utf8_lossy(&o.stderr).starts_with("loci: "));
    // A path that is not a workfolder.
    assert_eq!(
        code(&w.run(&["--palace", "/nonexistent-loci-contract", "status"])),
        2
    );
    // No such crystal, room, or handover.
    assert_eq!(code(&w.with_palace(&["read", "no-such-slug"])), 2);
    assert_eq!(
        code(&w.with_palace(&["crystals", "--room", "no-such-room"])),
        2
    );
    // A directory with only a CLAUDE.md is not a workfolder (it has no room).
    let bare = scratch("notfound_bare");
    fs::write(bare.join("CLAUDE.md"), "# p\n").unwrap();
    let o = w.run(&["--palace", bare.to_str().unwrap(), "handover"]);
    let _ = fs::remove_dir_all(&bare);
    assert_eq!(code(&o), 2);
    assert!(String::from_utf8_lossy(&o.stderr).contains("no workfolder found"));
    // A real workfolder with no handover: its own error, still 2.
    fs::remove_dir_all(w.root.join("handovers")).unwrap();
    let o = w.with_palace(&["handover"]);
    assert_eq!(code(&o), 2);
    assert!(o.stdout.is_empty());
    let err = String::from_utf8_lossy(&o.stderr).to_lowercase();
    assert!(
        err.contains("handover") && !err.contains("no workfolder found"),
        "{err}"
    );
}

#[test]
fn errors_leave_stdout_empty_even_with_json() {
    let w = World::new("jsonerr");
    let o = w.with_palace(&["--json", "read", "no-such-slug"]);
    assert_eq!(code(&o), 2);
    assert!(
        o.stdout.is_empty(),
        "stdout: {}",
        String::from_utf8_lossy(&o.stdout)
    );
    assert!(String::from_utf8_lossy(&o.stderr).starts_with("loci: "));
}

#[test]
fn bad_input_that_is_not_a_usage_error_is_exit_3() {
    let w = World::new("badinput");
    // Removed flag.
    assert_eq!(code(&w.with_palace(&["rain", "--fire"])), 3);
    // `new` into a non-empty directory without --force.
    let target = w.cwd.join("busy");
    fs::create_dir_all(&target).unwrap();
    fs::write(target.join("f"), "x").unwrap();
    assert_eq!(code(&w.run(&["new", target.to_str().unwrap()])), 3);
}

#[test]
fn the_fixture_selftest_a_broken_expectation_fails() {
    // The key-set comparison must actually compare: a wrong key list fails.
    let w = World::new("selftest");
    let o = w.with_palace(&["--json", "handover"]);
    assert_ne!(keys(&o), want(&["path"]));
}

#[test]
fn a_bundle_that_does_not_verify_is_exit_3_with_the_verdict_on_stdout() {
    // The one documented exception to "errors leave stdout empty": the verdict is the output.
    let w = World::new("walfail");
    let bundle = w.cwd.join("bundle.json");
    fs::write(
        &bundle,
        r#"{"version":999,"sig_algorithm":"x","signer_pubkey":"","range":[0,0],"frames":[],"signature":""}"#,
    )
    .unwrap();
    let b = bundle.to_str().unwrap();
    let o = w.run(&["--json", "wal", "verify", b]);
    assert_eq!(code(&o), 3);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).expect("verdict is JSON");
    assert_eq!(v["result"], "fail");
    assert!(String::from_utf8_lossy(&o.stderr).starts_with("loci: "));
    // Human output: FAIL on stdout, same code.
    let o = w.run(&["wal", "verify", b]);
    assert_eq!(code(&o), 3);
    assert!(String::from_utf8_lossy(&o.stdout).starts_with("FAIL"));
    // A malformed bundle is exit 1 with nothing on stdout.
    fs::write(&bundle, "not json").unwrap();
    let o = w.run(&["wal", "verify", b]);
    assert_eq!(code(&o), 1);
    assert!(o.stdout.is_empty());
}

fn same(a: &Output, b: &Output, what: &str) {
    assert_eq!(code(a), code(b), "{what}: exit code");
    assert_eq!(a.stdout, b.stdout, "{what}: stdout");
    assert_eq!(a.stderr, b.stderr, "{what}: stderr");
}

#[test]
fn the_flag_and_the_variable_have_two_spellings_and_one_behaviour() {
    let w = World::new("aliases");
    let root = w.root.to_str().unwrap().to_string();
    // The flag: `--workfolder` and the old `--palace` read the same, and neither warns.
    for args in [
        vec!["--json", "status"],
        vec!["--json", "crystals"],
        vec!["--json", "read", "decision"],
        vec!["--json", "handover"],
        vec!["status"],
    ] {
        let mut new = vec!["--workfolder", root.as_str()];
        new.extend(&args);
        let mut old = vec!["--palace", root.as_str()];
        old.extend(&args);
        let (n, o) = (w.run(&new), w.run(&old));
        assert_eq!(code(&n), 0, "{args:?}");
        assert!(
            n.stderr.is_empty(),
            "{args:?}: stderr: {}",
            String::from_utf8_lossy(&n.stderr)
        );
        same(&n, &o, &format!("{args:?}"));
    }
    // The variable: `LOCI_WORKFOLDER` and the old `LOCI_PALACE`, from an empty directory.
    let args = ["--json", "status"];
    let n = w.run_env(&args, &[("LOCI_WORKFOLDER", &root)]);
    let o = w.run_env(&args, &[("LOCI_PALACE", &root)]);
    assert_eq!(code(&n), 0);
    assert!(n.stderr.is_empty() && o.stderr.is_empty());
    same(&n, &o, "env");
    // Both set: the new name wins; a bad old name is not even looked at.
    let b = w.run_env(
        &args,
        &[
            ("LOCI_WORKFOLDER", &root),
            ("LOCI_PALACE", "/nonexistent-loci-x"),
        ],
    );
    same(&n, &b, "both set");
    // A bad variable stops (exit 2) and names itself.
    for var in ["LOCI_WORKFOLDER", "LOCI_PALACE"] {
        let o = w.run_env(&args, &[(var, "/nonexistent-loci-x")]);
        assert_eq!(code(&o), 2);
        assert!(o.stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&o.stderr).contains(var),
            "names {var}"
        );
    }
}

#[test]
fn workfolder_md_marks_a_workfolder_and_reads_like_the_other_two() {
    let w = World::new("marker");
    let before = w.with_palace(&["--json", "status"]);
    assert_eq!(code(&before), 0);
    // Move whichever marker the fixture has to the new name.
    let old = ["PALACE.md", "CLAUDE.md"]
        .iter()
        .map(|m| w.root.join(m))
        .find(|p| p.exists())
        .expect("fixture has a root marker");
    fs::rename(&old, w.root.join("WORKFOLDER.md")).unwrap();
    let after = w.with_palace(&["--json", "status"]);
    assert_eq!(
        code(&after),
        0,
        "{}",
        String::from_utf8_lossy(&after.stderr)
    );
    same(&before, &after, "status after renaming the marker");
}

/// 2.0 speaks "workfolder". The old word survives only as a legacy spelling (`--palace`,
/// `LOCI_PALACE`, `PALACE.md`) and in kept names (the `palace-maintenance` skill, the
/// `palace-subdir` layout value), so no help text, overview or error may use it as prose.
#[test]
fn user_facing_text_says_workfolder_not_palace() {
    let w = World::new("words");
    let scrub = |s: &[u8]| {
        String::from_utf8_lossy(s)
            .replace("--palace", "")
            .replace("LOCI_PALACE", "")
            .replace("PALACE.md", "")
            .replace("palace-maintenance", "")
            .replace("palace-subdir", "")
            .to_lowercase()
    };
    let mut seen: Vec<(String, String)> = Vec::new();
    for cmd in [
        "", "status", "crystals", "read", "handover", "check", "tokens", "reads", "rain", "recall",
        "new", "init", "audit", "overview",
    ] {
        let mut args: Vec<&str> = Vec::new();
        if !cmd.is_empty() {
            args.push(cmd);
        }
        args.push("--help");
        let o = w.run(&args);
        seen.push((format!("{args:?}"), scrub(&o.stdout)));
    }
    let o = w.run(&["wal", "verify", "--help"]);
    seen.push(("wal verify --help".into(), scrub(&o.stdout)));
    seen.push((
        "overview".into(),
        scrub(&w.with_palace(&["overview"]).stdout),
    ));
    seen.push(("overview, none".into(), scrub(&w.run(&["overview"]).stdout)));
    seen.push(("status".into(), scrub(&w.with_palace(&["status"]).stdout)));
    seen.push(("reads".into(), scrub(&w.with_palace(&["reads"]).stdout)));
    for args in [
        vec!["crystals"],
        vec!["read", "decision"],
        vec!["rain"],
        vec!["handover"],
        vec!["check"],
    ] {
        let o = w.with_palace(&args);
        seen.push((format!("{args:?}"), scrub(&o.stdout)));
    }
    for args in [
        vec!["read", "no-such-slug"],
        vec!["crystals", "--room", "no-such-room"],
    ] {
        let o = w.with_palace(&args);
        seen.push((format!("{args:?} stderr"), scrub(&o.stderr)));
        // An error leaves stdout empty, so only its stderr is checked for words.
    }
    let fresh = w.cwd.join("words-fresh");
    let o = w.run(&["new", fresh.to_str().unwrap()]);
    seen.push(("new".into(), scrub(&o.stdout)));
    seen.push(("no workfolder".into(), scrub(&w.run(&["status"]).stderr)));
    seen.push((
        "bad variable".into(),
        scrub(
            &Command::new(BIN)
                .arg("status")
                .current_dir(&w.cwd)
                .env("HOME", &w.home)
                .env("LOCI_WORKFOLDER", "/nonexistent-loci-words")
                .env_remove("LOCI_PALACE")
                .output()
                .unwrap()
                .stderr,
        ),
    ));
    for (what, text) in seen {
        assert!(
            !text.contains("palace"),
            "{what}: still says palace:\n{text}"
        );
        assert!(!text.is_empty(), "{what}: nothing to check");
    }
}

/// One version line: `--version` names the format it reads, and the methodology anchor carries the
/// same major as the CLI, so the two cannot drift apart unseen. (The minor may differ: a CLI-only
/// release does not have to open a methodology section.)
#[test]
fn version_names_the_format_and_matches_the_methodology_anchor() {
    let w = World::new("version");
    let o = w.run(&["--version"]);
    assert_eq!(code(&o), 0);
    let pkg = env!("CARGO_PKG_VERSION");
    assert_eq!(
        String::from_utf8_lossy(&o.stdout),
        format!("loci {pkg} (loci format 2)\n")
    );
    let spec = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../templates/format-2.md"
    ))
    .expect("templates/format-2.md");
    assert!(
        spec.lines().any(|l| l.trim() == "version: 2.0"),
        "the spec the CLI names must say version 2.0"
    );
    let anchor = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../PALACE-METHODOLOGY.md"
    ))
    .expect("PALACE-METHODOLOGY.md");
    let line = anchor
        .lines()
        .find_map(|l| l.strip_prefix("> loci-core version:"))
        .expect("anchor line");
    let major = |v: &str| {
        v.trim()
            .split(|c: char| c == '.' || c == '-')
            .next()
            .unwrap()
            .to_string()
    };
    assert_eq!(major(line), major(pkg), "anchor {line} vs CLI {pkg}");
}

/// The exit-code table in CONTRACT.md names every code the binary can return (0 to 4), so a
/// script author finds "ran, found problems" (4) beside "could not run" (1).
#[test]
fn the_contract_names_every_exit_code() {
    let doc = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/CONTRACT.md"))
        .expect("CONTRACT.md");
    let table = doc
        .split("## Exit codes")
        .nth(1)
        .and_then(|t| t.split("\n## ").next())
        .expect("an Exit codes section");
    for c in 0..=4 {
        let row = table
            .lines()
            .find(|l| l.starts_with(&format!("| `{c}` |")))
            .unwrap_or_else(|| panic!("CONTRACT.md Exit codes has no row for {c}"));
        assert!(row.len() > 20, "the row for {c} says nothing: {row}");
        if c == 4 {
            assert!(row.contains("found problems"), "row 4 must say what 4 means");
        }
    }
}
