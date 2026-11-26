//! `loci verify-cite` against the real binary, rows V-a to V-g of the grounding spec. Every
//! defect is planted one per file or line, every clean case has a mutation of a scratch copy
//! that must change the verdict, and the safety claims (nothing outside the root is opened or
//! named, a symlink is never followed) are asserted by comparing a real file with a missing one.
//! Fixtures are shared with `recall`: the format 2 workfolders and `grounding/setups/`.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_loci");
const FORMAT2: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/format2");
const SETUPS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/grounding/setups"
);

struct Scratch {
    base: PathBuf,
}

impl Scratch {
    fn new(name: &str) -> Scratch {
        let base =
            std::env::temp_dir().join(format!("loci_verify_{}_{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join("cwd")).unwrap();
        fs::create_dir_all(base.join("home")).unwrap();
        Scratch { base }
    }

    fn copy(&self, from: &str, as_name: &str) -> PathBuf {
        let to = self.base.join(as_name);
        copy_tree(Path::new(from), &to);
        to
    }

    /// An answer file in the scratch directory, outside any root under test.
    fn answer(&self, text: &str) -> PathBuf {
        let p = self.base.join("answer.md");
        fs::write(&p, text).unwrap();
        p
    }

    fn run_env(&self, args: &[&str], env: &[(&str, &str)], cwd: &Path) -> Output {
        let mut c = Command::new(BIN);
        c.args(args)
            .current_dir(cwd)
            .env("HOME", self.base.join("home"))
            .env_remove("LOCI_PALACE")
            .env_remove("LOCI_WORKFOLDER")
            .env_remove("CLAUDE_CONFIG_DIR");
        for (k, v) in env {
            c.env(k, v);
        }
        c.output().expect("run loci")
    }

    /// `loci verify-cite <answer> --root <root> --json`: exit code and parsed output.
    fn verify(&self, root: &Path, answer: &str) -> (i32, Value) {
        let a = self.answer(answer);
        let o = self.run_env(
            &[
                "verify-cite",
                a.to_str().unwrap(),
                "--root",
                root.to_str().unwrap(),
                "--json",
            ],
            &[],
            &self.base.join("cwd"),
        );
        let v = serde_json::from_slice(&o.stdout)
            .unwrap_or_else(|e| panic!("one JSON document ({e}): {}", stderr(&o)));
        (code(&o), v)
    }

    /// `verify-cite <answer> --root <root>` in text mode, killed and failed when it is still going
    /// after `secs` seconds. Output goes to a file, so a long listing cannot stall the child on a
    /// full pipe while this waits. Returns the exit code and the text.
    fn verify_bounded(&self, root: &Path, answer: &str, secs: u64) -> (Option<i32>, String) {
        let a = self.answer(answer);
        let out = self.base.join("bounded.out");
        let mut child = Command::new(BIN)
            .args([
                "verify-cite",
                a.to_str().unwrap(),
                "--root",
                root.to_str().unwrap(),
            ])
            .current_dir(self.base.join("cwd"))
            .env("HOME", self.base.join("home"))
            .env_remove("LOCI_PALACE")
            .env_remove("LOCI_WORKFOLDER")
            .stdout(std::process::Stdio::from(fs::File::create(&out).unwrap()))
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("spawn loci");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(secs);
        loop {
            if let Some(st) = child.try_wait().unwrap() {
                let text = String::from_utf8_lossy(&fs::read(&out).unwrap()).to_string();
                return (st.code(), text);
            }
            if std::time::Instant::now() > deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("verify-cite was still running after {secs} seconds");
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
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
        let dst = to.join(name);
        if e.file_type().unwrap().is_dir() {
            copy_tree(&e.path(), &dst);
        } else {
            fs::copy(e.path(), &dst).unwrap();
        }
    }
}

fn code(o: &Output) -> i32 {
    o.status.code().expect("exit code")
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

fn fixture(name: &str) -> String {
    format!("{FORMAT2}/{name}")
}

fn setup(name: &str) -> String {
    format!("{SETUPS}/{name}")
}

/// The statuses, in the order the cites appear in the answer.
fn statuses(v: &Value) -> Vec<String> {
    v["cites"]
        .as_array()
        .expect("cites array")
        .iter()
        .map(|c| c["status"].as_str().unwrap().to_string())
        .collect()
}

// ── V-a: setup a, one defect per cite ──────────────────────────────────────

const GOOD_A: &str =
    "We chose Postgres (rooms/work/decision.md:9 \"We use Postgres for the store.\") \
and the standup is daily (CLAUDE.md:8 \"Standup at 10:30 daily\").\n";

#[test]
fn a_grounded_answer_exits_zero_and_every_cite_is_ok() {
    let s = Scratch::new("good");
    let root = s.copy(&fixture("rooms-dir"), "rooms-dir");
    let (c, v) = s.verify(&root, GOOD_A);
    assert_eq!(c, 0);
    assert_eq!(statuses(&v), ["ok", "ok"]);
    assert_eq!(v["counts"]["total"], 2);
    assert_eq!(v["counts"]["ok"], 2);
    assert_eq!(v["counts"]["ungrounded"], 0);
    assert_eq!(v["verified"], 2);
    assert_eq!(v["scope"]["roots"][0]["layout"], "rooms-dir");
    let first = &v["cites"][0];
    assert_eq!(first["style"], "S1");
    assert_eq!(first["path"], "rooms/work/decision.md");
    assert_eq!(
        (first["line"].as_u64(), first["end_line"].as_u64()),
        (Some(9), Some(9))
    );
}

#[test]
fn the_committed_answer_fixture_holds_and_a_changed_copy_does_not() {
    let s = Scratch::new("fixture");
    let root = s.copy(&fixture("rooms-dir"), "rooms-dir");
    let good = fs::read_to_string(format!("{SETUPS}/../answers/rooms-dir-good.md")).unwrap();
    assert_eq!(s.verify(&root, &good).0, 0);
    let (c, v) = s.verify(&root, &good.replace("Postgres", "Oracle"));
    assert_eq!(
        (c, statuses(&v)),
        (4, vec!["claim_not_found".to_string(), "ok".to_string()])
    );
}

#[test]
fn each_defect_has_its_own_status_and_exits_four() {
    let s = Scratch::new("defects");
    let root = s.copy(&fixture("rooms-dir"), "rooms-dir");
    let cases = [
        ("rooms/work/missing.md:1 \"x\"", "no_file"),
        ("rooms/work/decision.md:99 \"x\"", "line_out_of_range"),
        (
            "rooms/work/decision.md:9 \"We use MySQL\"",
            "claim_not_found",
        ),
        ("../outside.md:1 \"x\"", "outside_workfolder"),
        ("rooms/work/decision.md:9", "unclaimed"),
        ("rooms/work/CLAUDE.md:3-1 \"x\"", "bad_range"),
        ("rooms/work/decision.md:0 \"x\"", "bad_range"),
    ];
    for (cite, want) in cases {
        let (c, v) = s.verify(&root, &format!("{cite}\n"));
        assert_eq!(statuses(&v), [want], "{cite}");
        assert_eq!(c, 4, "{cite}");
        assert_eq!(v["counts"]["ungrounded"], 1, "{cite}");
    }
}

#[test]
fn a_cite_inside_a_superseded_crystal_is_ok_with_a_stale_warning_and_exits_zero() {
    let s = Scratch::new("stale");
    let root = s.copy(&fixture("rooms-dir"), "rooms-dir");
    let (c, v) = s.verify(
        &root,
        "rooms/work/CLAUDE.md:10 \"Framing: plain labels win\"\n",
    );
    assert_eq!(c, 0);
    assert_eq!(statuses(&v), ["ok"]);
    assert_eq!(v["cites"][0]["warning"], "stale: superseded");

    // Mutation: the crystal no longer says it was superseded, so the warning goes.
    let f = root.join("rooms/work/CLAUDE.md");
    let t = fs::read_to_string(&f).unwrap();
    assert!(t.contains("superseded_by: Framing v2"));
    fs::write(&f, t.replace("superseded_by: Framing v2\n", "")).unwrap();
    let (c, v) = s.verify(
        &root,
        "rooms/work/CLAUDE.md:10 \"Framing: plain labels win\"\n",
    );
    assert_eq!(c, 0);
    assert_eq!(statuses(&v), ["ok"]);
    assert!(v["cites"][0]["warning"].is_null(), "{v}");
}

#[test]
fn times_and_urls_make_no_cites_and_a_zero_cite_file_exits_zero() {
    let s = Scratch::new("zero");
    let root = s.copy(&fixture("rooms-dir"), "rooms-dir");
    let (c, v) = s.verify(
        &root,
        "Standup at 10:30 daily, see https://example.com/notes/a.md:3 \"x\" and v2:1.\n",
    );
    assert_eq!(c, 0);
    assert_eq!(v["counts"]["total"], 0);
    assert_eq!(v["verified"], 0);

    let a = s.answer("nothing to cite here\n");
    let o = s.run_env(
        &[
            "verify-cite",
            a.to_str().unwrap(),
            "--root",
            root.to_str().unwrap(),
        ],
        &[],
        &s.base.join("cwd"),
    );
    assert_eq!(code(&o), 0);
    let out = String::from_utf8_lossy(&o.stdout).to_string();
    assert!(out.contains("0 cites found: nothing was verified"), "{out}");
}

#[test]
fn a_claim_that_matches_changes_when_the_file_changes() {
    let s = Scratch::new("mutate");
    let root = s.copy(&fixture("rooms-dir"), "rooms-dir");
    let ans = "rooms/work/decision.md:9 \"We use Postgres for the store.\"\n";
    assert_eq!(s.verify(&root, ans).0, 0);
    let f = root.join("rooms/work/decision.md");
    let t = fs::read_to_string(&f).unwrap();
    fs::write(&f, t.replace("Postgres", "MySQL")).unwrap();
    let (c, v) = s.verify(&root, ans);
    assert_eq!((c, statuses(&v)), (4, vec!["claim_not_found".to_string()]));
}

#[test]
fn text_output_ends_with_the_presence_not_support_line_and_the_scope_line() {
    let s = Scratch::new("text");
    let root = s.copy(&fixture("rooms-dir"), "rooms-dir");
    let a = s.answer(GOOD_A);
    let o = s.run_env(
        &[
            "verify-cite",
            a.to_str().unwrap(),
            "--root",
            root.to_str().unwrap(),
        ],
        &[],
        &s.base.join("cwd"),
    );
    let out = String::from_utf8_lossy(&o.stdout).to_string();
    let lines: Vec<&str> = out.lines().collect();
    let n = lines.len();
    assert!(
        lines[n - 3].contains("2 cites: 2 ok, 0 ungrounded"),
        "{out}"
    );
    assert_eq!(
        lines[n - 2],
        "this checks that quoted strings are present, not that they support the answer"
    );
    assert!(lines[n - 1].starts_with("scope: 2 files read"), "{out}");
    assert!(
        stderr(&o).is_empty(),
        "findings are output, not an error: {}",
        stderr(&o)
    );
}

#[test]
fn the_output_is_the_same_every_run() {
    let s = Scratch::new("det");
    let root = s.copy(&fixture("rooms-dir"), "rooms-dir");
    let a = s.verify(&root, GOOD_A).1;
    let b = s.verify(&root, GOOD_A).1;
    assert_eq!(a, b);
}

// ── V-b: the root ───────────────────────────────────────────────────────────

#[test]
fn the_root_comes_from_the_environment_and_a_different_root_changes_the_verdict() {
    let s = Scratch::new("env");
    let root = s.copy(&fixture("rooms-dir"), "rooms-dir");
    let other = s.copy(&setup("c"), "other");
    let a = s.answer(GOOD_A);
    let args = ["verify-cite", a.to_str().unwrap()];
    let via_env = s.run_env(
        &args,
        &[("LOCI_WORKFOLDER", root.to_str().unwrap())],
        &s.base.join("cwd"),
    );
    assert_eq!(code(&via_env), 0, "{}", stderr(&via_env));
    let via_flag = s.run_env(
        &[
            "verify-cite",
            a.to_str().unwrap(),
            "--root",
            root.to_str().unwrap(),
        ],
        &[],
        &s.base.join("cwd"),
    );
    assert_eq!(via_env.stdout, via_flag.stdout);

    let elsewhere = s.run_env(
        &args,
        &[("LOCI_WORKFOLDER", other.to_str().unwrap())],
        &s.base.join("cwd"),
    );
    assert_eq!(code(&elsewhere), 4);
    assert!(String::from_utf8_lossy(&elsewhere.stdout).contains("no_file"));
}

#[test]
fn no_root_exits_two_and_bad_input_exits_three() {
    let s = Scratch::new("usage");
    let a = s.answer(GOOD_A);
    let o = s.run_env(
        &["verify-cite", a.to_str().unwrap()],
        &[],
        &s.base.join("cwd"),
    );
    assert_eq!(code(&o), 2, "{}", stderr(&o));

    let root = s.copy(&fixture("rooms-dir"), "rooms-dir");
    let r = root.to_str().unwrap();
    let missing = s.base.join("nope.md");
    let o = s.run_env(
        &["verify-cite", missing.to_str().unwrap(), "--root", r],
        &[],
        &s.base.join("cwd"),
    );
    assert_eq!(code(&o), 3, "{}", stderr(&o));
    let o = s.run_env(
        &["verify-cite", s.base.to_str().unwrap(), "--root", r],
        &[],
        &s.base.join("cwd"),
    );
    assert_eq!(code(&o), 3, "a directory is not an answer file");
    fs::write(s.base.join("bin.md"), [0xff, 0xfe, 0x00, 0x80]).unwrap();
    let o = s.run_env(
        &[
            "verify-cite",
            s.base.join("bin.md").to_str().unwrap(),
            "--root",
            r,
        ],
        &[],
        &s.base.join("cwd"),
    );
    assert_eq!(code(&o), 3, "not UTF-8");
    let o = s.run_env(&["verify-cite"], &[], &s.base.join("cwd"));
    assert_eq!(code(&o), 3, "no argument");
}

// ── V-c: plain notes, unreadable files ─────────────────────────────────────

#[test]
fn a_plain_note_cite_is_ok_and_an_unreadable_file_is_counted_not_read() {
    let s = Scratch::new("plain");
    let root = s.copy(&setup("c"), "notes-root");
    let (c, v) = s.verify(
        &root,
        "notes/alpha.md:3 \"The kettle boils at the top of the hour.\"\n",
    );
    assert_eq!((c, statuses(&v)), (0, vec!["ok".to_string()]));
    assert_eq!(v["scope"]["roots"][0]["kind"], "plain");

    fs::write(root.join("notes/bad.md"), [b'a', b'\n', 0xff, 0xfe, b'\n']).unwrap();
    let big = vec![b'x'; 1024 * 1024 + 10];
    fs::write(root.join("notes/big.md"), big).unwrap();
    let (c, v) = s.verify(&root, "notes/bad.md:1 \"a\" notes/big.md:1 \"x\"\n");
    assert_eq!(c, 4);
    assert_eq!(statuses(&v), ["unreadable", "unreadable"]);
    assert_eq!(v["scope"]["not_read"]["total"], 2);
    assert_eq!(v["scope"]["not_read"]["not_utf8"], 1);
    assert_eq!(v["scope"]["not_read"]["over_cap"], 1);
}

// ── V-d: an Obsidian-style vault ───────────────────────────────────────────

#[test]
fn spaces_end_a_path_and_a_hidden_directory_is_not_in_the_corpus() {
    let s = Scratch::new("vault");
    let root = s.copy(&setup("d"), "vault-root").join("vault");
    let (c, v) = s.verify(&root, "Other Note.md:3 \"Nothing about the vault here.\"\n");
    // The path stops at the space, so this names a file called `Note.md`, which is not there.
    assert_eq!((c, statuses(&v)), (4, vec!["no_file".to_string()]));
    assert_eq!(v["cites"][0]["path"], "Note.md");

    let (_, v) = s.verify(
        &root,
        "[Note](Daily Notes/My Note.md#L5 \"My Note\") .obsidian/app.json:1 \"x\"\n",
    );
    assert_eq!(statuses(&v), ["no_file", "not_in_corpus"]);
    assert_eq!(v["cites"][0]["path"], "Note.md");

    // The whole path, with no space in it, is found.
    fs::create_dir_all(root.join("daily")).unwrap();
    fs::copy(
        root.join("Daily Notes/My Note.md"),
        root.join("daily/My_Note.md"),
    )
    .unwrap();
    let (c, v) = s.verify(&root, "[Note](daily/My_Note.md#L5 \"# My Note\")\n");
    assert_eq!((c, statuses(&v)), (0, vec!["ok".to_string()]));
    assert_eq!(v["cites"][0]["style"], "S3");
}

// ── V-e: a memory folder with an index ─────────────────────────────────────

#[test]
fn a_link_anchor_verifies_and_a_non_markdown_file_is_not_in_the_corpus() {
    let s = Scratch::new("mem");
    let root = s.copy(&setup("e"), "mem-root").join("mem");
    fs::create_dir_all(root.join("old")).unwrap();
    fs::copy(root.join("x.bak"), root.join("old/x.bak")).unwrap();
    let (c, v) = s.verify(
        &root,
        "[Harbour map](a.md#L8 \"Pier nine holds the long boats.\")\nMEMORY.md:1 \"Harbour map\"\nold/x.bak:1 \"x\"\n",
    );
    assert_eq!(c, 4);
    assert_eq!(statuses(&v), ["ok", "ok", "not_in_corpus"]);
}

// ── V-g: no Unicode normalisation ──────────────────────────────────────────

#[test]
fn a_decomposed_accent_does_not_match_a_precomposed_one() {
    let s = Scratch::new("uni");
    let root = s.copy(&setup("g"), "notes-root");
    let (c, v) = s.verify(&root, "notes/hu.md:3 \"árvíztűrő\"\n");
    assert_eq!((c, statuses(&v)), (0, vec!["ok".to_string()]));
    // The same word with every accented letter written as a base letter and a combining mark.
    let nfd = "a\u{301}rvi\u{301}ztu\u{30b}ro\u{30b}";
    let (c, v) = s.verify(&root, &format!("notes/hu.md:3 \"{nfd}\"\n"));
    assert_eq!((c, statuses(&v)), (4, vec!["claim_not_found".to_string()]));
}

#[test]
fn matching_collapses_whitespace_and_joins_the_cited_lines_but_stays_case_sensitive() {
    let s = Scratch::new("ws");
    let root = s.copy(&setup("c"), "notes-root");
    fs::write(
        root.join("notes/wrap.md"),
        "first line of it\n   second   line\n",
    )
    .unwrap();
    let (c, v) = s.verify(&root, "notes/wrap.md:1-2 \"of it second line\"\n");
    assert_eq!((c, statuses(&v)), (0, vec!["ok".to_string()]));
    let (_, v) = s.verify(&root, "notes/wrap.md:1 \"of it second line\"\n");
    assert_eq!(statuses(&v), ["claim_not_found"]);
    let (_, v) = s.verify(&root, "notes/wrap.md:1-2 \"Of it second line\"\n");
    assert_eq!(statuses(&v), ["claim_not_found"]);
}

// ── safety: the command is not an oracle for what lies outside the corpus ──

#[test]
fn nothing_outside_the_root_is_opened_or_told_apart() {
    let s = Scratch::new("oracle");
    let root = s.copy(&setup("c"), "notes-root");
    let verdict = |s: &Scratch, ans: &str| statuses(&s.verify(&root, ans).1);

    // A file next to the root that exists, and one that does not: the same answer.
    fs::write(s.base.join("secret.md"), "the key is 1234\n").unwrap();
    let real = verdict(&s, "../secret.md:1 \"the key is 1234\"\n");
    let none = verdict(&s, "../nothing.md:1 \"the key is 1234\"\n");
    assert_eq!(real, ["outside_workfolder"]);
    assert_eq!(real, none);

    // An absolute path outside the root is the same.
    let abs = format!("{}/secret.md:1 \"the key is 1234\"\n", s.base.display());
    assert_eq!(verdict(&s, &abs), ["outside_workfolder"]);
    // An absolute path inside the root is read as the relative one.
    let inside = format!("{}/notes/alpha.md:3 \"kettle\"\n", root.display());
    assert_eq!(verdict(&s, &inside), ["ok"]);

    // A file under the root that is not markdown, present or absent: the same answer.
    fs::create_dir_all(root.join("config")).unwrap();
    fs::write(root.join("config/.env"), "KEY=abc\n").unwrap();
    let real = verdict(&s, "config/.env:1 \"KEY=abc\"\n");
    let none = verdict(&s, "config/.nope:1 \"KEY=abc\"\n");
    assert_eq!(real, ["not_in_corpus"]);
    assert_eq!(real, none);
    // Same for a markdown file inside a hidden directory.
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::write(root.join(".git/n.md"), "hidden\n").unwrap();
    assert_eq!(verdict(&s, ".git/n.md:1 \"hidden\"\n"), ["not_in_corpus"]);
}

#[cfg(unix)]
#[test]
fn a_symlink_is_never_followed() {
    let s = Scratch::new("link");
    let root = s.copy(&setup("c"), "notes-root");
    fs::write(s.base.join("elsewhere.md"), "outside text\n").unwrap();
    std::os::unix::fs::symlink(s.base.join("elsewhere.md"), root.join("notes/link.md")).unwrap();
    std::os::unix::fs::symlink(root.join("notes/alpha.md"), root.join("notes/inside.md")).unwrap();
    std::os::unix::fs::symlink(s.base.join("cwd"), root.join("notes/dir")).unwrap();
    fs::write(s.base.join("cwd/e.md"), "via a linked directory\n").unwrap();
    let (c, v) = s.verify(
        &root,
        "notes/link.md:1 \"outside text\" notes/inside.md:3 \"kettle\" notes/dir/e.md:1 \"via\"\n",
    );
    assert_eq!(c, 4);
    assert_eq!(
        statuses(&v),
        ["not_in_corpus", "not_in_corpus", "not_in_corpus"]
    );
}

#[test]
fn control_bytes_in_the_answer_never_reach_the_terminal() {
    let s = Scratch::new("ctl");
    let root = s.copy(&setup("c"), "notes-root");
    let a = s.answer("notes/alpha.md:3 \"\u{1b}[31mred\u{1b}[0m \u{202e}\"\n");
    let o = s.run_env(
        &[
            "verify-cite",
            a.to_str().unwrap(),
            "--root",
            root.to_str().unwrap(),
        ],
        &[],
        &s.base.join("cwd"),
    );
    assert_eq!(code(&o), 4);
    assert!(
        !o.stdout.contains(&0x1b),
        "{:?}",
        String::from_utf8_lossy(&o.stdout)
    );
    assert!(!String::from_utf8_lossy(&o.stdout).contains('\u{202e}'));
}

// ── review findings: stale ranges, case, hostile input ─────────────────────

#[test]
fn a_retired_crystal_warns_and_so_does_a_range_that_spills_past_a_stale_one() {
    let s = Scratch::new("stale2");
    let root = s.copy(&fixture("rooms-dir"), "rooms-dir");
    let warn = |ans: &str| {
        let (c, v) = s.verify(&root, ans);
        assert_eq!(c, 0, "{ans}");
        assert_eq!(statuses(&v), ["ok"], "{ans}");
        v["cites"][0]["warning"].clone()
    };
    assert_eq!(
        warn("rooms/work/CLAUDE.md:13 \"Old rhythm\"\n"),
        "stale: retired"
    );
    assert_eq!(
        warn("rooms/work/CLAUDE.md:10 \"plain labels win\"\n"),
        "stale: superseded"
    );
    // The range starts above the crystal and ends inside it, or covers it whole.
    assert_eq!(
        warn("rooms/work/CLAUDE.md:9-10 \"plain labels win\"\n"),
        "stale: superseded"
    );
    assert_eq!(
        warn("rooms/work/CLAUDE.md:10-12 \"plain labels win\"\n"),
        "stale: superseded"
    );
    // Line 12 is the current replacement: no warning.
    assert!(warn("rooms/work/CLAUDE.md:12 \"in every segment\"\n").is_null());
}

#[test]
fn a_cite_that_spells_the_case_differently_is_read_as_the_file_it_reaches() {
    let s = Scratch::new("case");
    let root = s.copy(&fixture("rooms-dir"), "rooms-dir");
    let insensitive = {
        let probe = root.join("ROOMS");
        probe.exists()
    };
    // A skipped directory is not in the corpus however it is spelled, present or not.
    for dir in ["cron", "Cron", "CRON"] {
        let (c, v) = s.verify(&root, &format!("rooms/{dir}/CLAUDE.md:1 \"x\"\n"));
        assert_eq!(
            (c, statuses(&v)),
            (4, vec!["not_in_corpus".to_string()]),
            "{dir}"
        );
        let (_, v) = s.verify(&root, &format!("rooms/{dir}/NOPE.md:1 \"x\"\n"));
        assert_eq!(statuses(&v), ["not_in_corpus"], "{dir} missing file");
    }
    let (_, v) = s.verify(&root, "Node_Modules/p.md:1 \"x\"\n");
    assert_eq!(statuses(&v), ["not_in_corpus"]);
    if insensitive {
        // On a case-insensitive disk the wrong spelling reaches the same file: same crystals,
        // same warning, and one file read, not two.
        let (_, v) = s.verify(
            &root,
            "ROOMS/work/CLAUDE.md:10 \"plain labels win\"\nrooms/work/CLAUDE.md:10 \"plain labels win\"\n",
        );
        assert_eq!(v["cites"][0]["warning"], "stale: superseded");
        assert_eq!(v["cites"][1]["warning"], "stale: superseded");
        assert_eq!(v["scope"]["files_read"], 1);
    }
}

#[test]
fn a_file_named_just_md_is_not_in_the_corpus() {
    let s = Scratch::new("dotmd");
    let root = s.copy(&setup("c"), "notes-root");
    fs::write(root.join("notes/.md"), "hidden dotmd\n").unwrap();
    let (_, v) = s.verify(&root, "notes/.md:1 \"hidden dotmd\"\n");
    assert_eq!(statuses(&v), ["not_in_corpus"]);
}

#[test]
fn a_hostile_answer_costs_time_and_output_in_proportion_to_its_size() {
    let s = Scratch::new("dos");
    let root = s.copy(&setup("c"), "notes-root");
    // 60 KB of anchors with no path: no cites at all, and quickly.
    let t = std::time::Instant::now();
    let (c, v) = s.verify(&root, &format!("{}\n", "#L1".repeat(20_000)));
    assert_eq!((c, v["counts"]["total"].as_u64()), (0, Some(0)));
    // A long run of path characters ending in an anchor: refused as a path, not echoed whole.
    let long = format!("{}/x.md#L1 \"q\"\n", "a/".repeat(5_000));
    let (_, v) = s.verify(&root, &long);
    assert_eq!(v["counts"]["total"], 0);
    // Many short cites: one entry each, and the output stays small.
    let many = "notes/alpha.md#L3 \"kettle\" ".repeat(2_000);
    let (c, v) = s.verify(&root, &many);
    assert_eq!((c, v["counts"]["total"].as_u64()), (0, Some(2_000)));
    assert!(t.elapsed().as_secs() < 20, "took {:?}", t.elapsed());
    let a = s.answer(&format!("a/{}\n", "#L1".repeat(2_000)));
    let o = s.run_env(
        &[
            "verify-cite",
            a.to_str().unwrap(),
            "--root",
            root.to_str().unwrap(),
        ],
        &[],
        &s.base.join("cwd"),
    );
    assert!(
        o.stdout.len() < 10_000,
        "{} bytes out for 6 KB in",
        o.stdout.len()
    );
}

#[test]
fn arithmetic_with_slashes_is_not_a_cite_and_a_link_to_a_real_file_is() {
    let s = Scratch::new("noise");
    let root = s.copy(&setup("c"), "notes-root");
    let (c, v) = s.verify(&root, "ratio 3/4:5 and 2026/11/26:15\n");
    assert_eq!((c, v["counts"]["total"].as_u64()), (0, Some(0)));
    let (c, v) = s.verify(&root, "[Alpha](notes/alpha.md#L3 \"kettle\")\n");
    assert_eq!((c, statuses(&v)), (0, vec!["ok".to_string()]));
}

// ── the round trip with recall, and the edges of the answer file ───────────

#[test]
fn a_hidden_markdown_file_verifies_as_recall_reads_it_and_a_hidden_directory_does_not() {
    let s = Scratch::new("hidden");
    let root = s.copy(&setup("c"), "notes-root");
    fs::write(root.join("notes/.draft.md"), "kept in the draft\n").unwrap();
    fs::create_dir_all(root.join("notes/.stash")).unwrap();
    fs::write(root.join("notes/.stash/kept.md"), "kept in the stash\n").unwrap();
    // recall reads hidden files (it skips hidden directories), so the cite it prints must verify.
    let o = s.run_env(
        &["recall", "draft", "--root", root.to_str().unwrap()],
        &[],
        &s.base.join("cwd"),
    );
    let printed = String::from_utf8_lossy(&o.stdout).to_string();
    assert!(printed.contains("cite: notes/.draft.md:1"), "{printed}");
    let (c, v) = s.verify(
        &root,
        "notes/.draft.md:1 \"kept in the draft\"\nnotes/.stash/kept.md:1 \"kept in the stash\"\n",
    );
    assert_eq!(statuses(&v), ["ok", "not_in_corpus"]);
    assert_eq!(c, 4);
}

#[test]
fn an_empty_or_blank_claim_is_unclaimed_never_ok() {
    let s = Scratch::new("emptyclaim");
    let root = s.copy(&setup("c"), "notes-root");
    let (c, v) = s.verify(
        &root,
        "notes/alpha.md:3 \"\"\nnotes/alpha.md:3 \"   \"\nnotes/alpha.md:3 “”\n",
    );
    assert_eq!(statuses(&v), ["unclaimed", "unclaimed", "unclaimed"]);
    assert_eq!(c, 4);
}

#[test]
fn a_byte_order_mark_on_the_answer_does_not_change_its_first_cite() {
    let s = Scratch::new("bom");
    let root = s.copy(&setup("c"), "notes-root");
    let a = s.base.join("bom-answer.md");
    let mut bytes = vec![0xEF, 0xBB, 0xBF];
    bytes.extend_from_slice(b"notes/alpha.md:3 \"kettle\"\n");
    fs::write(&a, bytes).unwrap();
    let o = s.run_env(
        &[
            "verify-cite",
            a.to_str().unwrap(),
            "--root",
            root.to_str().unwrap(),
            "--json",
        ],
        &[],
        &s.base.join("cwd"),
    );
    let v: Value = serde_json::from_slice(&o.stdout).expect("one JSON document");
    assert_eq!((code(&o), statuses(&v)), (0, vec!["ok".to_string()]));
    assert_eq!(v["cites"][0]["path"], "notes/alpha.md");
}

#[test]
fn an_answer_of_exactly_one_mebibyte_is_read_and_one_byte_more_is_refused() {
    let s = Scratch::new("anscap");
    let root = s.copy(&setup("c"), "notes-root");
    let a = s.base.join("big-answer.md");
    let run = |s: &Scratch| {
        s.run_env(
            &[
                "verify-cite",
                a.to_str().unwrap(),
                "--root",
                root.to_str().unwrap(),
            ],
            &[],
            &s.base.join("cwd"),
        )
    };
    fs::write(&a, vec![b'x'; 1024 * 1024]).unwrap();
    let o = run(&s);
    assert_eq!(code(&o), 0, "{}", stderr(&o));
    fs::write(&a, vec![b'x'; 1024 * 1024 + 1]).unwrap();
    let o = run(&s);
    assert_eq!(code(&o), 3);
    assert!(o.stdout.is_empty());
    assert!(stderr(&o).contains("over 1 MiB"), "{}", stderr(&o));
}

#[test]
fn a_directory_or_a_pipe_named_like_a_note_is_no_file_and_never_opened() {
    let s = Scratch::new("odd");
    let root = s.copy(&setup("c"), "notes-root");
    fs::create_dir_all(root.join("notes/folder.md")).unwrap();
    let mut cites = String::from("notes/folder.md:1 \"x\"\n");
    #[cfg(unix)]
    {
        let made = Command::new("mkfifo")
            .arg(root.join("notes/pipe.md"))
            .status();
        if matches!(made, Ok(st) if st.success()) {
            cites.push_str("notes/pipe.md:1 \"x\"\n");
        }
    }
    let (code, text) = s.verify_bounded(&root, &cites, 10);
    assert_eq!(code, Some(4));
    assert_eq!(
        text.lines().filter(|l| l.starts_with("no_file")).count(),
        cites.lines().count(),
        "{text}"
    );
}

#[test]
fn an_answer_built_to_be_slow_is_not_slow() {
    let s = Scratch::new("slow");
    let root = s.copy(&setup("c"), "notes-root");
    // A file of 90,000 short lines, cited by 6,000 different ranges that each reach almost all of
    // it, quoting a string that is not there. The text of those lines used to be rebuilt per cite.
    let big: String = (0..90_000).map(|i| format!("line {i}\n")).collect();
    fs::write(root.join("notes/big.md"), big).unwrap();
    let wide: String = (0..6_000)
        .map(|i| format!("notes/big.md:1-{} \"zzz\"\n", 89_000 + i % 1_000))
        .collect();
    let (code, text) = s.verify_bounded(&root, &wide, 30);
    assert_eq!(code, Some(4));
    assert!(text.contains("6000 cites: 0 ok, 6000 ungrounded"), "{text}");
    // 50,000 openers on one line with no closer: each used to scan to the end of the line.
    let openers = format!("{}\n", "notes/alpha.md:3 “".repeat(50_000));
    let (code, text) = s.verify_bounded(&root, &openers, 30);
    assert_eq!(code, Some(4));
    assert!(
        text.contains("50000 cites: 0 ok, 50000 ungrounded"),
        "{}",
        &text[text.len().saturating_sub(300)..]
    );
}
