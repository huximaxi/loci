//! loci: read your local palace from the terminal.
//!
//! Five read-shaped commands. No network. No inference. No daemons.
//! Honest about what it is: a CLI that knows the palace layout and prints what's there.

use clap::{Parser, Subcommand};
use serde::Serialize;
use std::io::{IsTerminal, Read, Write};
use std::path::{Path, PathBuf};

use loci_wal::{ChainError, EgressClass, Frame, ProofBundle, Wal};
use std::process::ExitCode;

mod palace;

#[derive(Parser)]
#[command(
    name = "loci",
    version,
    about = "Read your local palace from the terminal.",
    long_about = "Read your local palace from the terminal.\n\n\
                  loci is the plain-text firmware for a persistent, private cognitive system.\n\
                  This CLI walks the palace structure and prints what's there. Read-only.\n\
                  No network, no inference, no daemons. The companion app and the templates\n\
                  kit are the other two doors; the CLI is the terminal-native one."
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
    /// Interactive setup wizard. Writes `~/.config/loci/config.toml`.
    Init,
    /// Egress receipt: what left the device, grouped by class, over the live WAL,
    /// with a hash-chain sanity check. For cryptographic proof, export a bundle and `wal verify` it.
    Audit {
        /// WAL path (default: ~/.loci/wal/egress.jsonl).
        #[arg(long)]
        wal: Option<PathBuf>,
        /// Only count frames at/after this ISO-8601 UTC timestamp (lexicographic).
        #[arg(long)]
        since: Option<String>,
    },
    /// Proof-bundle tools (verify an exported egress receipt).
    Wal {
        #[command(subcommand)]
        cmd: WalCmd,
    },
}

#[derive(Subcommand)]
enum WalCmd {
    /// Verify an exported proof bundle offline (pure, no network).
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
        Cmd::Init => cmd_init(),
        Cmd::Audit { wal, since } => cmd_audit(wal, since, cli.json),
        Cmd::Wal { cmd } => match cmd {
            WalCmd::Verify { bundle, expect_key } => {
                cmd_wal_verify(&bundle, expect_key.as_deref(), cli.json)
            }
        },
    }
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
    let degraded_content =
        std::fs::read_to_string(path.with_file_name("egress.degraded")).unwrap_or_default();
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
    let disabled_at = std::fs::read_to_string(path.with_file_name("egress.disabled"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|t| !t.is_empty() && since.as_deref().map_or(true, |s| t.as_str() >= s));
    let all = Wal::open(&path).read()?;
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
            classes,
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!("egress receipt : {}", path.display());
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
    if all.is_empty() {
        println!("frames         : 0 (no egress recorded)");
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

// ── init ─────────────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize)]
struct Config {
    palace_path: Option<String>,
    backend: Backend,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Backend {
    kind: String,
    endpoint: String,
    model: String,
}

fn cmd_init() -> Result<(), Error> {
    if !std::io::stdin().is_terminal() {
        return Err(Error::bad_input(
            "init is interactive; run from a terminal".to_string(),
        ));
    }

    println!("loci init");
    println!("---------");
    println!("Interactive setup. Press Ctrl-C to abort.\n");

    let palace_path = prompt(
        "Palace path",
        std::env::current_dir()
            .ok()
            .map(|p| p.display().to_string())
            .as_deref(),
    )?;
    let trimmed = palace_path.trim();
    if !trimmed.is_empty() && palace::validate(std::path::Path::new(trimmed)).is_none() {
        eprintln!(
            "  warning: '{trimmed}' does not look like a palace (no PALACE.md or CLAUDE.md at root, or no rooms). Saving anyway."
        );
    }

    // Backend: this slice is Ollama-only. Other backends land in a later release.
    println!("\nAI backend: ollama (this slice ships only the local backend).");
    let endpoint = prompt("Ollama endpoint", Some("http://localhost:11434"))?;
    let model = prompt("Ollama model", Some("qwen3:8b"))?;

    let cfg = Config {
        palace_path: if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        },
        backend: Backend {
            kind: "ollama".to_string(),
            endpoint: endpoint.trim().to_string(),
            model: model.trim().to_string(),
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

fn require_palace(arg: Option<PathBuf>) -> Result<palace::Palace, Error> {
    palace::detect(arg).ok_or_else(|| {
        Error::not_found(
            "no palace found. Pass --palace <path>, set LOCI_PALACE, or run from inside a palace."
                .to_string(),
        )
    })
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
