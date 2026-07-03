//! loci: read your local palace from the terminal.
//!
//! Read-shaped commands. No network. No inference. No daemons.
//! Honest about what it is: a CLI that knows the palace layout and prints what's there.
//! The one hand-off: `rain --fire` execs your agent runtime and exits; the CLI
//! itself still does no inference.

use clap::{Parser, Subcommand};
use serde::Serialize;
use std::io::{IsTerminal, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

mod palace;
mod tokens;

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
    /// Approximate agent-runtime session-window status (5h rolling) from local transcripts.
    Tokens,
    /// Garden watering weather: window headroom + garden state + how to fire a rain round.
    Rain {
        /// Hand off to the agent runtime now (`claude -p "rain"` from the palace root).
        #[arg(long)]
        fire: bool,
    },
    /// Interactive setup wizard. Writes `~/.config/loci/config.toml`.
    Init,
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
        Cmd::Rain { fire } => cmd_rain(cli.palace, fire, cli.json),
        Cmd::Init => cmd_init(),
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
    let p = require_palace(palace_arg)?;
    let garden = rain_garden(&p);
    let window = tokens::window_status();
    let invocation = "claude -p \"rain\"".to_string();

    if fire {
        println!(
            "firing rain from {} ({} {})…",
            p.root.display(),
            signal_glyph(&window.signal),
            window.signal
        );
        let status = std::process::Command::new("claude")
            .args(["-p", "rain"])
            .current_dir(&p.root)
            .status()
            .map_err(|e| {
                Error::io(format!(
                    "could not launch agent runtime `claude`: {e}. Is it on PATH?"
                ))
            })?;
        if !status.success() {
            return Err(Error::io(format!("rain round exited with {status}")));
        }
        return Ok(());
    }

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
    println!("fire    : loci rain --fire   (or in-session: Workflow({{ name: \"rain\" }}))");
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
