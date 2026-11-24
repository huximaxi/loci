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
        Command::new(BIN)
            .args(args)
            .current_dir(&self.cwd)
            .env("HOME", &self.home)
            .env_remove("LOCI_PALACE")
            .env_remove("CLAUDE_CONFIG_DIR")
            .output()
            .expect("run loci")
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
    // No workfolder anywhere (empty cwd, no env).
    let o = w.run(&["status"]);
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
    let empty = scratch("notfound_empty");
    fs::write(empty.join("CLAUDE.md"), "# p\n").unwrap();
    let o = w.run(&["--palace", empty.to_str().unwrap(), "handover"]);
    let _ = fs::remove_dir_all(&empty);
    assert_eq!(code(&o), 2);
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
