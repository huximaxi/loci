//! `loci check` against the real binary, on the three format 2 workfolders. The rules themselves
//! are tested where they land (each with its planted and clean fixture); this file holds what
//! every run must do whichever rules are registered: say how much it checked, exit 4 exactly when
//! it found a `fail` (a `warn` is printed and exits 0), never write, and give the same output
//! twice.

use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_loci");
const FORMAT2: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/format2");
const LAYOUTS: [&str; 3] = ["rooms-dir", "palace-subdir", "rooms-at-root"];

struct Scratch {
    base: PathBuf,
}

impl Scratch {
    fn new(name: &str) -> Scratch {
        let base = std::env::temp_dir().join(format!("loci_check_{}_{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join("cwd")).unwrap();
        fs::create_dir_all(base.join("home")).unwrap();
        Scratch { base }
    }

    /// A fixture copied into the scratch directory; `_palace.fixture` becomes `_palace`.
    fn copy(&self, layout: &str) -> PathBuf {
        let to = self.base.join(layout);
        copy_tree(&Path::new(FORMAT2).join(layout), &to);
        to
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(BIN)
            .args(args)
            .current_dir(self.base.join("cwd"))
            .env("HOME", self.base.join("home"))
            .env_remove("LOCI_PALACE")
            .env_remove("LOCI_WORKFOLDER")
            .env_remove("CLAUDE_CONFIG_DIR")
            .output()
            .expect("run loci")
    }

    fn check(&self, root: &Path, extra: &[&str]) -> Output {
        let mut args = vec!["--workfolder", root.to_str().unwrap(), "check"];
        args.extend_from_slice(extra);
        self.run(&args)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap().flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        let name = if name == "_palace.fixture" {
            "_palace".to_string()
        } else {
            name
        };
        let (src, dst) = (e.path(), to.join(&name));
        if src.is_dir() {
            copy_tree(&src, &dst);
        } else {
            fs::copy(&src, &dst).unwrap();
        }
    }
}

/// Every file under `root` with its bytes and modification time.
fn snapshot(root: &Path) -> BTreeMap<PathBuf, (Vec<u8>, std::time::SystemTime)> {
    fn walk(dir: &Path, out: &mut BTreeMap<PathBuf, (Vec<u8>, std::time::SystemTime)>) {
        for e in fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else {
                let t = fs::metadata(&p).unwrap().modified().unwrap();
                out.insert(p.clone(), (fs::read(&p).unwrap(), t));
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, &mut out);
    out
}

fn code(o: &Output) -> i32 {
    o.status.code().expect("exit code")
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

fn json(o: &Output) -> Value {
    serde_json::from_slice(&o.stdout).expect("one JSON document")
}

fn expected() -> Value {
    serde_json::from_str(&fs::read_to_string(Path::new(FORMAT2).join("expected.json")).unwrap())
        .unwrap()
}

#[test]
fn it_says_how_much_it_read_on_every_layout() {
    let s = Scratch::new("counts");
    let exp = expected();
    for layout in LAYOUTS {
        let root = s.copy(layout);
        let o = s.check(&root, &["--json"]);
        let v = json(&o);
        assert_eq!(v["layout"], layout, "{layout}");
        assert_eq!(
            v["crystals_considered"].as_u64().unwrap() as usize,
            exp[layout]["crystals"].as_array().unwrap().len(),
            "{layout}: crystals considered match what the reader reads"
        );
        assert_eq!(
            v["legacy_lines_not_counted"], exp[layout]["notes"],
            "{layout}"
        );
        assert_eq!(
            v["rules_run"].as_u64().unwrap() as usize,
            v["rules"].as_array().unwrap().len(),
            "{layout}: rules_run is the length of the rule list"
        );
        assert_eq!(
            v["files_unread"],
            serde_json::json!([]),
            "{layout}: a workfolder that reads whole says nothing was skipped"
        );
    }
}

#[test]
fn the_exit_code_is_four_exactly_when_a_finding_is_a_fail_and_zero_otherwise() {
    let s = Scratch::new("exit");
    for layout in LAYOUTS {
        let root = s.copy(layout);
        let o = s.check(&root, &["--json"]);
        let failed = json(&o)["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["severity"] == "fail");
        assert_eq!(
            code(&o),
            if failed { 4 } else { 0 },
            "{layout}: {}",
            stderr(&o)
        );
        assert!(
            stderr(&o).is_empty(),
            "{layout}: findings are output, not an error"
        );
    }
}

#[test]
fn the_text_never_stays_silent_about_what_it_checked() {
    let s = Scratch::new("text");
    let root = s.copy("rooms-dir");
    let o = s.check(&root, &[]);
    let t = String::from_utf8_lossy(&o.stdout).into_owned();
    assert!(t.contains(" crystals considered, "), "{t}");
    assert!(t.contains("rule"), "{t}");
    assert!(t.contains("A check reads; it never edits."), "{t}");
}

#[test]
fn a_check_never_writes_and_gives_the_same_output_twice() {
    let s = Scratch::new("readonly");
    for layout in LAYOUTS {
        let root = s.copy(layout);
        let before = snapshot(&root);
        let a = s.check(
            &root,
            &["--json", "--scope", "workfolder", "--as-of", "2027-01-02"],
        );
        let b = s.check(
            &root,
            &["--json", "--scope", "workfolder", "--as-of", "2027-01-02"],
        );
        assert_eq!(
            a.stdout, b.stdout,
            "{layout}: same files and flags, same bytes"
        );
        assert!(
            before == snapshot(&root),
            "{layout}: a check must not change a file or a modification time"
        );
    }
}

#[test]
fn the_flags_reach_the_output() {
    let s = Scratch::new("flags");
    let root = s.copy("rooms-dir");
    let v = json(&s.check(&root, &["--json"]));
    assert_eq!(v["scope"], "room", "room is the default scope");
    let today = v["as_of"].as_str().unwrap().to_string();
    assert_eq!(today.len(), 10, "default as_of is a day: {today}");
    assert!(
        today.as_str() >= "2026-11-26",
        "default as_of is today, not a fixed or epoch day: {today}"
    );
    let v = json(&s.check(
        &root,
        &["--json", "--scope", "workfolder", "--as-of", "2027-01-02"],
    ));
    assert_eq!(v["scope"], "workfolder");
    assert_eq!(v["as_of"], "2027-01-02");
}

#[test]
fn bad_input_is_exit_3_and_a_missing_workfolder_is_exit_2_with_empty_stdout() {
    let s = Scratch::new("errors");
    let root = s.copy("rooms-dir");
    for extra in [
        vec!["--as-of", "2027-02-30"],
        vec!["--as-of", "tomorrow"],
        vec!["--scope", "galaxy"],
        vec!["--as-of"],
    ] {
        let o = s.check(&root, &extra);
        assert_eq!(code(&o), 3, "{extra:?}");
        assert!(o.stdout.is_empty(), "{extra:?}: stdout stays empty");
        assert!(!o.stderr.is_empty(), "{extra:?}: stderr says why");
    }
    // A folder that is not a workfolder is not checked, with no fallback.
    let plain = s.base.join("plain");
    fs::create_dir_all(&plain).unwrap();
    fs::write(plain.join("note.md"), "# a note\n").unwrap();
    let o = s.run(&["--workfolder", plain.to_str().unwrap(), "check"]);
    assert_eq!(code(&o), 2);
    assert!(o.stdout.is_empty());
    assert!(stderr(&o).starts_with("loci: "), "{}", stderr(&o));
}

/// A workfolder with a root marker and one room, for the tests that break a file on purpose.
fn small_workfolder(s: &Scratch, name: &str) -> PathBuf {
    let root = s.base.join(name);
    fs::create_dir_all(root.join("rooms/work")).unwrap();
    fs::write(root.join("WORKFOLDER.md"), "# workfolder\n").unwrap();
    fs::write(root.join("rooms/work/CLAUDE.md"), "◆ fine = a fact\n").unwrap();
    root
}

/// Run `check`, but give up (and fail) after ten seconds: the thing under test is a hang.
fn check_within_ten_seconds(s: &Scratch, root: &Path) -> Output {
    let child = Command::new(BIN)
        .args(["--workfolder", root.to_str().unwrap(), "check", "--json"])
        .current_dir(s.base.join("cwd"))
        .env("HOME", s.base.join("home"))
        .env_remove("LOCI_PALACE")
        .env_remove("LOCI_WORKFOLDER")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn loci");
    let id = child.id();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(child.wait_with_output());
    });
    match rx.recv_timeout(std::time::Duration::from_secs(10)) {
        Ok(out) => out.expect("loci output"),
        Err(_) => {
            let _ = Command::new("kill").args(["-9", &id.to_string()]).status();
            panic!("loci check did not finish in ten seconds");
        }
    }
}

/// A file the reader cannot read must be said, not skipped in silence: "0 findings" over a file
/// nobody read is the failure this command exists to avoid. A FIFO is the case that also hangs
/// a reader that opens before it looks.
#[cfg(unix)]
#[test]
fn a_file_it_cannot_read_is_named_and_never_hangs_the_run() {
    let s = Scratch::new("fifo");
    let root = small_workfolder(&s, "w");
    let made = Command::new("mkfifo").arg(root.join("CLAUDE.md")).status();
    if !matches!(made, Ok(st) if st.success()) {
        return; // no mkfifo on this machine: nothing to prove
    }
    let o = check_within_ten_seconds(&s, &root);
    assert_eq!(code(&o), 0, "{}", stderr(&o));
    let v = json(&o);
    assert_eq!(v["files_unread"], serde_json::json!(["CLAUDE.md"]));
    assert_eq!(v["crystals_considered"], 1, "the rest was still read");
    let text = s.check(&root, &[]);
    let t = String::from_utf8_lossy(&text.stdout).into_owned();
    assert!(t.contains("1 file could not be read in full"), "{t}");
    assert!(t.contains("CLAUDE.md"), "{t}");
}

/// A container longer than the read cap is cut; the cut is said, never silent.
#[test]
fn a_container_cut_at_the_read_cap_is_named() {
    let s = Scratch::new("cap");
    let root = small_workfolder(&s, "w");
    fs::write(
        root.join("rooms/work/CLAUDE.md"),
        vec![b'x'; 9 * 1024 * 1024],
    )
    .unwrap();
    let v = json(&s.check(&root, &["--json"]));
    assert_eq!(
        v["files_unread"],
        serde_json::json!(["rooms/work/CLAUDE.md"])
    );
}
