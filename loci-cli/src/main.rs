//! loci: read your local workfolder from the terminal.
//!
//! Mostly read-shaped commands. No network. No inference. No daemons.
//! Two commands write: `new` (scaffold) and `init` (config file).
//! Honest about what it is: a CLI that knows the palace layout and prints what's there.
//! It never starts an agent runtime: `loci rain` shows the weather, and the person
//! starts the round in their own agent.

use clap::{CommandFactory, Parser, Subcommand};
use serde::Serialize;
use std::io::{IsTerminal, Read, Write};
use std::path::{Path, PathBuf};

use loci_wal::{ChainError, EgressClass, Frame, ProofBundle, Wal};
use std::process::ExitCode;

mod palace;
mod reads;
mod scaffold;
mod tokens;

#[derive(Parser)]
#[command(
    name = "loci",
    version,
    about = "Read your local workfolder from the terminal.",
    long_about = "Read your local workfolder from the terminal.\n\n\
                  loci is a local AI topology: one workfolder your AI reads and writes.\n\
                  This CLI walks the workfolder and prints what's there. Most commands\n\
                  only read; `new` and `init` write. It never starts an agent.\n\
                  No network, no inference, no daemons. The templates kit is the other\n\
                  door; the CLI is the terminal-native one. Files, flags and output\n\
                  still say palace: --palace and $LOCI_PALACE point at the workfolder."
)]
struct Cli {
    /// Palace path. Overrides $LOCI_PALACE and cwd auto-detect.
    #[arg(long, global = true)]
    palace: Option<PathBuf>,

    /// Emit machine-readable JSON.
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Print palace path, layout, room and crystal counts.
    Status,
    /// List crystals (`.md` files inside rooms).
    Crystals {
        /// Only list crystals inside this room.
        #[arg(long)]
        room: Option<String>,
    },
    /// Print a crystal's contents by slug (filename without `.md`).
    Read {
        /// Crystal slug, case-insensitive.
        slug: String,
        /// Disambiguate when the same slug exists in multiple rooms.
        #[arg(long)]
        room: Option<String>,
    },
    /// Print the most recent handover (by mtime).
    Handover,
    /// Approximate agent-runtime session-window status (5h rolling) from local transcripts.
    Tokens,
    /// How often the agent reads the palace: user turns followed by a palace read, from local Claude Code transcripts (counts only).
    Reads {
        /// Look back this many days (1-90).
        #[arg(long, default_value_t = reads::DEFAULT_DAYS)]
        days: u64,
    },
    /// Garden watering weather: window headroom + garden state + how to start a rain round.
    Rain {
        /// Removed in 2.0: the CLI no longer starts an agent. Kept hidden so an old
        /// habit gets a clear message instead of a usage error.
        #[arg(long, hide = true)]
        fire: bool,
    },
    /// Scaffold a new palace at PATH: a starter palace + the lifecycle skills.
    New {
        /// Where to create the palace (e.g. ~/my-palace).
        path: PathBuf,
        /// Scaffold into a non-empty or existing-palace directory instead of refusing.
        #[arg(long)]
        force: bool,
    },
    /// Interactive setup wizard. Writes `~/.config/loci/config.toml`.
    Init,
    /// Egress receipt (experimental): what a runtime recorded as leaving the device.
    ///
    /// Nothing in loci writes this log yet; a runtime that wants receipts calls
    /// `loci_wal::record_egress`. An empty receipt means nothing was recorded, not
    /// that nothing left. Frames are grouped by class with a hash-chain sanity
    /// check. For cryptographic proof, a runtime that holds the signing key exports a
    /// bundle and `wal verify` checks it.
    Audit {
        /// WAL path (default: ~/.loci/wal/egress.jsonl).
        #[arg(long)]
        wal: Option<PathBuf>,
        /// Only count frames at/after this ISO-8601 UTC timestamp (lexicographic).
        #[arg(long)]
        since: Option<String>,
    },
    /// Proof-bundle tools (experimental): verify an exported egress receipt.
    Wal {
        #[command(subcommand)]
        cmd: WalCmd,
    },
    /// Overview of every command, grouped, with examples and the two doors.
    #[command(alias = "commands")]
    Overview,
}

#[derive(Subcommand)]
enum WalCmd {
    /// Verify an exported proof bundle offline (experimental; pure, no network).
    ///
    /// Bundles are minted by a runtime that holds the signing key; nothing in this
    /// CLI mints one.
    Verify {
        /// Path to the bundle JSON.
        bundle: PathBuf,
        /// Require the signer key to equal this hex pubkey (provenance check).
        #[arg(long)]
        expect_key: Option<String>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("loci: {e}");
            match e.kind() {
                ErrKind::NotFound => ExitCode::from(2),
                ErrKind::BadInput => ExitCode::from(3),
                ErrKind::Io => ExitCode::from(1),
            }
        }
    }
}

fn run(cli: Cli) -> Result<(), Error> {
    match cli.cmd {
        Cmd::Status => cmd_status(cli.palace, cli.json),
        Cmd::Crystals { room } => cmd_crystals(cli.palace, room, cli.json),
        Cmd::Read { slug, room } => cmd_read(cli.palace, &slug, room.as_deref(), cli.json),
        Cmd::Handover => cmd_handover(cli.palace, cli.json),
        Cmd::Tokens => cmd_tokens(cli.json),
        Cmd::Reads { days } => cmd_reads(cli.palace, days, cli.json),
        Cmd::Rain { fire } => cmd_rain(cli.palace, fire, cli.json),
        Cmd::New { path, force } => cmd_new(&path, force, cli.json),
        Cmd::Init => cmd_init(None),
        Cmd::Audit { wal, since } => cmd_audit(wal, since, cli.json),
        Cmd::Wal { cmd } => match cmd {
            WalCmd::Verify { bundle, expect_key } => {
                cmd_wal_verify(&bundle, expect_key.as_deref(), cli.json)
            }
        },
        Cmd::Overview => cmd_overview(cli.palace, cli.json),
    }
}

// ── overview ─────────────────────────────────────────────────────────────

/// Curated grouping of the commands. Descriptions are deliberately NOT stored
/// here: they are read live from clap's `about` (the doc comment on each `Cmd`
/// variant), so this table only decides ordering, sectioning, and one example
/// per command. A unit test asserts every clap subcommand appears here exactly
/// once, so a new command cannot silently drop out of the overview.
const OVERVIEW_GROUPS: &[(&str, &[(&str, &str)])] = &[
    (
        "Read the palace",
        &[
            ("status", "loci status"),
            ("crystals", "loci crystals --room soul"),
            ("read", "loci read <slug>"),
            ("handover", "loci handover"),
        ],
    ),
    (
        "Garden & session",
        &[
            ("tokens", "loci tokens"),
            ("reads", "loci reads --days 30"),
            ("rain", "loci rain"),
        ],
    ),
    (
        "Egress & proof",
        &[
            ("audit", "loci audit --since 2026-01-01"),
            ("wal", "loci wal verify <bundle.json>"),
        ],
    ),
    (
        "Setup",
        &[
            ("new", "loci new ~/my-palace"),
            ("init", "loci init"),
        ],
    ),
    ("Meta", &[("overview", "loci overview")]),
];

/// The two ways into a palace. Named in the CLI's own `long_about`; restated
/// here so the overview places this terminal door among its siblings.
const DOORS: &[(&str, &str)] = &[
    (
        "CLI (this door)",
        "terminal-native: walk the palace and print what's there",
    ),
    (
        "Templates kit",
        "scaffold a new palace to fork: personas, skills, structure",
    ),
];

#[derive(Serialize)]
struct OverviewOut {
    tool: &'static str,
    version: &'static str,
    palace: Option<String>,
    groups: Vec<OverviewGroupOut>,
    doors: Vec<DoorOut>,
}

#[derive(Serialize)]
struct OverviewGroupOut {
    name: String,
    commands: Vec<OverviewCmdOut>,
}

#[derive(Serialize)]
struct OverviewCmdOut {
    name: String,
    about: String,
    usage: String,
}

#[derive(Serialize)]
struct DoorOut {
    name: String,
    role: String,
}

/// Read each command's one-line description straight from clap (`about`), so the
/// overview can never disagree with `--help`. Keyed by command name; clap's
/// built-in `help` subcommand is excluded.
fn command_abouts() -> std::collections::BTreeMap<String, String> {
    Cli::command()
        .get_subcommands()
        .filter(|c| c.get_name() != "help")
        .map(|c| {
            let about = c.get_about().map(|s| s.to_string()).unwrap_or_default();
            // Keep the compact catalogue to one line; full text is in `--help`.
            let first = about.lines().next().unwrap_or("").to_string();
            (c.get_name().to_string(), first)
        })
        .collect()
}

/// Trim a description to a brief, word-boundary-clean line for the text view.
/// The full text stays available in `--json` and via `<cmd> --help`.
fn brief(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out = String::new();
    for word in s.split_whitespace() {
        if out.chars().count() + word.chars().count() + 1 > max {
            break;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(word);
    }
    out.push('…');
    out
}

fn cmd_overview(palace_arg: Option<PathBuf>, json: bool) -> Result<(), Error> {
    let abouts = command_abouts();
    // Soft detection: overview must work with no palace (unlike require_palace).
    let palace = match palace::detect(palace_arg) {
        Ok(found) => found.map(|p| p.root.display().to_string()),
        Err(palace::BadEnv(p)) => {
            eprintln!(
                "loci: warning: LOCI_PALACE is set to {} but no palace is there",
                p.display()
            );
            None
        }
    };

    if json {
        let groups = OVERVIEW_GROUPS
            .iter()
            .map(|(gname, cmds)| OverviewGroupOut {
                name: (*gname).to_string(),
                commands: cmds
                    .iter()
                    .map(|(n, usage)| OverviewCmdOut {
                        name: (*n).to_string(),
                        about: abouts.get(*n).cloned().unwrap_or_default(),
                        usage: (*usage).to_string(),
                    })
                    .collect(),
            })
            .collect();
        let doors = DOORS
            .iter()
            .map(|(n, r)| DoorOut {
                name: (*n).to_string(),
                role: (*r).to_string(),
            })
            .collect();
        let out = OverviewOut {
            tool: "loci",
            version: env!("CARGO_PKG_VERSION"),
            palace,
            groups,
            doors,
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!(
        "loci {} · read your local workfolder from the terminal",
        env!("CARGO_PKG_VERSION")
    );
    println!("Mostly reads (new and init write). Starts no agent. No network, no inference, no daemons.");
    match &palace {
        Some(p) => println!("palace : {p}"),
        None => println!("palace : (none detected — pass --palace <path> or run inside one)"),
    }
    println!();
    for (gname, cmds) in OVERVIEW_GROUPS {
        println!("{gname}");
        for (n, usage) in *cmds {
            let about = brief(abouts.get(*n).map(|s| s.as_str()).unwrap_or(""), 64);
            println!("  {n:<10} {about}");
            println!("  {:<10}   e.g. {usage}", "");
        }
        println!();
    }
    println!("The two doors");
    for (n, r) in DOORS {
        println!("  {n:<16} {r}");
    }
    println!();
    println!("Run `loci <command> --help` for the full options of any command.");
    Ok(())
}

// ── Commands ───────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct StatusOut {
    palace: String,
    layout: &'static str,
    rooms: Vec<RoomOut>,
    crystal_total: usize,
}

#[derive(Serialize)]
struct RoomOut {
    name: String,
    crystals: usize,
}

fn cmd_status(palace_arg: Option<PathBuf>, json: bool) -> Result<(), Error> {
    let p = require_palace(palace_arg)?;
    let rooms = palace::list_rooms(&p);
    let crystal_total = palace::count_md_files(&p.scan_root);
    let layout = match p.layout {
        palace::Layout::PalaceSubdir => "palace-subdir",
        palace::Layout::RoomsDir => "rooms-dir",
        palace::Layout::RoomsAtRoot => "rooms-at-root",
    };

    if json {
        let out = StatusOut {
            palace: p.root.display().to_string(),
            layout,
            rooms: rooms
                .iter()
                .map(|r| RoomOut {
                    name: r.name.clone(),
                    crystals: r.crystal_count,
                })
                .collect(),
            crystal_total,
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!("palace : {}", p.root.display());
    println!("layout : {layout}");
    println!("rooms  : {}", rooms.len());
    for r in &rooms {
        println!("         {:<24} ({} crystals)", r.name, r.crystal_count);
    }
    println!("total  : {crystal_total} crystals");
    Ok(())
}

#[derive(Serialize)]
struct CrystalsOut {
    palace: String,
    rooms: Vec<RoomListOut>,
}

#[derive(Serialize)]
struct RoomListOut {
    room: String,
    crystals: Vec<String>,
}

fn cmd_crystals(
    palace_arg: Option<PathBuf>,
    room: Option<String>,
    json: bool,
) -> Result<(), Error> {
    let p = require_palace(palace_arg)?;
    let mut rooms = palace::list_rooms(&p);
    if let Some(ref filter) = room {
        rooms.retain(|r| r.name == *filter);
        if rooms.is_empty() {
            return Err(Error::not_found(format!("room not found: {filter}")));
        }
    }

    let mut per_room: Vec<RoomListOut> = Vec::new();
    for r in &rooms {
        let mut slugs = collect_slugs(&r.path);
        slugs.sort();
        per_room.push(RoomListOut {
            room: r.name.clone(),
            crystals: slugs,
        });
    }

    if json {
        let out = CrystalsOut {
            palace: p.root.display().to_string(),
            rooms: per_room,
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    for r in &per_room {
        println!("# {}", r.room);
        for slug in &r.crystals {
            println!("  {slug}");
        }
        println!();
    }
    Ok(())
}

fn collect_slugs(dir: &std::path::Path) -> Vec<String> {
    let mut slugs = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().and_then(|e| e.to_str()) == Some("md") {
                if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                    slugs.push(stem.to_string());
                }
            }
        }
    }
    slugs
}

#[derive(Serialize)]
struct ReadOut {
    slug: String,
    path: String,
    content: String,
}

fn cmd_read(
    palace_arg: Option<PathBuf>,
    slug: &str,
    room: Option<&str>,
    json: bool,
) -> Result<(), Error> {
    let p = require_palace(palace_arg)?;
    let hits = palace::find_crystal(&p, slug, room);
    match hits.len() {
        0 => Err(Error::not_found(format!(
            "no crystal with slug '{slug}'{}",
            room.map(|r| format!(" in room '{r}'")).unwrap_or_default()
        ))),
        1 => {
            let path = &hits[0];
            let content = std::fs::read_to_string(path)?;
            if json {
                let out = ReadOut {
                    slug: slug.to_string(),
                    path: path.display().to_string(),
                    content,
                };
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                print!("{content}");
            }
            Ok(())
        }
        _ => {
            let mut msg = format!("multiple crystals named '{slug}'. Disambiguate with --room:\n");
            for h in &hits {
                msg.push_str(&format!("  {}\n", h.display()));
            }
            Err(Error::bad_input(msg))
        }
    }
}

#[derive(Serialize)]
struct HandoverOut {
    path: String,
    content: String,
}

fn cmd_handover(palace_arg: Option<PathBuf>, json: bool) -> Result<(), Error> {
    let p = require_palace(palace_arg)?;
    let path = palace::latest_handover(&p)
        .ok_or_else(|| Error::not_found("no handover found in palace".to_string()))?;
    let content = std::fs::read_to_string(&path)?;
    if json {
        let out = HandoverOut {
            path: path.display().to_string(),
            content,
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        print!("{content}");
    }
    Ok(())
}

// ── audit / proof bundle ──────────────────────────────────────────────────

/// Refuse to read files larger than these caps: `audit` + `wal verify` run on
/// potentially attacker-supplied files, so bound the allocation up front.
const MAX_WAL_BYTES: u64 = 256 * 1024 * 1024;
const MAX_BUNDLE_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Serialize)]
struct AuditClassOut {
    egress_class: &'static str,
    events: usize,
    bytes: u64,
    hosts: Vec<String>,
}

#[derive(Serialize)]
struct AuditOut {
    wal: String,
    frames: usize,
    chain_ok: bool,
    chain_break_seq: Option<u64>,
    since: Option<String>,
    degraded: usize,
    degraded_latest: Option<String>,
    disabled_at: Option<String>,
    unreadable_lines: usize,
    /// Always true while nothing in loci writes the log.
    experimental: bool,
    /// Set when the log holds no frames: an empty receipt is not proof of no egress.
    note: Option<&'static str>,
    classes: Vec<AuditClassOut>,
}

fn class_label(c: EgressClass) -> &'static str {
    match c {
        EgressClass::Local => "local",
        EgressClass::LocalNetwork => "local_network",
        EgressClass::ExternalCloud => "external_cloud",
        EgressClass::ChannelEgress => "channel_egress",
        EgressClass::ProfileWrite => "profile_write",
    }
}

fn chain_break_seq(e: &ChainError) -> Option<u64> {
    match e {
        ChainError::Empty => None,
        ChainError::BrokenLink(s) | ChainError::NonContiguous(s) => Some(*s),
    }
}

fn default_wal_path() -> Result<PathBuf, Error> {
    let home = dirs::home_dir().ok_or_else(|| Error::io("could not resolve home dir".to_string()))?;
    Ok(home.join(".loci").join("wal").join("egress.jsonl"))
}

const EMPTY_RECEIPT_NOTE: &str =
    "nothing in loci writes this log yet, so an empty receipt is not evidence that nothing left the device";

fn cmd_audit(wal_arg: Option<PathBuf>, since: Option<String>, json: bool) -> Result<(), Error> {
    let path = match wal_arg {
        Some(p) => p,
        None => default_wal_path()?,
    };
    if path.exists() {
        let len = std::fs::metadata(&path)?.len();
        if len > MAX_WAL_BYTES {
            return Err(Error::bad_input(format!(
                "WAL too large: {len} bytes (cap {MAX_WAL_BYTES})"
            )));
        }
    }
    // Incompleteness signals, scoped by --since (both markers carry RFC3339 stamps):
    //   egress.degraded — one line per failed write (a dropped write leaves no chain gap)
    //   egress.disabled — last time LOCI_WAL_DISABLED suppressed the writer entirely
    // A missing marker means "nothing to report"; an unreadable one means the
    // receipt's completeness is unknown, so audit stops instead of reading it as zero.
    let degraded_content = read_marker(&path.with_file_name("egress.degraded"))?.unwrap_or_default();
    let degraded_lines: Vec<&str> = degraded_content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter(|l| {
            let ts = l.split_whitespace().next().unwrap_or("");
            since.as_deref().map_or(true, |s| ts >= s)
        })
        .collect();
    let degraded = degraded_lines.len();
    let degraded_latest = degraded_lines
        .last()
        .and_then(|l| l.split_whitespace().next())
        .map(|s| s.to_string());
    let disabled_at = read_marker(&path.with_file_name("egress.disabled"))?
        .map(|s| s.trim().to_string())
        .filter(|t| !t.is_empty() && since.as_deref().map_or(true, |s| t.as_str() >= s));
    let (all, unreadable_lines) = Wal::open(&path).read_counted()?;
    let (chain_ok, break_seq) = match Wal::verify_full(&all) {
        Ok(()) => (true, None),
        Err(e) => (false, chain_break_seq(&e)),
    };

    let selected: Vec<&Frame> = all
        .iter()
        .filter(|f| since.as_deref().map_or(true, |s| f.ts.as_str() >= s))
        .collect();

    use std::collections::{BTreeMap, BTreeSet};
    let mut groups: BTreeMap<&'static str, (usize, u64, BTreeSet<String>)> = BTreeMap::new();
    for f in &selected {
        let e = groups.entry(class_label(f.egress_class)).or_default();
        e.0 += 1;
        e.1 += f.byte_count.unwrap_or(0);
        e.2.insert(f.dest_host.clone());
    }
    let classes: Vec<AuditClassOut> = groups
        .into_iter()
        .map(|(egress_class, (events, bytes, hosts))| AuditClassOut {
            egress_class,
            events,
            bytes,
            hosts: hosts.into_iter().collect(),
        })
        .collect();

    if json {
        let out = AuditOut {
            wal: path.display().to_string(),
            frames: selected.len(),
            chain_ok,
            chain_break_seq: break_seq,
            since,
            degraded,
            degraded_latest,
            disabled_at,
            unreadable_lines,
            experimental: true,
            note: all.is_empty().then_some(EMPTY_RECEIPT_NOTE),
            classes,
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!("egress receipt : {} (experimental)", path.display());
    if let Some(ref t) = disabled_at {
        println!(
            "⚠ WARNING      : egress logging was DISABLED (LOCI_WAL_DISABLED) as of {t} — receipt is INCOMPLETE"
        );
    }
    if degraded > 0 {
        println!(
            "⚠ WARNING      : {degraded} egress record(s) failed to write (latest {}) — receipt may be INCOMPLETE",
            degraded_latest.as_deref().unwrap_or("?")
        );
    }
    if unreadable_lines > 0 {
        println!(
            "⚠ WARNING      : {unreadable_lines} unreadable line(s) in the log were skipped; receipt may be INCOMPLETE"
        );
    }
    if all.is_empty() {
        println!("frames         : 0 recorded");
        println!("note           : {EMPTY_RECEIPT_NOTE}");
        return Ok(());
    }
    let chain = if chain_ok {
        "ok".to_string()
    } else {
        format!(
            "BROKEN at seq {}",
            break_seq.map(|s| s.to_string()).unwrap_or_else(|| "?".to_string())
        )
    };
    println!("frames         : {} (chain: {chain})", selected.len());
    if let Some(ref s) = since {
        println!("since          : {s}");
    }
    println!();
    for c in &classes {
        println!(
            "  {:<16} {:>5} events   {:>10} bytes",
            c.egress_class, c.events, c.bytes
        );
        for h in &c.hosts {
            println!("       {h}");
        }
    }
    Ok(())
}

#[derive(Serialize)]
struct VerifyOut {
    bundle: String,
    result: &'static str,
    signer_pubkey: String,
    range: [u64; 2],
    frames: usize,
    detail: Option<String>,
}

fn cmd_wal_verify(path: &Path, expect_key: Option<&str>, json: bool) -> Result<(), Error> {
    let len = std::fs::metadata(path)?.len();
    if len > MAX_BUNDLE_BYTES {
        return Err(Error::bad_input(format!(
            "bundle too large: {len} bytes (cap {MAX_BUNDLE_BYTES})"
        )));
    }
    let text = std::fs::read_to_string(path)?;
    let bundle: ProofBundle = serde_json::from_str(&text)?;
    let outcome = match expect_key {
        Some(k) => bundle.verify_against(k),
        None => bundle.verify(),
    };
    let (ok, detail) = match &outcome {
        Ok(()) => (true, None),
        Err(e) => (false, Some(format!("{e:?}"))),
    };

    if json {
        let out = VerifyOut {
            bundle: path.display().to_string(),
            result: if ok { "pass" } else { "fail" },
            signer_pubkey: bundle.signer_pubkey.clone(),
            range: bundle.range,
            frames: bundle.frames.len(),
            detail,
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else if ok {
        println!(
            "PASS  {} frames, seq {}..{}",
            bundle.frames.len(),
            bundle.range[0],
            bundle.range[1]
        );
        println!("      signer {}", bundle.signer_pubkey);
        if expect_key.is_some() {
            println!("      signer matches the pinned key");
        } else {
            println!("      (signature valid for the embedded key; pass --expect-key for provenance)");
        }
    } else {
        println!("FAIL  {}", detail.unwrap_or_default());
    }

    if ok {
        Ok(())
    } else {
        Err(Error::bad_input("proof bundle verification failed".to_string()))
    }
}


// ── tokens + rain ────────────────────────────────────────────────────────

fn fmt_tok(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1e6)
    } else if n >= 1_000 {
        format!("{:.0}k", n as f64 / 1e3)
    } else {
        n.to_string()
    }
}

fn signal_glyph(signal: &str) -> &'static str {
    match signal {
        "fresh" => "☔",
        "open" => "🌦",
        "closing" => "⏳",
        _ => "🌫",
    }
}

fn cmd_tokens(json: bool) -> Result<(), Error> {
    let w = tokens::window_status();
    if json {
        println!("{}", serde_json::to_string_pretty(&w)?);
        return Ok(());
    }
    println!("window  : {} {}", signal_glyph(&w.signal), w.state);
    match (&w.window_start_utc, &w.window_reset_utc, w.minutes_remaining) {
        (Some(start), Some(reset), Some(min)) => {
            println!("started : {start}");
            println!("resets  : {reset} ({min}m left)");
        }
        _ => {}
    }
    println!(
        "spent   : {} total (in {} · out {} · cache-w {} · cache-r {}) over {} messages",
        fmt_tok(w.tokens.total),
        fmt_tok(w.tokens.input),
        fmt_tok(w.tokens.output),
        fmt_tok(w.tokens.cache_creation),
        fmt_tok(w.tokens.cache_read),
        w.messages
    );
    println!("note    : {}", w.note);
    Ok(())
}

fn cmd_reads(palace_arg: Option<PathBuf>, days: u64, json: bool) -> Result<(), Error> {
    let p = require_palace(palace_arg)?;
    let r = reads::read_rate(&p.root, days);
    if json {
        println!("{}", serde_json::to_string_pretty(&r)?);
        return Ok(());
    }
    let pct = |x: Option<f64>| x.map_or_else(|| "n/a".to_string(), |x| format!("{x:.1}%"));
    println!("root     : {}", r.root);
    println!("window   : last {} day(s), {} transcript file(s)", r.days, r.files_scanned);
    println!("turns    : {} user turn(s) in {} session(s)", r.user_turns, r.sessions);
    if r.user_turns == 0 {
        println!(
            "rates    : n/a, no user turns found in the last {} day(s) ({} transcript file(s) scanned)",
            r.days, r.files_scanned
        );
        if r.files_scanned == 0 {
            println!(
                "           no Claude Code transcripts modified in the last {} day(s) under ~/.claude/projects, ~/.config/claude/projects or $CLAUDE_CONFIG_DIR (try a larger --days)",
                r.days
            );
        }
    } else {
        println!(
            "read rate: {} of turns read the palace ({} of {}; tool reads, the floor)",
            pct(r.read_rate_pct),
            r.turns_with_read,
            r.user_turns
        );
        println!(
            "touch    : {} of turns read it or ran a shell command naming it ({} of {}; a wider count; misses shell commands that spell it relative, ~, $HOME or with escaped spaces)",
            pct(r.touch_rate_pct),
            r.turns_with_touch,
            r.user_turns
        );
    }
    println!(
        "palace   : {} read(s) · {} write(s) · {} shell touch(es) · {} of {} session(s) read it",
        r.palace_reads, r.palace_writes, r.shell_touches, r.sessions_with_read, r.sessions
    );
    println!(
        "delegated: {} read(s) · {} write(s) by subagents (outside the rate)",
        r.delegated_reads, r.delegated_writes
    );
    println!("note     : {}", r.note);
    Ok(())
}

#[derive(Serialize)]
struct RainGardenOut {
    plants: usize,
    last_rain: Option<String>,
    last_rain_waterings: Option<usize>,
}

#[derive(Serialize)]
struct RainOut {
    garden: RainGardenOut,
    window: tokens::WindowStatus,
    invocation: String,
}

fn rain_garden(p: &palace::Palace) -> RainGardenOut {
    let garden = p.scan_root.join("garden");
    let plants = std::fs::read_dir(garden.join("plants"))
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("md"))
                .count()
        })
        .unwrap_or(0);
    // Latest archived round: garden/.rain/waterings-<date>.json
    let mut last: Option<(String, PathBuf)> = None;
    if let Ok(rd) = std::fs::read_dir(garden.join(".rain")) {
        for e in rd.filter_map(|e| e.ok()) {
            let name = e.file_name().to_string_lossy().to_string();
            if let Some(date) = name
                .strip_prefix("waterings-")
                .and_then(|s| s.strip_suffix(".json"))
            {
                if last.as_ref().map(|(d, _)| date > d.as_str()).unwrap_or(true) {
                    last = Some((date.to_string(), e.path()));
                }
            }
        }
    }
    let (last_rain, last_rain_waterings) = match last {
        Some((date, path)) => {
            let n = std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                .and_then(|v| v.as_array().map(|a| a.len()));
            (Some(date), n)
        }
        None => (None, None),
    };
    RainGardenOut {
        plants,
        last_rain,
        last_rain_waterings,
    }
}

fn cmd_rain(palace_arg: Option<PathBuf>, fire: bool, json: bool) -> Result<(), Error> {
    if fire {
        return Err(Error::bad_input(
            "`loci rain --fire` was removed in 2.0: the CLI no longer starts an agent. \
             Start the round in your own agent (say `rain`); `loci rain` shows the weather."
                .to_string(),
        ));
    }
    let p = require_palace(palace_arg)?;
    let garden = rain_garden(&p);
    let window = tokens::window_status();
    // A hint for the person, never executed by the CLI.
    let invocation = "claude -p \"rain\"".to_string();

    if json {
        let out = RainOut {
            garden,
            window,
            invocation,
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!("rain · garden watering round");
    let weather = match (&window.window_reset_utc, window.minutes_remaining) {
        (Some(reset), Some(min)) => format!(
            "{} {} · {}m left (resets {reset}) · {} spent",
            signal_glyph(&window.signal),
            window.signal,
            min,
            fmt_tok(window.tokens.total)
        ),
        _ => format!(
            "{} {} · {}",
            signal_glyph(&window.signal),
            window.signal,
            window.note
        ),
    };
    println!("weather : {weather}");
    let last = match (&garden.last_rain, garden.last_rain_waterings) {
        (Some(d), Some(n)) => format!("last rain {d} ({n} waterings)"),
        (Some(d), None) => format!("last rain {d}"),
        _ => "no rain on record".to_string(),
    };
    println!("garden  : {} plants · {last}", garden.plants);
    println!("start   : say `rain` in your own agent (or Workflow({{ name: \"rain\" }})); loci starts nothing");
    if window.signal == "fresh" {
        println!("          window is fresh: full headroom. Good weather for rain.");
    } else if window.signal == "closing" {
        println!("          window closes soon: spare capacity expires with it.");
    }
    Ok(())
}

// ── init ─────────────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize)]
struct Config {
    palace_path: Option<String>,
}

// ── new (scaffold a palace) ─────────────────────────────────────────────────

#[derive(Serialize)]
struct NewOut {
    palace: String,
    created: Vec<String>,
}

fn cmd_new(path: &Path, force: bool, json: bool) -> Result<(), Error> {
    // Overwrite policy: refuse an existing palace or a non-empty directory
    // unless --force, so `new` never silently writes over someone's work.
    if !force {
        if palace::validate(path).is_some() {
            return Err(Error::bad_input(format!(
                "{} already looks like a palace. Pass --force to scaffold into it anyway.",
                path.display()
            )));
        }
        if path.is_dir()
            && path
                .read_dir()
                .map(|mut d| d.next().is_some())
                .unwrap_or(false)
        {
            return Err(Error::bad_input(format!(
                "{} is not empty. Pass --force to scaffold into it anyway.",
                path.display()
            )));
        }
    }

    std::fs::create_dir_all(path)?;
    let written = scaffold::write_files(path)?;

    if json {
        let out = NewOut {
            palace: path.display().to_string(),
            created: written.iter().map(|p| p.display().to_string()).collect(),
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!("Created a starter palace at {}", path.display());
    for p in &written {
        // Compact tree: show paths relative to the palace root.
        let shown = p.strip_prefix(path).unwrap_or(p);
        println!("  {}", shown.display());
    }
    println!();
    println!("Next:");
    println!("  1. Edit PALACE.md and soul/SOUL.md: say who you are, name your collaborator.");
    println!(
        "  2. Run `loci status --palace {}` to confirm it reads.",
        path.display()
    );
    println!("  3. The full templates kit (personas, more skills) is at loci.garden.");

    // Hand off to the config step, defaulting the palace path to the new palace.
    if std::io::stdin().is_terminal() {
        print!("\nRecord this palace in ~/.config/loci/config.toml now? (no command reads it yet) [Y/n]: ");
        std::io::stdout().flush()?;
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf)?;
        let ans = buf.trim().to_lowercase();
        if ans.is_empty() || ans == "y" || ans == "yes" {
            println!();
            return cmd_init(Some(path.display().to_string()));
        }
    }
    println!("\nWhen ready: loci init   (records the palace path; no command reads it yet)");
    Ok(())
}

fn cmd_init(default_palace: Option<String>) -> Result<(), Error> {
    if !std::io::stdin().is_terminal() {
        return Err(Error::bad_input(
            "init is interactive; run from a terminal".to_string(),
        ));
    }

    println!("loci init");
    println!("---------");
    println!("Interactive setup. Press Ctrl-C to abort.\n");

    let default =
        default_palace.or_else(|| std::env::current_dir().ok().map(|p| p.display().to_string()));
    let palace_path = prompt("Palace path", default.as_deref())?;
    let trimmed = palace_path.trim();
    if !trimmed.is_empty() && palace::validate(std::path::Path::new(trimmed)).is_none() {
        eprintln!(
            "  warning: '{trimmed}' does not look like a palace (no PALACE.md or CLAUDE.md at root, or no rooms). Saving anyway."
        );
    }

    let cfg = Config {
        palace_path: if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        },
    };

    let cfg_dir = dirs::config_dir()
        .ok_or_else(|| Error::io("could not resolve user config dir".to_string()))?
        .join("loci");
    std::fs::create_dir_all(&cfg_dir)?;
    let cfg_path = cfg_dir.join("config.toml");
    let serialized = toml::to_string_pretty(&cfg)
        .map_err(|e| Error::io(format!("serialize config: {e}")))?;
    std::fs::write(&cfg_path, serialized)?;
    println!("\nwrote {}", cfg_path.display());
    Ok(())
}

fn prompt(label: &str, default: Option<&str>) -> Result<String, Error> {
    let mut out = std::io::stdout();
    match default {
        Some(d) => write!(out, "{label} [{d}]: ")?,
        None => write!(out, "{label}: ")?,
    }
    out.flush()?;
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    let trimmed = buf.trim();
    if trimmed.is_empty() {
        Ok(default.unwrap_or("").to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

// ── helpers ─────────────────────────────────────────────────────────────

/// Read an audit marker file. Missing is `None`; any other read failure is an
/// error, never an empty marker.
fn read_marker(path: &Path) -> Result<Option<String>, Error> {
    match std::fs::read_to_string(path) {
        Ok(s) => Ok(Some(s)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(Error::io(format!(
            "cannot read {}: {e}. The receipt's completeness is unknown.",
            path.display()
        ))),
    }
}

fn require_palace(arg: Option<PathBuf>) -> Result<palace::Palace, Error> {
    match palace::detect(arg) {
        Ok(Some(p)) => Ok(p),
        Ok(None) => Err(Error::not_found(
            "no palace found. Pass --palace <path>, set LOCI_PALACE, or run from inside a palace."
                .to_string(),
        )),
        Err(palace::BadEnv(p)) => Err(Error::not_found(format!(
            "LOCI_PALACE is set to {} but no palace is there. Fix or unset it; \
             loci does not fall back to the current directory.",
            p.display()
        ))),
    }
}

// ── Error ────────────────────────────────────────────────────────────────

#[derive(Debug)]
struct Error {
    msg: String,
    kind: ErrKind,
}

#[derive(Debug, Clone, Copy)]
enum ErrKind {
    NotFound,
    BadInput,
    Io,
}

impl Error {
    fn kind(&self) -> ErrKind {
        self.kind
    }
    fn not_found(msg: String) -> Self {
        Self {
            msg,
            kind: ErrKind::NotFound,
        }
    }
    fn bad_input(msg: String) -> Self {
        Self {
            msg,
            kind: ErrKind::BadInput,
        }
    }
    fn io(msg: String) -> Self {
        Self {
            msg,
            kind: ErrKind::Io,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::io(e.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::io(format!("json: {e}"))
    }
}

// Keep Read in scope to silence the unused-import lint when no command uses it.
#[allow(dead_code)]
fn _read_used(_r: &dyn Read) {}

#[cfg(test)]
mod overview_tests {
    use super::*;
    use std::collections::BTreeSet;

    /// The chokepoint: every clap subcommand must be grouped in the overview
    /// exactly once. Add a command without grouping it and this fails, instead
    /// of the command silently missing from `loci overview`.
    #[test]
    fn every_command_is_grouped_exactly_once() {
        let app = Cli::command();
        let clap_names: BTreeSet<String> = app
            .get_subcommands()
            .map(|c| c.get_name().to_string())
            .filter(|n| n != "help")
            .collect();

        let mut grouped: Vec<String> = Vec::new();
        for (_, cmds) in OVERVIEW_GROUPS {
            for (n, _) in *cmds {
                grouped.push((*n).to_string());
            }
        }
        let grouped_set: BTreeSet<String> = grouped.iter().cloned().collect();

        assert_eq!(
            grouped.len(),
            grouped_set.len(),
            "a command is listed in more than one overview group"
        );
        assert_eq!(
            clap_names, grouped_set,
            "overview groups must cover every command exactly once (clap vs OVERVIEW_GROUPS)"
        );
    }

    /// Every command listed in the overview must carry a non-empty description
    /// sourced from clap, proving the descriptions really come from `about`.
    #[test]
    fn every_grouped_command_has_a_clap_about() {
        let abouts = command_abouts();
        for (_, cmds) in OVERVIEW_GROUPS {
            for (n, _) in *cmds {
                assert!(
                    abouts.get(*n).is_some_and(|s| !s.is_empty()),
                    "command `{n}` has no clap `about` (add a doc comment on its Cmd variant)"
                );
            }
        }
    }
}

#[cfg(test)]
mod rain_tests {
    use super::*;

    /// `--fire` is gone: it must say so (bad input, exit 3) and start nothing,
    /// even with no palace around, rather than fail as an unknown flag.
    #[test]
    fn fire_is_refused_with_a_clear_message_and_spawns_nothing() {
        let err = cmd_rain(Some(PathBuf::from("/nonexistent-loci-rain-test")), true, false)
            .expect_err("--fire must be refused");
        assert!(matches!(err.kind(), ErrKind::BadInput));
        assert!(err.to_string().contains("removed in 2.0"), "got: {err}");
    }

    /// The hidden flag must not show in `loci rain --help`.
    #[test]
    fn fire_is_hidden_from_help() {
        let mut app = Cli::command();
        let rain = app.find_subcommand_mut("rain").expect("rain command");
        let help = rain.render_long_help().to_string();
        assert!(!help.contains("--fire"), "help still advertises --fire:\n{help}");
    }
}

#[cfg(test)]
mod audit_tests {
    use super::*;

    #[test]
    fn missing_marker_is_none_unreadable_marker_is_an_error() {
        let dir = std::env::temp_dir().join(format!("loci_cli_marker_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert!(matches!(read_marker(&dir.join("egress.degraded")), Ok(None)));
        // A directory where the marker should be cannot be read as text.
        let blocked = dir.join("egress.disabled");
        std::fs::create_dir_all(&blocked).unwrap();
        assert!(read_marker(&blocked).is_err(), "an unreadable marker must not read as empty");
        std::fs::write(dir.join("egress.degraded"), "2026-07-14T00:00:00Z write failed\n").unwrap();
        assert!(matches!(read_marker(&dir.join("egress.degraded")), Ok(Some(_))));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
