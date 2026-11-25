//! `loci recall` against the real binary: one planted positive and one clean case per setup,
//! each with a mutation of a scratch copy that must change the verdict, so a search that always
//! answers the same thing would fail. Fixtures: the format 2 workfolders for setup a, and
//! `tests/fixtures/grounding/setups/` for the plain folders (c, d, e, g). Files that cannot be
//! committed sensibly (invalid UTF-8, one over the size cap, a symlink) are made at run time.

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
            std::env::temp_dir().join(format!("loci_recall_{}_{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join("cwd")).unwrap();
        fs::create_dir_all(base.join("home")).unwrap();
        Scratch { base }
    }

    fn cwd(&self) -> PathBuf {
        self.base.join("cwd")
    }

    /// A fixture copied into the scratch directory; `_palace.fixture` becomes `_palace`.
    fn copy(&self, from: &str, as_name: &str) -> PathBuf {
        let to = self.base.join(as_name);
        copy_tree(Path::new(from), &to);
        to
    }

    fn run(&self, args: &[&str]) -> Output {
        self.run_env(args, &[], &self.cwd())
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

    /// `loci recall <query> --root <root> --json ...`, parsed.
    fn recall(&self, root: &Path, query: &str, extra: &[&str]) -> Value {
        let mut args = vec!["recall", query, "--root", root.to_str().unwrap(), "--json"];
        args.extend_from_slice(extra);
        let o = self.run(&args);
        assert_eq!(code(&o), 0, "recall {query:?} failed: {}", stderr(&o));
        serde_json::from_slice(&o.stdout).expect("one JSON document")
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

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

fn results(v: &Value) -> &Vec<Value> {
    v["results"].as_array().expect("results array")
}

fn cites(v: &Value) -> Vec<String> {
    results(v)
        .iter()
        .map(|r| r["cite"].as_str().unwrap().to_string())
        .collect()
}

fn fixture(name: &str) -> String {
    format!("{FORMAT2}/{name}")
}

fn setup(name: &str) -> String {
    format!("{SETUPS}/{name}")
}

// ── setup a: the format 2 workfolders ──────────────────────────────────────

#[test]
fn a_current_crystal_ranks_above_the_one_it_replaced() {
    let s = Scratch::new("a_order");
    let root = s.copy(&fixture("rooms-dir"), "wf");
    let v = s.recall(&root, "plain labels", &[]);
    let c = cites(&v);
    let new = c
        .iter()
        .position(|x| x == "rooms/work/CLAUDE.md:12")
        .unwrap();
    let old = c
        .iter()
        .position(|x| x == "rooms/work/CLAUDE.md:10-11")
        .unwrap();
    assert!(
        new < old,
        "the superseded Framing outranks Framing v2: {c:?}"
    );
    assert_eq!(results(&v)[new]["state"], "current");
    assert_eq!(results(&v)[old]["state"], "superseded");
    assert_eq!(results(&v)[old]["superseded_by"], "Framing v2");
    assert_eq!(results(&v)[old]["superseded_by_found"], true);
}

#[test]
fn retiring_the_newer_crystal_inverts_the_order() {
    // The mutation that proves the ordering reads state: a crystal that is no longer current
    // must not outrank one that is, however well it matches.
    let s = Scratch::new("a_mutation");
    let root = s.copy(&fixture("rooms-dir"), "wf");
    let path = root.join("rooms/work/CLAUDE.md");
    let text = fs::read_to_string(&path).unwrap();
    let text = text.replace(
        "◇ Framing v2: plain labels win in every segment\n",
        "◇ Framing v2: plain labels win in every segment\nretired: 2026-11-20 replaced again\n",
    );
    fs::write(&path, text).unwrap();
    let v = s.recall(&root, "plain labels", &[]);
    let c = cites(&v);
    let new = c
        .iter()
        .position(|x| x == "rooms/work/CLAUDE.md:12-13")
        .unwrap();
    let old = c
        .iter()
        .position(|x| x == "rooms/work/CLAUDE.md:10-11")
        .unwrap();
    assert!(old < new, "retired must rank below superseded: {c:?}");
    assert_eq!(results(&v)[new]["state"], "retired");
}

#[test]
fn a_dangling_superseded_by_still_reads_superseded() {
    let s = Scratch::new("a_dangling");
    let root = s.copy(&fixture("rooms-dir"), "wf");
    let v = s.recall(&root, "postgres", &[]);
    let r = &results(&v)[0];
    assert_eq!(r["cite"], "rooms/work/decision.md:9");
    assert_eq!(r["state"], "superseded");
    assert_eq!(r["superseded_by"], "Hosting choice");
    assert_eq!(r["superseded_by_found"], false);
    assert_eq!(r["frontmatter"]["tier"], "Working");
}

#[test]
fn a_glyphless_bullet_is_a_searchable_note_not_a_crystal() {
    let s = Scratch::new("a_note");
    let root = s.copy(&fixture("rooms-dir"), "wf");
    let v = s.recall(&root, "gardening", &[]);
    let r = &results(&v)[0];
    assert_eq!(r["cite"], "CLAUDE.md:41");
    assert_eq!(r["kind"], "note");
    assert_eq!(r["state"], "none");
    assert_eq!(v["scope"]["legacy_lines_not_counted"], 1);
}

#[test]
fn a_crystal_becomes_due_only_when_as_of_reaches_its_date() {
    let s = Scratch::new("a_due");
    let root = s.copy(&fixture("rooms-dir"), "wf");
    let state = |as_of: &str| -> String {
        let v = s.recall(&root, "renewal pending", &["--as-of", as_of]);
        let r = results(&v)
            .iter()
            .find(|r| r["name"] == "Contract")
            .expect("Contract is a hit")
            .clone();
        assert_eq!(r["cite"], "rooms/work/CLAUDE.md:7-9");
        assert_eq!(r["valid_until_kind"], "scheduled");
        r["state"].as_str().unwrap().to_string()
    };
    assert_eq!(state("2027-01-02"), "due");
    assert_eq!(state("2026-11-24"), "current");
}

#[test]
fn a_label_never_makes_a_crystal_due() {
    let s = Scratch::new("a_label");
    let root = s.copy(&fixture("rooms-dir"), "wf");
    let v = s.recall(&root, "daily while the feature", &["--as-of", "2099-01-01"]);
    let r = &results(&v)[0];
    assert_eq!(r["valid_until"], "sprint-end");
    assert_eq!(r["valid_until_kind"], "unscheduled");
    assert_eq!(r["state"], "current");
}

#[test]
fn the_other_two_layouts_are_read_where_their_crystals_live() {
    let s = Scratch::new("a_layouts");
    let sub = s.copy(&fixture("palace-subdir"), "sub");
    let v = s.recall(&sub, "quiet", &[]);
    assert_eq!(cites(&v)[0], "_palace/ideas/CLAUDE.md:3");
    assert_eq!(v["scope"]["roots"][0]["layout"], "palace-subdir");

    let flat = s.copy(&fixture("rooms-at-root"), "flat");
    let v = s.recall(&flat, "fence", &[]);
    let c = cites(&v);
    assert!(c.contains(&"home/CLAUDE.md:3-4".to_string()), "{c:?}");
    assert!(c.iter().any(|x| x.starts_with("home/plan.md:")), "{c:?}");
    assert_eq!(v["scope"]["roots"][0]["layout"], "rooms-at-root");
}

#[test]
fn a_file_crystal_hit_in_its_body_still_reports_its_state() {
    let s = Scratch::new("a_body");
    let root = s.copy(&fixture("rooms-dir"), "wf");
    let v = s.recall(&root, "free text", &[]);
    let r = results(&v)
        .iter()
        .find(|r| r["path"] == "rooms/work/decision.md")
        .expect("the body paragraph is a hit");
    assert_eq!(r["line"], 11);
    assert_eq!(r["state"], "superseded");
    assert_eq!(r["superseded_by"], "Hosting choice");
}

#[test]
fn an_unmatched_query_is_an_answer_and_says_what_ran() {
    let s = Scratch::new("a_clean");
    let root = s.copy(&fixture("rooms-dir"), "wf");
    let o = s.run(&["recall", "zzzqqq", "--root", root.to_str().unwrap()]);
    assert_eq!(code(&o), 0);
    let text = stdout(&o);
    assert!(text.contains("no matches"), "{text}");
    assert!(text.contains("found in no file: zzzqqq"), "{text}");
    let v = s.recall(&root, "zzzqqq", &[]);
    assert!(results(&v).is_empty());
    assert_eq!(v["unmatched_tokens"][0], "zzzqqq");
    assert!(v["scope"]["files_read"].as_u64().unwrap() > 0);
    assert!(v["scope"]["chunks"].as_u64().unwrap() > 0);
}

#[test]
fn two_runs_are_byte_identical() {
    let s = Scratch::new("a_same");
    let root = s.copy(&fixture("rooms-dir"), "wf");
    let args = [
        "recall",
        "plain labels hosting",
        "--root",
        root.to_str().unwrap(),
        "--as-of",
        "2026-11-25",
    ];
    for extra in [&[][..], &["--json"][..]] {
        let mut a = args.to_vec();
        a.extend_from_slice(extra);
        let (x, y) = (s.run(&a), s.run(&a));
        assert_eq!(code(&x), 0);
        assert_eq!(x.stdout, y.stdout);
    }
}

#[test]
fn limit_and_room_narrow_the_results_without_changing_scores() {
    let s = Scratch::new("a_narrow");
    let root = s.copy(&fixture("rooms-dir"), "wf");
    let all = s.recall(&root, "plain", &[]);
    let top = s.recall(&root, "plain", &["--limit", "1"]);
    assert_eq!(results(&top).len(), 1);
    assert_eq!(results(&top)[0]["cite"], results(&all)[0]["cite"]);
    let room = s.recall(&root, "plain", &["--room", "work"]);
    assert!(results(&room).iter().all(|r| r["room"] == "work"));
    for r in results(&room) {
        let same = results(&all)
            .iter()
            .find(|a| a["cite"] == r["cite"])
            .unwrap();
        assert_eq!(same["score"], r["score"]);
    }
    let o = s.run(&[
        "recall",
        "plain",
        "--root",
        root.to_str().unwrap(),
        "--room",
        "nowhere",
    ]);
    assert_eq!(code(&o), 2, "{}", stderr(&o));
}

#[test]
fn same_name_unresolved_counts_a_name_two_crystals_share() {
    let s = Scratch::new("a_samename");
    let root = s.copy(&fixture("rooms-dir"), "wf");
    let clean = s.recall(&root, "acme hosting", &[]);
    assert_eq!(results(&clean)[0]["name"], "Vendor");
    assert_eq!(results(&clean)[0]["same_name_unresolved"], 1);
    fs::write(
        root.join("rooms/work/vendor-change.md"),
        "---\ntype: crystal\ntier: contextual\noccurred: 2026-11-20\n---\n# vendor\n\nBolt for hosting.\n",
    )
    .unwrap();
    let v = s.recall(&root, "acme hosting", &[]);
    assert_eq!(results(&v)[0]["same_name_unresolved"], 2);
    let text = stdout(&s.run(&["recall", "acme hosting", "--root", root.to_str().unwrap()]));
    assert!(text.contains("same_name_unresolved: 2"), "{text}");
}

// ── setup b: how the root is found ─────────────────────────────────────────

#[test]
fn b_the_workfolder_variable_finds_the_palace_with_no_flag() {
    let s = Scratch::new("b_env");
    let ws = s.copy(&fixture("palace-subdir"), "ws");
    let ws = ws.to_str().unwrap();
    let hits = |o: &Output| -> Vec<String> {
        let v: Value = serde_json::from_slice(&o.stdout).unwrap();
        cites(&v)
    };
    let a = s.run_env(
        &["recall", "quiet", "--json"],
        &[("LOCI_WORKFOLDER", ws)],
        &s.cwd(),
    );
    assert_eq!(code(&a), 0, "{}", stderr(&a));
    assert_eq!(hits(&a), vec!["_palace/ideas/CLAUDE.md:3"]);
    // The older name reads the same folder.
    let b = s.run_env(
        &["recall", "quiet", "--json"],
        &[("LOCI_PALACE", ws)],
        &s.cwd(),
    );
    assert_eq!(hits(&b), hits(&a));
    // So does the flag.
    let c = s.run(&["recall", "quiet", "--json", "--workfolder", ws]);
    assert_eq!(hits(&c), hits(&a));
}

#[test]
fn b_a_variable_naming_a_missing_directory_does_not_fall_back_to_the_cwd() {
    let s = Scratch::new("b_missing");
    // The cwd is a workfolder that would answer; the variable must still be honoured.
    let ws = s.copy(&fixture("palace-subdir"), "ws");
    let gone = s.base.join("not-there");
    let o = s.run_env(
        &["recall", "quiet"],
        &[("LOCI_WORKFOLDER", gone.to_str().unwrap())],
        &ws,
    );
    assert_eq!(code(&o), 2);
    assert!(stderr(&o).contains("LOCI_WORKFOLDER"), "{}", stderr(&o));
    assert!(stdout(&o).is_empty());
}

#[test]
fn b_walking_up_finds_the_workfolder_and_nothing_else_does() {
    let s = Scratch::new("b_walk");
    let ws = s.copy(&fixture("palace-subdir"), "ws");
    let inside = ws.join("_palace/ideas");
    let o = s.run_env(&["recall", "quiet", "--json"], &[], &inside);
    assert_eq!(code(&o), 0, "{}", stderr(&o));
    // A plain folder is never found by walking: with nothing named, exit 2 and the three ways.
    let o = s.run(&["recall", "quiet"]);
    assert_eq!(code(&o), 2);
    assert!(stderr(&o).contains("--root"), "{}", stderr(&o));
}

// ── setup c: a plain folder of notes ───────────────────────────────────────

#[test]
fn c_a_plain_folder_is_read_when_named_and_counts_what_it_could_not() {
    let s = Scratch::new("c_plain");
    let root = s.copy(&setup("c/notes"), "notes");
    let clean = s.run(&["recall", "kettle", "--root", root.to_str().unwrap()]);
    assert_eq!(code(&clean), 0);
    assert!(!stdout(&clean).contains("not read"), "{}", stdout(&clean));
    let v = s.recall(&root, "kettle", &[]);
    assert_eq!(v["scope"]["roots"][0]["kind"], "plain");
    assert_eq!(v["scope"]["files_read"], 3);
    assert_eq!(v["scope"]["not_read"]["total"], 0);
    assert!(results(&v).iter().all(|r| r["state"] == "none"));
    assert_eq!(results(&v).len(), 2);

    // The mutation: two files that cannot be read must be counted, not lost, and must not
    // change the exit code.
    fs::write(root.join("binary.md"), [0xff, 0xfe, 0x00, 0xc3, 0x28]).unwrap();
    fs::write(root.join("huge.md"), vec![b'a'; 1024 * 1024 + 1]).unwrap();
    let o = s.run(&["recall", "kettle", "--root", root.to_str().unwrap()]);
    assert_eq!(code(&o), 0);
    assert!(
        stdout(&o).contains("2 not read (1 not UTF-8, 1 over 1 MiB)"),
        "{}",
        stdout(&o)
    );
    let v = s.recall(&root, "kettle", &[]);
    assert_eq!(v["scope"]["not_read"]["not_utf8"], 1);
    assert_eq!(v["scope"]["not_read"]["over_cap"], 1);
    assert_eq!(v["scope"]["files_read"], 3);
}

#[test]
fn c_a_symlink_is_counted_and_never_followed() {
    let s = Scratch::new("c_link");
    let root = s.copy(&setup("c/notes"), "notes");
    let outside = s.base.join("outside");
    fs::create_dir_all(&outside).unwrap();
    fs::write(outside.join("secret.md"), "the walrus keeps a secret\n").unwrap();
    std::os::unix::fs::symlink(&outside, root.join("link")).unwrap();
    std::os::unix::fs::symlink(outside.join("secret.md"), root.join("secret-link.md")).unwrap();
    let v = s.recall(&root, "walrus", &[]);
    assert!(
        results(&v).is_empty(),
        "a linked file was read: {:?}",
        cites(&v)
    );
    assert_eq!(v["scope"]["skipped_symlinks"], 2);
    assert_eq!(v["scope"]["files_read"], 3);
}

#[test]
fn c_a_query_with_no_words_and_bad_arguments_exit_3() {
    let s = Scratch::new("c_bad");
    let root = s.copy(&setup("c/notes"), "notes");
    let r = root.to_str().unwrap();
    for args in [
        vec!["recall", "!!! ???", "--root", r],
        vec!["recall", "kettle", "--root", r, "--as-of", "2026-02-30"],
        vec!["recall", "kettle", "--root", r, "--limit", "0"],
        vec!["recall", "kettle", "--root", r, "--limit", "101"],
    ] {
        let o = s.run(&args);
        assert_eq!(code(&o), 3, "{args:?}: {}", stderr(&o));
        assert!(stdout(&o).is_empty(), "{args:?}");
    }
    let o = s.run(&["recall", "kettle", "--root", "/definitely/not/here"]);
    assert_eq!(code(&o), 2);
}

#[test]
fn c_a_file_whose_frontmatter_never_closes_is_read_as_text_and_counted() {
    let s = Scratch::new("c_malformed");
    let root = s.base.join("notes");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("open.md"),
        "---\ntitle: never closed\n\nthe osprey nests here\n",
    )
    .unwrap();
    let v = s.recall(&root, "osprey", &[]);
    assert_eq!(cites(&v), vec!["open.md:4"]);
    assert_eq!(v["scope"]["malformed_frontmatter"], 1);
}

#[test]
fn c_a_long_paragraph_is_cut_into_chunks_of_at_most_forty_lines() {
    let s = Scratch::new("c_long");
    let root = s.base.join("notes");
    fs::create_dir_all(&root).unwrap();
    let mut text = String::new();
    for i in 1..=100 {
        text.push_str(&format!("line{i} filler\n"));
    }
    text.push_str("\nthe heron lands\n");
    fs::write(root.join("long.md"), &text).unwrap();
    let v = s.recall(&root, "line95", &[]);
    assert_eq!(cites(&v), vec!["long.md:81-100"]);
    let v = s.recall(&root, "line1", &[]);
    assert_eq!(cites(&v), vec!["long.md:1-40"]);
}

// ── setup d: an Obsidian-style vault ───────────────────────────────────────

#[test]
fn d_tags_and_link_text_are_found_and_the_config_directory_is_not_read() {
    let s = Scratch::new("d_vault");
    let root = s.copy(&setup("d/vault"), "vault");
    let v = s.recall(&root, "hatchery", &[]);
    let r = &results(&v)[0];
    assert_eq!(r["path"], "Daily Notes/My Note.md");
    assert_eq!(r["line"], 1);
    assert_eq!(r["frontmatter"]["tags"], "[seed, hatchery]");

    let v = s.recall(&root, "alias", &[]);
    assert_eq!(cites(&v), vec!["Daily Notes/My Note.md:7"]);

    // The clean case: a word that occurs only under `.obsidian/` returns nothing, and the
    // directory is counted as skipped.
    let v = s.recall(&root, "zebrafish", &[]);
    assert!(results(&v).is_empty());
    assert_eq!(v["scope"]["skipped_dirs"], 1);
    assert_eq!(v["scope"]["files_read"], 2);
}

#[test]
fn d_a_date_in_frontmatter_is_not_searchable() {
    let s = Scratch::new("d_dates");
    let root = s.copy(&setup("d/vault"), "vault");
    let v = s.recall(&root, "2026", &[]);
    assert!(results(&v).is_empty(), "{:?}", cites(&v));
}

// ── setup e: an auto-memory directory ──────────────────────────────────────

#[test]
fn e_a_description_is_found_and_shown_and_nothing_here_is_a_crystal() {
    let s = Scratch::new("e_mem");
    let root = s.copy(&setup("e/mem"), "mem");
    let v = s.recall(&root, "moorings", &[]);
    let r = &results(&v)[0];
    assert_eq!(r["path"], "a.md");
    assert_eq!(
        r["frontmatter"]["description"],
        "where the moorings are, by pier number"
    );
    assert_eq!(r["frontmatter"]["name"], "Harbour map");
    // A nested `metadata:` block is not a top-level key.
    assert!(r["frontmatter"].get("metadata").is_none());
    assert!(r["frontmatter"].get("type").is_none());
    assert_eq!(v["scope"]["crystals_read"], 0);
    assert_eq!(r["kind"], "document");
    assert_eq!(r["state"], "none");

    let text = stdout(&s.run(&["recall", "moorings", "--root", root.to_str().unwrap()]));
    assert!(text.contains("0 crystals read"), "{text}");

    // The `.bak` backup beside the memories is not part of the corpus.
    let v = s.recall(&root, "tangerine", &[]);
    assert!(results(&v).is_empty());
    assert_eq!(v["scope"]["files_read"], 3);
}

#[test]
fn e_an_index_link_is_findable_by_its_title() {
    let s = Scratch::new("e_index");
    let root = s.copy(&setup("e/mem"), "mem");
    let v = s.recall(&root, "boats are kept", &[]);
    assert_eq!(cites(&v)[0], "MEMORY.md:1-2");
}

// ── setup g: other languages ───────────────────────────────────────────────

#[test]
fn g_words_match_whole_and_the_limits_are_visible() {
    let s = Scratch::new("g_lang");
    let root = s.copy(&setup("g/notes"), "notes");
    let v = s.recall(&root, "árvíztűrő", &[]);
    assert_eq!(results(&v)[0]["path"], "hu.md");
    assert_eq!(v["tokenizer"], "unicode-alnum-v1");
    assert_eq!(v["scoring"], "bm25(k1=1.2,b=0.75)");

    // Known limits, asserted so they cannot change without notice: no accent folding, no
    // stemming. The query is echoed as the tokens it became, and the miss is named.
    let v = s.recall(&root, "arvizturo", &[]);
    assert!(results(&v).is_empty());
    assert_eq!(v["unmatched_tokens"][0], "arvizturo");
    let v = s.recall(&root, "házban", &[]);
    assert!(results(&v).is_empty());

    let v = s.recall(&root, "café", &[]);
    let paths: Vec<&str> = results(&v)
        .iter()
        .map(|r| r["path"].as_str().unwrap())
        .collect();
    assert!(
        paths.contains(&"fr.md") && paths.contains(&"mixed.md"),
        "{paths:?}"
    );
}

#[test]
fn g_a_dotted_or_hyphenated_compound_finds_itself_and_its_parts() {
    let s = Scratch::new("g_compound");
    let root = s.base.join("notes");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("v.md"),
        "Pinned to version 2.5.0 for the well-known bug.\n",
    )
    .unwrap();
    for q in ["2.5.0", "well-known", "well", "known"] {
        let v = s.recall(&root, q, &[]);
        assert_eq!(cites(&v), vec!["v.md:1"], "query {q}");
    }
    let v = s.recall(&root, "5.2", &[]);
    assert!(
        results(&v).is_empty() || v["unmatched_tokens"].as_array().unwrap().len() == 1,
        "5.2 is not a compound that appears in the text"
    );
}
