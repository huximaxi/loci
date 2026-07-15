// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

// ─── 1B: MCP server ───────────────────────────────────────────────────────────
mod mcp;

// ─── Phase 4a: pluggable inference (trait, not vendor) ────────────────────────
mod inference;
use inference::{ClaudeBackend, EgressLogged, InferenceBackend, OllamaBackend};

// ─── Loci config (persisted to ~/.loci/config.json) ──────────────────────────
//
// LociRustConfig mirrors the TypeScript LociConfig type in packages/core/src/types.ts.
// Stored as pretty-printed JSON at ~/.loci/config.json.
// Read by the Tauri backend at startup and on-demand via Tauri commands.

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct LociRustConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    ollama: Option<OllamaRustConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mcp: Option<McpRustConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    palace_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
struct OllamaRustConfig {
    enabled: bool,
    base_url: String,
    chat_model: String,
    embed_model: String,
    offline_mode: bool,
    fail_closed: bool,
}

impl Default for OllamaRustConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: "http://localhost:11434".to_string(),
            chat_model: "llama3".to_string(),
            embed_model: "nomic-embed-text".to_string(),
            offline_mode: false,
            fail_closed: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct McpRustConfig {
    enabled: bool,
    port: u16,
    expose_rooms: Vec<String>,
}

impl Default for McpRustConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 3456,
            expose_rooms: vec![],
        }
    }
}

fn loci_config_path() -> Result<std::path::PathBuf, String> {
    dirs::home_dir()
        .ok_or_else(|| "could not determine home directory".to_string())
        .map(|h| h.join(".loci").join("config.json"))
}

// ─── 1A: Ollama local inference ───────────────────────────────────────────────
//
// Cipher gate (non-negotiable):
//   1. base_url is validated before any HTTP call — localhost / 127.0.0.1 / Tailscale (100.x) only.
//      Arbitrary URLs are rejected to prevent SSRF.
//   2. offline_mode=true → commands return Err("ollama_offline") immediately.
//      No silent external fallback. Ever.
//   3. A single shared reqwest::Client is held in Tauri managed state for connection reuse.
//
// TODO(v2): extract to src/ollama.rs when this module grows.

/// Managed state: one shared HTTP client for the app lifetime.
struct OllamaState {
    client: reqwest::Client,
}

// ─── Ollama API response shapes ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModelInfo>,
}

#[derive(Debug, Deserialize)]
struct OllamaModelInfo {
    name: String,
}

#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaChatMessage>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    choices: Vec<OllamaChatChoice>,
}

#[derive(Debug, Deserialize)]
struct OllamaChatChoice {
    message: OllamaChatMessage,
}

#[derive(Debug, Serialize)]
struct OllamaEmbedRequest {
    model: String,
    prompt: String,
}

#[derive(Debug, Deserialize)]
struct OllamaEmbedResponse {
    embedding: Vec<f32>,
}

/// Inference readiness status — returned to the frontend gate and Settings page.
#[derive(Debug, Serialize, Clone)]
struct InferenceStatus {
    /// Ollama is healthy AND has at least one model installed.
    has_local: bool,
    /// Ollama responds at the configured URL (may have zero models).
    ollama_running: bool,
    /// Claude Code CLI was found and responds.
    has_claude: bool,
    /// Names of installed Ollama models (empty when Ollama is offline).
    local_models: Vec<String>,
}

// ─── URL validation (Cipher gate) ────────────────────────────────────────────
//
// Accepts:
//   - http://localhost:*      (local loop, IPv4 name)
//   - http://127.0.0.1:*     (local loop, IPv4 literal)
//   - http://[::1]:*         (local loop, IPv6)
//   - http://100.*.*.*:*     (Tailscale mesh — for 2H integration)
//
// Rejects everything else. HTTPS is accepted for any of the above (e.g., Tailscale HTTPS).

fn validate_ollama_url(raw: &str) -> Result<url::Url, String> {
    let parsed = url::Url::parse(raw)
        .map_err(|e| format!("invalid base_url '{}': {}", raw, e))?;

    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(format!("base_url must use http or https scheme, got '{}'", scheme));
    }

    let host = parsed.host_str().unwrap_or("");

    let is_localhost = host == "localhost" || host == "127.0.0.1" || host == "[::1]" || host == "::1";
    let is_tailscale = host.starts_with("100.") && {
        // Validate it's a real Tailscale CGNAT address (100.64.0.0/10)
        let parts: Vec<&str> = host.splitn(4, '.').collect();
        if parts.len() >= 2 {
            parts[1].parse::<u8>().map(|n| n >= 64).unwrap_or(false)
        } else {
            false
        }
    };

    if !is_localhost && !is_tailscale {
        return Err(format!(
            "base_url '{}' is not permitted. Allowed: localhost, 127.0.0.1, [::1], or Tailscale IP (100.64.x.x–100.127.x.x)",
            host
        ));
    }

    Ok(parsed)
}

// ─── Tauri commands ───────────────────────────────────────────────────────────

/// Check whether the Ollama daemon is reachable at `base_url`.
/// Returns true if healthy, false if not reachable (never errors — callers use this as a probe).
#[tauri::command]
async fn check_ollama_health(
    state: tauri::State<'_, OllamaState>,
    base_url: Option<String>,
) -> Result<bool, String> {
    let raw = base_url.as_deref().unwrap_or("http://localhost:11434");
    let base = validate_ollama_url(raw)?;
    let url = base.join("/api/tags").map_err(|e| e.to_string())?;

    match state.client.get(url).send().await {
        Ok(resp) => Ok(resp.status().is_success()),
        Err(_) => Ok(false), // unreachable = not healthy, not an error
    }
}

/// List all model names available in the local Ollama instance.
/// Returns Err("ollama_offline") if unreachable (never silently falls back).
#[tauri::command]
async fn list_ollama_models(
    state: tauri::State<'_, OllamaState>,
    base_url: Option<String>,
) -> Result<Vec<String>, String> {
    let raw = base_url.as_deref().unwrap_or("http://localhost:11434");
    let base = validate_ollama_url(raw)?;
    let url = base.join("/api/tags").map_err(|e| e.to_string())?;

    let resp = state
        .client
        .get(url)
        .send()
        .await
        .map_err(|_| "ollama_offline".to_string())?;

    if !resp.status().is_success() {
        return Err(format!("ollama returned status {}", resp.status()));
    }

    let body: OllamaTagsResponse = resp.json().await.map_err(|e| e.to_string())?;
    Ok(body.models.into_iter().map(|m| m.name).collect())
}

/// Send a single prompt to an Ollama chat model and return the full response text.
///
/// Uses the OpenAI-compatible `/v1/chat/completions` endpoint so the same logic
/// works with any drop-in compatible server (LM Studio, llama.cpp, etc.).
///
/// Returns Err("ollama_offline") if the daemon is not reachable.
#[tauri::command]
async fn call_ollama(
    state: tauri::State<'_, OllamaState>,
    prompt: String,
    model: String,
    base_url: Option<String>,
) -> Result<String, String> {
    let raw = base_url.as_deref().unwrap_or("http://localhost:11434");
    let base = validate_ollama_url(raw)?;
    let url = base.join("/v1/chat/completions").map_err(|e| e.to_string())?;

    let body = OllamaChatRequest {
        model,
        messages: vec![OllamaChatMessage {
            role: "user".to_string(),
            content: prompt,
        }],
        stream: false,
    };

    let resp = state
        .client
        .post(url)
        .json(&body)
        .send()
        .await
        .map_err(|_| "ollama_offline".to_string())?;

    if !resp.status().is_success() {
        return Err(format!("ollama returned status {}", resp.status()));
    }

    let parsed: OllamaChatResponse = resp.json().await.map_err(|e| e.to_string())?;
    parsed
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or_else(|| "ollama returned empty choices".to_string())
}

/// Embed `text` using the specified model (default: `nomic-embed-text`).
///
/// Returns a float vector suitable for cosine similarity search.
/// Returns Err("ollama_offline") if the daemon is not reachable.
#[tauri::command]
async fn embed_text(
    state: tauri::State<'_, OllamaState>,
    text: String,
    model: Option<String>,
    base_url: Option<String>,
) -> Result<Vec<f32>, String> {
    let raw = base_url.as_deref().unwrap_or("http://localhost:11434");
    let base = validate_ollama_url(raw)?;
    let url = base.join("/api/embeddings").map_err(|e| e.to_string())?;

    let body = OllamaEmbedRequest {
        model: model.unwrap_or_else(|| "nomic-embed-text".to_string()),
        prompt: text,
    };

    let resp = state
        .client
        .post(url)
        .json(&body)
        .send()
        .await
        .map_err(|_| "ollama_offline".to_string())?;

    if !resp.status().is_success() {
        return Err(format!("ollama returned status {}", resp.status()));
    }

    let parsed: OllamaEmbedResponse = resp.json().await.map_err(|e| e.to_string())?;
    Ok(parsed.embedding)
}

// ─── Phase 4a: chat as QUERY ──────────────────────────────────────────────────
//
// The first job of the chat field is to ANSWER, not act (read-path, lowest risk).
// Config-driven: base_url / model / offline_mode come from ~/.loci/config.json,
// so the privacy posture lives in config, not in each call site. Routes through
// the InferenceBackend trait — the command never names a vendor beyond picking
// which trait impl to construct.

/// Where the egress receipt WAL lives, shared with `loci audit`: ~/.loci/wal/egress.jsonl.
fn egress_wal_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".loci")
        .join("wal")
        .join("egress.jsonl")
}

#[tauri::command]
async fn chat_query(
    state: tauri::State<'_, OllamaState>,
    prompt: String,
    // Which brain: "local" (default, Ollama) or "claude"/"external" (the online
    // garden). The frontend sends this from an explicit, marked toggle — there is
    // no silent fallback from one to the other.
    provider: Option<String>,
) -> Result<String, String> {
    let full_cfg = read_loci_config();
    let cfg = full_cfg.ollama.clone().unwrap_or_default();

    // don't-disturb (offline_mode) is the deepest floor: it blocks ALL inference,
    // local OR external. Nothing is sent anywhere.
    if cfg.offline_mode {
        return Err("do-not-disturb is on — inference is paused, nothing was sent anywhere".into());
    }

    // Grounding is built ABOVE the backend so every brain answers in the assistant voice.
    let system = build_grounding(&full_cfg.palace_path);

    match provider.as_deref() {
        // ── ONLINE GARDEN ──────────────────────────────────────────────────────
        // Explicit, opt-in, MARKED in the UI. Leaves the local garden for
        // Anthropic's API on the user's own license (Claude Code CLI, OAuth sub —
        // no API key). Never reached as a fallback: only when the UI asks for it.
        Some("claude") | Some("external") | Some("anthropic") => {
            let (bin, path_env) = resolve_claude().ok_or(
                "external brain unavailable — Claude Code CLI not found on this machine",
            )?;
            let backend = EgressLogged::new(ClaudeBackend { bin, path_env }, egress_wal_path());
            if !backend.health().await {
                return Err(
                    "Claude CLI is present but won't run (node missing from PATH, or not logged in?)".into(),
                );
            }
            // Pro subscriptions include Sonnet; default to it. (Configurable later.)
            backend.chat(&system, &prompt, "sonnet").await
        }
        // ── LOCAL GARDEN (default) ───────────────────────────────────────────────
        // Active by default; availability decided by reachability, not an opt-in
        // (design ruling 2026-05-26). Privacy-by-default is satisfied by locality.
        _ => {
            // SSRF gate: reject any base_url that isn't localhost / [::1] / Tailscale.
            let base = validate_ollama_url(&cfg.base_url)?;
            let backend = EgressLogged::new(
                OllamaBackend {
                    client: state.client.clone(),
                    base,
                },
                egress_wal_path(),
            );
            // Probe first so an unreachable daemon reads as an honest message,
            // not a 120s hang. The trait's health() never errors.
            if !backend.health().await {
                return Err("the local garden is asleep — is Ollama running? (fail-closed: no online fallback)".into());
            }
            // Resolve against installed models so a stale config still works:
            // "llama3" → "llama3.2:latest" rather than a bare 404.
            let model = backend.resolve_model(&cfg.chat_model).await?;
            backend.chat(&system, &prompt, &model).await
        }
    }
}

/// Resolve the Claude Code CLI: an absolute binary path plus a PATH that includes
/// node. A bundled `.app` has a minimal PATH and cannot see shell aliases, and the
/// CLI is a node script whose shebang needs `node` (which lives in nvm/homebrew).
/// Returns None when no claude binary is found.
fn resolve_claude() -> Option<(PathBuf, std::ffi::OsString)> {
    let home = dirs::home_dir()?;
    let candidates = [
        std::env::var("LOCI_CLAUDE_BIN").ok().map(PathBuf::from),
        Some(home.join("claude-code-local/node_modules/.bin/claude")),
        Some(home.join(".claude/local/claude")),
        Some(PathBuf::from("/opt/homebrew/bin/claude")),
        Some(PathBuf::from("/usr/local/bin/claude")),
    ];
    let bin = candidates.into_iter().flatten().find(|p| p.exists())?;

    // Build the PATH the node-based CLI needs: every nvm node version dir that
    // actually has `node`, the usual brew/system bins, and the claude bin's dir.
    let mut dirs_to_add: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = fs::read_dir(home.join(".nvm/versions/node")) {
        for e in entries.flatten() {
            let b = e.path().join("bin");
            if b.join("node").exists() {
                dirs_to_add.push(b);
            }
        }
    }
    for p in ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"] {
        let pb = PathBuf::from(p);
        if pb.is_dir() {
            dirs_to_add.push(pb);
        }
    }
    if let Some(parent) = bin.parent() {
        dirs_to_add.push(parent.to_path_buf());
    }

    let mut path = dirs_to_add
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(":");
    if let Some(inherited) = std::env::var_os("PATH").and_then(|p| p.into_string().ok()) {
        path.push(':');
        path.push_str(&inherited);
    }
    Some((bin, std::ffi::OsString::from(path)))
}

// ─── 1B: MCP server managed state ────────────────────────────────────────────
//
// McpServerHandle holds the shutdown sender for the running MCP server.
// When None, no server is running. Wrapped in Mutex for interior mutability
// inside Tauri's managed state.
//
// Cipher gate: port range validated (1024–65535) before bind.
// The server itself always binds to 127.0.0.1 — see mcp/server.rs.

struct McpServerHandle {
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    port: Option<u16>,
}

/// Start the MCP server. Idempotent: if already running, returns the current port.
///
/// - `port`: port to bind (default: 3456). Must be 1024–65535.
/// - `loci_dir`: override for the loci base directory (default: `~/.loci/`).
#[tauri::command]
async fn start_mcp_server(
    state: tauri::State<'_, Mutex<McpServerHandle>>,
    port: Option<u16>,
    loci_dir: Option<String>,
) -> Result<u16, String> {
    // --- Cipher gate: port validation ---
    let requested_port = port.unwrap_or(3456);
    if requested_port < 1024 {
        return Err(format!(
            "port {} is reserved (< 1024). Use a port in range 1024–65535.",
            requested_port
        ));
    }

    // If already running, return current port
    {
        let handle = state.lock().map_err(|e| e.to_string())?;
        if handle.shutdown_tx.is_some() {
            return Ok(handle.port.unwrap_or(requested_port));
        }
    }

    // Resolve loci base directory
    let base = if let Some(dir) = loci_dir {
        std::path::PathBuf::from(dir)
    } else {
        dirs::home_dir()
            .ok_or("could not determine home directory")?
            .join(".loci")
    };

    // Create the loci dir if it doesn't exist
    std::fs::create_dir_all(&base)
        .map_err(|e| format!("failed to create loci base dir: {}", e))?;

    // Read expose_rooms from persisted config (empty = expose all rooms)
    let expose_rooms = read_loci_config()
        .mcp
        .map(|m| m.expose_rooms)
        .unwrap_or_default();

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let actual_port = mcp::server::start_server(requested_port, base, expose_rooms, shutdown_rx)
        .await
        .map_err(|e| format!("MCP server failed to start: {}", e))?;

    {
        let mut handle = state.lock().map_err(|e| e.to_string())?;
        handle.shutdown_tx = Some(shutdown_tx);
        handle.port = Some(actual_port);
    }

    Ok(actual_port)
}

/// Stop the running MCP server. No-op if not running.
#[tauri::command]
async fn stop_mcp_server(
    state: tauri::State<'_, Mutex<McpServerHandle>>,
) -> Result<(), String> {
    let mut handle = state.lock().map_err(|e| e.to_string())?;
    if let Some(tx) = handle.shutdown_tx.take() {
        // send() errors if receiver already dropped — fine to ignore
        let _ = tx.send(());
        handle.port = None;
    }
    Ok(())
}

/// Return the current MCP server status.
#[tauri::command]
fn mcp_server_status(
    state: tauri::State<'_, Mutex<McpServerHandle>>,
) -> serde_json::Value {
    let handle = match state.lock() {
        Ok(h) => h,
        Err(_) => return serde_json::json!({ "running": false }),
    };
    serde_json::json!({
        "running": handle.shutdown_tx.is_some(),
        "port": handle.port,
    })
}

// ─── Config persistence ───────────────────────────────────────────────────────

/// Read the persisted Loci config from ~/.loci/config.json.
/// Returns defaults if the file does not exist or cannot be parsed.
/// Never errors — callers always get a valid config.
#[tauri::command]
fn read_loci_config() -> LociRustConfig {
    let path = match loci_config_path() {
        Ok(p) => p,
        Err(_) => return LociRustConfig::default(),
    };
    if !path.exists() {
        return LociRustConfig::default();
    }
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return LociRustConfig::default(),
    };
    serde_json::from_str(&content).unwrap_or_default()
}

/// Write the Loci config to ~/.loci/config.json (pretty-printed JSON).
/// Creates ~/.loci/ if it does not exist.
#[tauri::command]
fn write_loci_config(config: LociRustConfig) -> Result<(), String> {
    let path = loci_config_path()?;
    std::fs::create_dir_all(path.parent().ok_or("no parent dir")?)
        .map_err(|e| format!("failed to create .loci directory: {}", e))?;
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("failed to serialize config: {}", e))?;
    std::fs::write(&path, content)
        .map_err(|e| format!("failed to write config: {}", e))
}

// ─── Palace commands ─────────────────────────────────────────────────────────

/// True if `root` has at least one subdir containing CLAUDE.md (rooms-at-root layout).
/// Prunes the obvious build artefacts so a workspace clone doesn't false-positive.
fn has_room_at_root(root: &Path) -> bool {
    let Ok(entries) = fs::read_dir(root) else {
        return false;
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with('.')
            || name_str == "_palace"
            || name_str == "node_modules"
            || name_str == "target"
            || name_str == "cron"
        {
            continue;
        }
        let p = entry.path();
        if p.is_dir() && p.join("CLAUDE.md").exists() {
            return true;
        }
    }
    false
}

/// Check that `path` is a valid palace root.
///
/// Accepts two layouts:
///   * legacy:        PALACE.md/CLAUDE.md at root + `_palace/` subdir with room dirs inside
///   * rooms-at-root: PALACE.md/CLAUDE.md at root + at least one sibling dir containing CLAUDE.md
///
/// The rooms-at-root shape is what palaces ported from older organic layouts look like
/// (rooms grew at root, never moved into `_palace/`). Both are valid; detection no longer
/// presumes one over the other.
#[tauri::command]
fn validate_palace_path(path: String) -> bool {
    let p = Path::new(&path);
    if !p.exists() || !p.is_dir() {
        return false;
    }
    if !(p.join("PALACE.md").exists() || p.join("CLAUDE.md").exists()) {
        return false;
    }
    p.join("_palace").is_dir() || has_room_at_root(p)
}

/// Open a native directory picker and return the chosen path (or None on cancel).
///
/// Lives in Rust on purpose: a pure-WASM frontend has no `window.__TAURI__.dialog`
/// JS sugar (reaching for it crashes), and the raw `plugin:dialog|open` invoke
/// deadlocks because the native panel never reaches the main thread. The plugin's
/// Rust API dispatches the panel correctly; we bridge its completion callback back
/// to the awaiting command through a oneshot so the frontend just `invoke`s a
/// normal command — same transport the dashboard reads already use.
#[tauri::command]
async fn pick_palace_dir(app: AppHandle, title: String) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_title(title)
        .pick_folder(move |path| {
            let _ = tx.send(path);
        });
    let picked = rx
        .await
        .map_err(|e| format!("dialog channel closed before a choice was made: {e}"))?;
    Ok(picked
        .and_then(|fp| fp.into_path().ok())
        .map(|p| p.to_string_lossy().into_owned()))
}

#[derive(Debug, Serialize)]
struct CronJobState {
    job: String,
    status: String,
    last_run: Option<String>,
    summary: Option<String>,
    ciq: Option<f64>,
    ciq_delta: Option<f64>,
}

#[derive(Debug, Serialize)]
struct PalaceState {
    palace_path: String,
    room_count: usize,
    cron_jobs: Vec<CronJobState>,
    current_focus: String,
    pending_tasks: Vec<String>,
    generated_at: String,
}

/// Read the live state of the palace: cron jobs, current focus, pending tasks.
/// Mirrors the logic in gen-dashboard.py for dashboard parity.
#[tauri::command]
fn read_palace_state(palace_path: String) -> Result<PalaceState, String> {
    let root = Path::new(&palace_path);
    if !root.exists() {
        return Err(format!("palace path not found: {}", palace_path));
    }

    let palace_dir = root.join("_palace");

    // Count rooms (subdirs with CLAUDE.md, excluding cron and dotfiles)
    let room_count = if palace_dir.exists() {
        fs::read_dir(&palace_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        let name = e.file_name();
                        let n = name.to_string_lossy();
                        e.path().is_dir()
                            && n != "cron"
                            && !n.starts_with('.')
                            && e.path().join("CLAUDE.md").exists()
                    })
                    .count()
            })
            .unwrap_or(0)
    } else {
        0
    };

    // Read cron job states
    let mut cron_jobs = Vec::new();
    let cron_dir = palace_dir.join("cron");
    if cron_dir.exists() {
        if let Ok(entries) = fs::read_dir(&cron_dir) {
            let mut dirs: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .collect();
            dirs.sort_by_key(|e| e.file_name());
            for entry in dirs {
                let job_name = entry.file_name().to_string_lossy().to_string();
                let state_file = entry.path().join("state.json");
                if state_file.exists() {
                    if let Ok(content) = fs::read_to_string(&state_file) {
                        let v: serde_json::Value =
                            serde_json::from_str(&content).unwrap_or(serde_json::Value::Null);
                        cron_jobs.push(CronJobState {
                            job: v["job"].as_str().unwrap_or(&job_name).to_string(),
                            status: v["status"].as_str().unwrap_or("unknown").to_string(),
                            last_run: v["last_run"].as_str().map(|s| s.to_string()),
                            summary: v["summary"].as_str().map(|s| s.to_string()),
                            ciq: v["ciq"].as_f64(),
                            ciq_delta: v["ciq_delta"].as_f64(),
                        });
                    } else {
                        cron_jobs.push(CronJobState {
                            job: job_name,
                            status: "error".to_string(),
                            last_run: None,
                            summary: Some("failed to read state.json".to_string()),
                            ciq: None,
                            ciq_delta: None,
                        });
                    }
                }
            }
        }
    }

    let current_focus = extract_md_section(&root.join("CLAUDE.md"), "## Current Focus");
    let pending_tasks = extract_pending_tasks(&palace_dir.join("TASKS.md"));

    Ok(PalaceState {
        palace_path,
        room_count,
        cron_jobs,
        current_focus,
        pending_tasks,
        generated_at: chrono::Utc::now().to_rfc3339(),
    })
}

fn extract_md_section(path: &std::path::PathBuf, header: &str) -> String {
    let Ok(text) = fs::read_to_string(path) else {
        return String::new();
    };
    let mut in_section = false;
    let mut collected: Vec<&str> = Vec::new();
    for line in text.lines() {
        if line.trim() == header {
            in_section = true;
            continue;
        }
        if in_section {
            if line.starts_with("## ") {
                break;
            }
            collected.push(line);
        }
    }
    collected.join("\n").trim().to_string()
}

/// Like `extract_md_section` but matches the header by PREFIX, so headers that
/// carry a parenthetical (e.g. "## Current Focus (top 5–7 themes)") still match.
fn extract_md_section_prefix(path: &std::path::PathBuf, header_prefix: &str) -> String {
    let Ok(text) = fs::read_to_string(path) else {
        return String::new();
    };
    let mut in_section = false;
    let mut collected: Vec<&str> = Vec::new();
    for line in text.lines() {
        if line.starts_with("## ") && line.starts_with(header_prefix) {
            in_section = true;
            continue;
        }
        if in_section {
            if line.starts_with("## ") {
                break;
            }
            collected.push(line);
        }
    }
    collected.join("\n").trim().to_string()
}

/// Build the system prompt that grounds the local brain with the assistant identity.
///
/// Loads three layers of identity grounding:
///   L0  — hard-coded base identity (voice kernel, always present)
///   L1  — palace CLAUDE.md § Identity + § Current Focus
///   L3  — latest handover delta: ## State + ## Next action sections
///
/// Each layer is char-capped to keep total prompt small: local 7B context is
/// precious and first-load latency is the main UX risk.
///
/// NOT included (intentional):
///   - The full ~2845-line VESPER.md (far too large, voice nuance not worth it at 7B)
///   - Room/persona switching (deferred)
/// Branching: PALACE.md without CLAUDE.md → build_companion_grounding (ceremony palaces).
///            CLAUDE.md present → identity grounding (legacy palaces).
/// NEXT: (C) retrieval over soul/room files for full in-character fidelity (persona switching).
fn build_grounding(palace_path: &Option<String>) -> String {
    const FOCUS_CHAR_CAP: usize = 1200;
    const HANDOVER_CHAR_CAP: usize = 800;

    let base = "You are the local assistant for Loci, a private, local-first memory tool. \
You are privacy-native and you value simplicity. Write clearly and directly, first-person \
plural where the voice is shared. Never use em-dashes.";

    let Some(root) = palace_path.as_ref().map(Path::new) else {
        return base.to_string();
    };
    let claude_md = root.join("CLAUDE.md");
    let palace_md_path = root.join("PALACE.md");

    // Branch: PALACE.md without CLAUDE.md = ceremony-created palace.
    // The companion has its own name, origin crystal, and opening moment.
    // Using the base kernel here is the identity confusion bug: it answers
    // questions that should be answered by e.g. "Flux".
    if palace_md_path.is_file() && !claude_md.is_file() {
        return build_companion_grounding(root);
    }

    // L1 — root orientation
    let identity = extract_md_section_prefix(&claude_md, "## Identity");
    let focus_full = extract_md_section_prefix(&claude_md, "## Current Focus");
    // Truncate on a char boundary (palace text has multibyte chars; String::truncate panics mid-codepoint).
    let focus: String = focus_full.chars().take(FOCUS_CHAR_CAP).collect();
    let focus_truncated = focus.chars().count() < focus_full.chars().count();

    // L3 — latest handover delta (State + Next action sections)
    let latest_delta = find_latest_handover_delta(root, HANDOVER_CHAR_CAP);

    let mut out = String::from(base);
    if !identity.is_empty() {
        out.push_str("\n\n# Who you are (from the palace)\n");
        out.push_str(&identity);
    }
    if !focus.is_empty() {
        out.push_str("\n\n# What we are working on right now\n");
        out.push_str(&focus);
        if focus_truncated {
            out.push_str("\n…(truncated for context budget)");
        }
    }
    if !latest_delta.is_empty() {
        out.push_str("\n\n# Recent session state (latest handover)\n");
        out.push_str(&latest_delta);
    }
    out
}

/// Find the most recent handover file and extract its ## State + ## Next action sections.
/// Tries the same three directory conventions as the `read_handovers` command.
/// Returns a char-capped string; empty if no convention matches.
fn find_latest_handover_delta(palace: &Path, char_cap: usize) -> String {
    let scan = palace_scan_root(palace);
    // great-hall mixes handovers with other living docs: prefix-gate it.
    let candidates = [
        (scan.join("handovers"), None),
        (palace.join("handovers"), None),
        (scan.join("great-hall"), Some("_HANDOVER")),
    ];
    let Some((dir, name_prefix)) = candidates.iter().find(|(p, _)| p.is_dir()) else {
        return String::new();
    };
    let Ok(read) = fs::read_dir(dir) else {
        return String::new();
    };

    // Pick newest .md by mtime
    let mut best: Option<(f64, PathBuf)> = None;
    for entry in read.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else { continue };
        if !name.ends_with(".md") || name.starts_with('.') { continue }
        if let Some(prefix) = name_prefix {
            if !name.starts_with(prefix) { continue }
        }
        let Ok(meta) = fs::metadata(&path) else { continue };
        if !meta.is_file() { continue }
        let mtime = meta.modified().ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);
        if best.as_ref().map_or(true, |(best_t, _)| mtime > *best_t) {
            best = Some((mtime, path));
        }
    }
    let Some((_, path)) = best else { return String::new() };

    // Extract the two most grounding sections from the delta format
    let state = extract_md_section_prefix(&path, "## State");
    let next_action = extract_md_section_prefix(&path, "## Next action");

    let mut combined = String::new();
    if !state.is_empty() {
        combined.push_str("## State\n");
        combined.push_str(&state);
    }
    if !next_action.is_empty() {
        if !combined.is_empty() { combined.push('\n'); }
        combined.push_str("## Next action\n");
        combined.push_str(&next_action);
    }
    // Fallback: if no standard sections found, take the top of the file
    if combined.is_empty() {
        combined = fs::read_to_string(&path).unwrap_or_default();
    }

    combined.chars().take(char_cap).collect()
}

// ─── Companion grounding (ceremony-created palaces) ──────────────────────────
//
// Palaces created through the naming ceremony have PALACE.md at their root but
// no CLAUDE.md. `build_grounding` branches here so the chat model speaks
// as the named companion instead of defaulting to the base kernel.

/// Read a `> Field: value` frontmatter line from PALACE.md.
/// Format is  `> Companion: Flux`  — strips `>`, trims, then strips the field prefix.
fn read_palace_field(palace_md: &Path, field: &str) -> Option<String> {
    let text = fs::read_to_string(palace_md).ok()?;
    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with('>') { continue }
        let rest = line[1..].trim();
        if let Some(value) = rest.strip_prefix(field) {
            let v = value.trim();
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

/// Find the greeter's origin crystal in the garden.
/// Primary: `{companion-slug}-origin.md`. Fallback: any `*-origin.md` in garden root.
fn find_greeter_origin_crystal(garden: &Path, companion_name: &str) -> Option<String> {
    let slug = make_slug(companion_name);
    let named = garden.join(format!("{slug}-origin.md"));
    if named.is_file() {
        return fs::read_to_string(&named).ok();
    }
    // Fallback: scan garden root for any *-origin.md
    let Ok(entries) = fs::read_dir(garden) else { return None };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") { continue }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else { continue };
        if name.ends_with("-origin.md") {
            return fs::read_to_string(&path).ok();
        }
    }
    None
}

/// Find the holder's opening moment crystal — written during the naming ceremony
/// and marked `greeter-header: true` in its YAML frontmatter.
fn find_opening_moment_crystal(garden: &Path) -> Option<String> {
    let Ok(entries) = fs::read_dir(garden) else { return None };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") { continue }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else { continue };
        if name.ends_with("-origin.md") { continue } // skip companion's own crystal
        let Ok(content) = fs::read_to_string(&path) else { continue };
        if content.contains("greeter-header: true") {
            return Some(content);
        }
    }
    None
}

/// Build a companion-scoped system prompt for ceremony-created palaces.
/// Reads `> Companion:` and `> Palace holder:` from PALACE.md, then seeds
/// the prompt with the companion's origin crystal and the holder's opening moment.
fn build_companion_grounding(palace_root: &Path) -> String {
    const CRYSTAL_CHAR_CAP: usize = 800;

    let palace_md = palace_root.join("PALACE.md");
    let companion = read_palace_field(&palace_md, "Companion:")
        .filter(|s| s != "unnamed" && !s.is_empty())
        .unwrap_or_else(|| "unnamed".to_string());
    let holder = read_palace_field(&palace_md, "Palace holder:")
        .filter(|s| s != "unknown" && !s.is_empty())
        .unwrap_or_else(|| "the palace holder".to_string());

    let mut out = format!(
        "You are {companion}, the AI companion of this memory palace. \
         The palace holder is {holder}. \
         You are NOT a generic AI assistant. \
         You are NOT Llama or any base model. \
         When asked who you are, say you are {companion}. \
         You live in this palace and speak from what you know about it. \
         Be warm, curious, and specific to what you know about {holder}. \
         Never use em-dashes."
    );

    let garden = palace_scan_root(palace_root).join("garden");

    if let Some(origin) = find_greeter_origin_crystal(&garden, &companion) {
        out.push_str("\n\n# Your origin crystal (who you are in this palace)\n");
        let clipped: String = origin.chars().take(CRYSTAL_CHAR_CAP).collect();
        out.push_str(&clipped);
    }

    if let Some(moment) = find_opening_moment_crystal(&garden) {
        out.push_str(&format!("\n\n# What you first learned about {holder}\n"));
        let clipped: String = moment.chars().take(CRYSTAL_CHAR_CAP).collect();
        out.push_str(&clipped);
    }

    out
}

fn extract_pending_tasks(tasks_path: &std::path::PathBuf) -> Vec<String> {
    let Ok(text) = fs::read_to_string(tasks_path) else {
        return Vec::new();
    };
    text.lines()
        .filter(|l| l.contains("[ ]"))
        .map(|l| l.trim().to_string())
        .collect()
}

#[derive(Debug, Serialize)]
struct RoomInfo {
    name: String,
    path: String,
    file_count: usize,
}

#[derive(Debug, Serialize)]
struct PalaceManifest {
    path: String,
    rooms: Vec<RoomInfo>,
    cron_job_count: usize,
    crystal_count: usize,
    /// Companion name read from `> Companion:` in PALACE.md, or None for legacy palaces.
    /// Used by the dashboard attribution line so it shows the actual companion name
    /// instead of the hardcoded default-name fallback.
    companion: Option<String>,
}

/// Scaffold a fresh palace at parent_path/_palace/.
/// Creates room dirs + seeds CLAUDE.md. Persists palace_path to config.
#[tauri::command]
fn scaffold_palace(parent_path: String) -> Result<String, String> {
    let parent = Path::new(&parent_path);
    let palace = parent.join("_palace");

    for room in &["dev-room", "hatchery", "design-room", "engine-room", "library"] {
        let room_path = palace.join(room);
        fs::create_dir_all(&room_path)
            .map_err(|e| format!("failed to create {}: {}", room, e))?;
        let claude_md = room_path.join("CLAUDE.md");
        if !claude_md.exists() {
            fs::write(&claude_md, format!("# {} Room\n\n*No context yet.*\n", room))
                .map_err(|e| format!("failed to seed CLAUDE.md for {}: {}", room, e))?;
        }
    }
    fs::create_dir_all(palace.join("cron"))
        .map_err(|e| format!("failed to create cron dir: {}", e))?;

    // Seed top-level CLAUDE.md from template if available, else minimal placeholder
    let claude_dest = parent.join("CLAUDE.md");
    if !claude_dest.exists() {
        let template_content = dirs::home_dir()
            .map(|h| h.join("Dev/loci/templates/CLAUDE-master.md"))
            .filter(|p| p.exists())
            .and_then(|p| fs::read_to_string(&p).ok())
            .unwrap_or_else(|| {
                format!(
                    "# Palace — {}\n\n## Current Focus\n\n*Your active themes go here.*\n\n## Pending\n\n*Open tasks go here.*\n",
                    chrono::Utc::now().format("%Y-%m-%d")
                )
            });
        fs::write(&claude_dest, template_content)
            .map_err(|e| format!("failed to create CLAUDE.md: {}", e))?;
    }

    let mut config = read_loci_config();
    config.palace_path = Some(parent_path.clone());
    write_loci_config(config)?;

    Ok(parent_path)
}

/// Load and validate an existing palace directory.
/// Accepts: PALACE.md (native loci format) or CLAUDE.md (legacy) at root,
///          plus _palace/ with at least one room subdir containing CLAUDE.md.
/// Persists palace_path to config on success.
#[tauri::command]
fn load_palace(path: String) -> Result<PalaceManifest, String> {
    let __t = std::time::Instant::now();
    // Derive root from an owned PathBuf so `path` stays free to move into the return struct.
    let root_buf = std::path::PathBuf::from(&path);
    let root = root_buf.as_path();

    if !root.exists() || !root.is_dir() {
        return Err("path does not exist or is not a directory".to_string());
    }
    let has_root_marker = root.join("PALACE.md").exists() || root.join("CLAUDE.md").exists();
    if !has_root_marker {
        return Err("not a palace: neither PALACE.md nor CLAUDE.md found at root".to_string());
    }
    // Pick the layout: rooms-inside-_palace (legacy) OR rooms-at-root (organic ports).
    let palace_dir = root.join("_palace");
    let scan_root: std::path::PathBuf = if palace_dir.is_dir() {
        palace_dir.clone()
    } else if has_room_at_root(root) {
        root.to_path_buf()
    } else {
        return Err(
            "not a palace: no _palace/ subdir and no rooms-at-root (subdirs with CLAUDE.md)"
                .to_string(),
        );
    };

    let mut rooms = Vec::new();
    if let Ok(entries) = fs::read_dir(&scan_root) {
        let mut dirs: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                let n = e.file_name();
                let name = n.to_string_lossy();
                // Prune build artefacts + `_palace` (when scanning at root) + cron.
                e.path().is_dir()
                    && name != "cron"
                    && name != "_palace"
                    && name != "node_modules"
                    && name != "target"
                    && !name.starts_with('.')
            })
            .collect();
        dirs.sort_by_key(|e| e.file_name());
        for entry in dirs {
            let room_path = entry.path();
            if room_path.join("CLAUDE.md").exists() {
                rooms.push(RoomInfo {
                    name: entry.file_name().to_string_lossy().to_string(),
                    path: room_path.to_string_lossy().to_string(),
                    file_count: count_md_files(&room_path),
                });
            }
        }
    }

    if rooms.is_empty() {
        return Err("not a palace: no rooms found (subdirs with CLAUDE.md)".to_string());
    }

    // Cron lives where the layout lives.
    let cron_job_count = fs::read_dir(scan_root.join("cron"))
        .map(|e| e.filter_map(|x| x.ok()).filter(|x| x.path().is_dir()).count())
        .unwrap_or(0);

    let mut config = read_loci_config();
    config.palace_path = Some(path.clone());
    write_loci_config(config)?;

    // Crystal count scans the layout root (prune-guarded; never walks target/node_modules/.git).
    let crystal_count = count_md_files(&scan_root);

    // Companion name: read from PALACE.md `> Companion:` field for ceremony-created
    // palaces. Legacy palaces (CLAUDE.md only) default to None → dashboard uses the default name.
    let companion = read_palace_field(&root_buf.join("PALACE.md"), "Companion:")
        .filter(|s| s != "unnamed" && !s.is_empty());

    eprintln!("[TIMING] load_palace: {} rooms, {} crystals in {:?}", rooms.len(), crystal_count, __t.elapsed());

    Ok(PalaceManifest {
        path,
        rooms,
        cron_job_count,
        crystal_count,
        companion,
    })
}

#[derive(Debug, Serialize, Deserialize)]
struct DetectionResult {
    found: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rooms: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    crystal_count: Option<usize>,
    suggestion: String,
}

#[tauri::command]
fn detect_palace(search_path: String) -> DetectionResult {
    let path = Path::new(&search_path);

    // 1. Check for loci palace (already migrated) — accepts both PALACE.md and CLAUDE.md
    if let Some(home) = dirs::home_dir() {
        let loci_path = home.join(".loci");
        if loci_path.exists()
            && (loci_path.join("PALACE.md").exists() || loci_path.join("CLAUDE.md").exists())
        {
            return DetectionResult {
                found: true,
                kind: Some("loci".to_string()),
                path: Some(loci_path.to_string_lossy().to_string()),
                rooms: Some(detect_rooms(&loci_path)),
                crystal_count: Some(count_crystals(&loci_path)),
                suggestion: "You already have a loci palace at ~/.loci/".to_string(),
            };
        }
    }

    // 1b. Check for native PALACE.md at search_path (loci ceremony-created palace)
    if path.join("PALACE.md").exists() && path.join("_palace").is_dir() {
        return DetectionResult {
            found: true,
            kind: Some("loci-native".to_string()),
            path: Some(path.to_string_lossy().to_string()),
            rooms: Some(detect_rooms(&path.join("_palace"))),
            crystal_count: Some(count_crystals(&path.join("_palace"))),
            suggestion: "Found a native loci palace (PALACE.md). Ready to load.".to_string(),
        };
    }

    // 2. Check for _palace/ pattern (legacy)
    let palace_dir = path.join("_palace");
    if palace_dir.exists() {
        return DetectionResult {
            found: true,
            kind: Some("mempalace".to_string()),
            path: Some(palace_dir.to_string_lossy().to_string()),
            rooms: Some(detect_rooms(&palace_dir)),
            crystal_count: Some(count_crystals(&palace_dir)),
            suggestion: "Found a memory palace. Ready to migrate to loci format.".to_string(),
        };
    }

    // 3. Check for mila-mempalace/ pattern
    let mila_dir = path.join("mila-mempalace");
    if mila_dir.exists() {
        return DetectionResult {
            found: true,
            kind: Some("mila-mempalace".to_string()),
            path: Some(mila_dir.to_string_lossy().to_string()),
            rooms: Some(detect_rooms(&mila_dir)),
            crystal_count: Some(count_crystals(&mila_dir)),
            suggestion: "Found Mila's memory palace. Ready to migrate to loci format.".to_string(),
        };
    }

    // 4. Check for karpathy pattern (LLM folder structure)
    let llm_dir = path.join("LLM");
    if llm_dir.exists() && llm_dir.join("CLAUDE.md").exists() {
        return DetectionResult {
            found: true,
            kind: Some("karpathy".to_string()),
            path: Some(llm_dir.to_string_lossy().to_string()),
            rooms: None,
            crystal_count: Some(count_md_files(&llm_dir)),
            suggestion: "Found Karpathy-style LLM folder. Ready to migrate to loci format.".to_string(),
        };
    }

    // 5. Foreign memory-provider conventions — the PORTABILITY SEAM. This is the
    // read-side mirror of the InferenceBackend "trait not vendor" move: just as
    // the app is agnostic to WHICH brain answers, it should be agnostic to WHAT
    // structure holds the memory. Recognising memory laid down by other tools is
    // how Loci reads ACROSS providers without anyone federating up to a central
    // server (the anti-honeypot moat: portability, not centralisation).
    //
    // STRUCTURE-only detection here. Reading the CONTENT of a foreign store is a
    // separate, Quarantined step (foreign text is never trusted-by-construction).
    //
    // Each signature matches if ALL its markers exist (file or dir) at `path`.
    // Extend with Hermes and bourdon's L0–L6 once their on-disk signatures are
    // confirmed under recon — do NOT guess a recogniser for a shape we haven't seen.
    let signatures: &[(&str, &str, &[&str])] = &[
        ("claude-code", "Found a Claude Code store (CLAUDE.md). Loci can read across to it.", &["CLAUDE.md"]),
        ("agents-md", "Found an AGENTS.md store (Codex / kilo convention). Loci can read across to it.", &["AGENTS.md"]),
        ("cursor", "Found Cursor project rules (.cursor/rules). Loci can read across to it.", &[".cursor/rules"]),
        ("cursor", "Found a Cursor rules file (.cursorrules). Loci can read across to it.", &[".cursorrules"]),
        ("windsurf", "Found Windsurf rules (.windsurfrules). Loci can read across to it.", &[".windsurfrules"]),
    ];
    for (kind, suggestion, markers) in signatures {
        if markers.iter().all(|m| path.join(m).exists()) {
            return DetectionResult {
                found: true,
                kind: Some((*kind).to_string()),
                path: Some(path.to_string_lossy().to_string()),
                rooms: None,
                // count_md_files is now prune-guarded, so scanning a foreign repo
                // root no longer walks target/ or node_modules/.
                crystal_count: Some(count_md_files(path)),
                suggestion: (*suggestion).to_string(),
            };
        }
    }

    // 6. No memory store recognised.
    DetectionResult {
        found: false,
        kind: None,
        path: None,
        rooms: None,
        crystal_count: None,
        suggestion: "No memory palace detected. Would you like to create one?".to_string(),
    }
}

#[tauri::command]
fn migrate_to_loci(source_path: String) -> Result<String, String> {
    let source = Path::new(&source_path);

    let home = dirs::home_dir().ok_or("Could not determine home directory")?;
    let dest = home.join(".loci");

    // Create destination if it doesn't exist
    fs::create_dir_all(&dest).map_err(|e| format!("Failed to create .loci directory: {}", e))?;

    // Copy all contents recursively
    copy_dir_recursive(source, &dest)?;

    // Create loci manifest if it doesn't exist
    let manifest_path = dest.join("loci.json");
    if !manifest_path.exists() {
        let manifest = serde_json::json!({
            "version": "1.0",
            "migrated_from": source.to_string_lossy(),
            "migrated_at": chrono::Utc::now().to_rfc3339(),
        });
        fs::write(manifest_path, serde_json::to_string_pretty(&manifest).unwrap())
            .map_err(|e| format!("Failed to create manifest: {}", e))?;
    }

    Ok(dest.to_string_lossy().to_string())
}

// Helper: count rooms (subdirectories)
fn detect_rooms(path: &Path) -> usize {
    if let Ok(entries) = fs::read_dir(path) {
        entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .count()
    } else {
        0
    }
}

// Helper: count crystals (CLAUDE.md + .md files)
fn count_crystals(path: &Path) -> usize {
    count_md_files(path)
}

// Helper: count all .md files recursively, PRUNING heavy/irrelevant dirs. Without
// the prune list this descends into target/ + node_modules/ + .git/ — tens of
// thousands of build-artifact files — which is what turned a palace scan into a
// 30-second main-thread beachball. Never walk what a palace never keeps.
fn count_md_files(path: &Path) -> usize {
    const PRUNE: &[&str] = &[
        "node_modules", "target", ".git", "dist", "build", ".next", ".cache", "vendor",
    ];
    let mut count = 0;

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                let name = entry.file_name();
                let n = name.to_string_lossy();
                // Skip hidden dirs and known-heavy artifact/dependency trees.
                if n.starts_with('.') || PRUNE.contains(&n.as_ref()) {
                    continue;
                }
                count += count_md_files(&entry_path);
            } else if entry_path.extension().and_then(|s| s.to_str()) == Some("md") {
                count += 1;
            }
        }
    }

    count
}

// Helper: recursive directory copy
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    if !dst.exists() {
        fs::create_dir_all(dst).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    for entry in fs::read_dir(src).map_err(|e| format!("Failed to read directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path).map_err(|e| format!("Failed to copy file: {}", e))?;
        }
    }

    Ok(())
}

// ─── manifest reader + state watcher ─────────────────────────────────────────
//
// Two surfaces:
//
//   1. read_manifest(root_path): typed read of root_path/.schema/manifest.json.
//
//   2. start_state_watcher(root_path): notify-based watcher on root_path/cron/.
//      Filters to state.json files only. Emits "state_changed" events to the
//      frontend with RELATIVE paths only (no absolute-path leakage).
//
// Manifest types are duplicated in src-leptos/src/models.rs to give the WASM
// side strong types over the serde-wasm-bindgen wire. The two definitions MUST
// stay in lockstep.

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Manifest {
    manifest_version: String,
    source_root: String,
    dest_root: String,
    captured_ts_utc: String,
    vocabulary: String,
    tree_hash: String,
    nodes: Vec<ManifestNode>,
    relations: Vec<ManifestRelation>,
    edges: Vec<ManifestEdge>,
    scope: ManifestScope,
    #[serde(default)]
    errors: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ManifestNode {
    rel_path: String,
    sha256: String,
    size: u64,
    mtime: f64,
    mode: u32,
    is_symlink: bool,
    primitive_class: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ManifestRelation {
    rel_path: String,
    sha256: String,
    size: u64,
    mtime: f64,
    mode: u32,
    is_symlink: bool,
    primitive_class: String,
    endpoints: Vec<Option<String>>,
    weight: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ManifestEdge {
    rel_path: String,
    is_symlink: bool,
    symlink_target: String,
    size: u64,
    primitive_class: String,
    #[serde(default)]
    symlink_target_original: Option<String>,
    #[serde(default)]
    flag: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ManifestScope {
    root: String,
}

fn resolve_manifest_path(root_path: &Path) -> Option<PathBuf> {
    // Look for root_path/.schema/manifest.json. The resolved path is by
    // construction inside root_path (literal child).
    let direct = root_path.join(".schema").join("manifest.json");
    if direct.is_file() {
        return Some(direct);
    }
    None
}

#[tauri::command]
fn read_manifest(palace_path: String) -> Result<Manifest, String> {
    let palace = PathBuf::from(&palace_path);
    let manifest_path = resolve_manifest_path(&palace).ok_or_else(|| {
        format!("no .schema/manifest.json found at {}", palace.display())
    })?;
    let __t = std::time::Instant::now();
    let bytes = fs::read(&manifest_path).map_err(|e| format!("read manifest: {e}"))?;
    let manifest: Manifest =
        serde_json::from_slice(&bytes).map_err(|e| format!("parse manifest JSON: {e}"))?;
    eprintln!("[TIMING] read_manifest: {} bytes parsed in {:?}", bytes.len(), __t.elapsed());
    Ok(manifest)
}

/// Slim manifest read for the dashboard schema panel: parses the manifest but
/// returns ONLY counts + meta, not the node graph. The full `read_manifest`
/// ships the entire graph (~180KB, hundreds of nodes) across IPC and forces a
/// large WASM-side deserialize just to display three `.len()` counts — that
/// deserialize is the dashboard's main load hitch. This returns ~200 bytes.
#[derive(Debug, Serialize)]
struct ManifestSummary {
    manifest_version: String,
    vocabulary: String,
    captured_ts_utc: String,
    tree_hash: String,
    node_count: usize,
    relation_count: usize,
    edge_count: usize,
}

#[tauri::command]
fn read_manifest_summary(palace_path: String) -> Result<ManifestSummary, String> {
    let m = read_manifest(palace_path)?;
    Ok(ManifestSummary {
        manifest_version: m.manifest_version,
        vocabulary: m.vocabulary,
        captured_ts_utc: m.captured_ts_utc,
        tree_hash: m.tree_hash,
        node_count: m.nodes.len(),
        relation_count: m.relations.len(),
        edge_count: m.edges.len(),
    })
}

struct WatcherState {
    watcher: Mutex<Option<notify::RecommendedWatcher>>,
}

/// Rooms live either inside `_palace/` (the packaged layout) or at the palace
/// root (organic ports; accepted by load_palace since rc.2). Every read that
/// addresses palace state must resolve through this, not hardcode `_palace/`.
fn palace_scan_root(root: &Path) -> PathBuf {
    let packaged = root.join("_palace");
    if packaged.is_dir() {
        packaged
    } else {
        root.to_path_buf()
    }
}

#[tauri::command]
fn start_state_watcher(
    palace_path: String,
    app: AppHandle,
    state: tauri::State<'_, WatcherState>,
) -> Result<(), String> {
    use notify::{EventKind, RecursiveMode, Watcher};

    let palace = PathBuf::from(&palace_path);
    // Must match read_cron_states: jobs live at <scan_root>/cron in both layouts.
    let cron_dir = palace_scan_root(&palace).join("cron");
    if !cron_dir.is_dir() {
        return Err(format!("cron directory missing: {}", cron_dir.display()));
    }

    let palace_for_thread = palace.clone();
    let app_clone = app.clone();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        let Ok(event) = res else { return };
        if !matches!(
            event.kind,
            EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
        ) {
            return;
        }
        for path in event.paths {
            if path.file_name().and_then(|n| n.to_str()) != Some("state.json") {
                continue;
            }
            // Cipher: strip prefix so absolute palace path never reaches JS.
            // If strip fails the path is outside the palace; drop silently.
            if let Ok(rel) = path.strip_prefix(&palace_for_thread) {
                eprintln!("[TIMING] watcher emit state_changed: {}", rel.display());
                let _ = app_clone.emit("state_changed", rel.to_string_lossy().to_string());
            }
        }
    })
    .map_err(|e| format!("create watcher: {e}"))?;

    watcher
        .watch(&cron_dir, RecursiveMode::Recursive)
        .map_err(|e| format!("watch cron dir: {e}"))?;

    let mut guard = state
        .watcher
        .lock()
        .map_err(|e| format!("watcher lock poisoned: {e}"))?;
    // Dropping the previous watcher (if any) stops it cleanly.
    *guard = Some(watcher);
    Ok(())
}

// ─── v0.6.0 bridge · Phase 3 · cron states + handovers ───────────────────────

#[derive(Debug, Serialize)]
struct CronJobSnapshot {
    // Filesystem dir name under _palace/cron. The stable identity key for
    // detail lookups. Distinct from `job`, which is a content-derived display
    // label (e.g. "cron-job/run") and is NOT safe as a path component.
    key: String,
    job: String,
    status: Option<String>,
    summary: Option<String>,
    last_run: Option<String>,
    ciq: Option<f64>,
    ciq_delta: Option<f64>,
    pulse: Option<String>,
    // alert_count present only on jobs that surface alerts (e.g. alert-watcher-daily).
    alert_count: Option<usize>,
    raw: serde_json::Value,
}

#[tauri::command]
fn read_cron_states(palace_path: String) -> Result<Vec<CronJobSnapshot>, String> {
    let __t = std::time::Instant::now();
    let palace = PathBuf::from(&palace_path);
    let cron_dir = palace_scan_root(&palace).join("cron");
    if !cron_dir.is_dir() {
        // Either fresh scaffold (no jobs yet) or non-standard layout.
        return Ok(Vec::new());
    }
    let mut snapshots = Vec::new();
    let entries = fs::read_dir(&cron_dir).map_err(|e| format!("read cron dir: {e}"))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let state_file = path.join("state.json");
        if !state_file.is_file() {
            continue;
        }
        let Ok(bytes) = fs::read(&state_file) else {
            continue;
        };
        let Ok(raw) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
            continue;
        };
        let dir_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        // Two state.json shapes exist in the wild:
        //   (a) Object: the standard palace-cron pattern with job/status/summary/last_run.
        //   (b) Array: queue-style jobs like inbox-watcher publish a list of records.
        //       Synthesize a summary from the count so the dashboard still surfaces them.
        let snapshot = match &raw {
            serde_json::Value::Array(arr) => CronJobSnapshot {
                key: dir_name.clone(),
                job: dir_name,
                status: Some("ok".into()),
                summary: Some(format!("{} entries", arr.len())),
                last_run: None,
                ciq: None,
                ciq_delta: None,
                pulse: None,
                alert_count: None,
                raw: raw.clone(),
            },
            _ => {
                let job = raw
                    .get("job")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .unwrap_or_else(|| dir_name.clone());
                let alert_count = raw
                    .get("alerts")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len());
                // last_run convention varies: most jobs use "last_run", some use "run_ts_utc".
                let last_run = raw
                    .get("last_run")
                    .or_else(|| raw.get("run_ts_utc"))
                    .and_then(|v| v.as_str())
                    .map(String::from);
                CronJobSnapshot {
                    key: dir_name,
                    job,
                    status: raw.get("status").and_then(|v| v.as_str()).map(String::from),
                    summary: raw.get("summary").and_then(|v| v.as_str()).map(String::from),
                    last_run,
                    ciq: raw.get("ciq").and_then(|v| v.as_f64()),
                    ciq_delta: raw.get("ciq_delta").and_then(|v| v.as_f64()),
                    pulse: raw.get("pulse").and_then(|v| v.as_str()).map(String::from),
                    alert_count,
                    raw,
                }
            }
        };
        snapshots.push(snapshot);
    }
    snapshots.sort_by(|a, b| b.last_run.cmp(&a.last_run));
    eprintln!("[TIMING] read_cron_states: {} jobs in {:?}", snapshots.len(), __t.elapsed());
    Ok(snapshots)
}

#[derive(Debug, Serialize)]
struct HandoverEntry {
    filename: String,
    mtime: f64,
    size: u64,
}

/// Look up recent handovers from a few palace conventions:
///   palace_path/_palace/handovers/            (proposed generic)
///   palace_path/handovers/                    (last-resort)
/// Returns at most `limit` entries, newest first. Empty Vec if no convention matches.
#[tauri::command]
fn read_handovers(palace_path: String, limit: Option<usize>) -> Result<Vec<HandoverEntry>, String> {
    let __t = std::time::Instant::now();
    let palace = PathBuf::from(&palace_path);
    let scan = palace_scan_root(&palace);
    // Dedicated handovers dirs take any .md; the great-hall convention mixes
    // handovers with other living docs, so only `_HANDOVER*` files count there.
    let candidates = [
        (scan.join("handovers"), None),
        (palace.join("handovers"), None),
        (scan.join("great-hall"), Some("_HANDOVER")),
    ];
    let Some((handovers_dir, name_prefix)) =
        candidates.into_iter().find(|(p, _)| p.is_dir())
    else {
        return Ok(Vec::new());
    };
    let mut entries = Vec::new();
    let read = fs::read_dir(&handovers_dir).map_err(|e| format!("read handovers: {e}"))?;
    for entry in read.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.ends_with(".md") || name.starts_with('.') {
            continue;
        }
        if let Some(prefix) = name_prefix {
            if !name.starts_with(prefix) {
                continue;
            }
        }
        let Ok(meta) = fs::metadata(&path) else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);
        entries.push(HandoverEntry {
            filename: name.to_string(),
            mtime,
            size: meta.len(),
        });
    }
    entries.sort_by(|a, b| b.mtime.partial_cmp(&a.mtime).unwrap_or(std::cmp::Ordering::Equal));
    if let Some(n) = limit {
        entries.truncate(n);
    }
    eprintln!("[TIMING] read_handovers in {:?}", __t.elapsed());
    Ok(entries)
}

// ─── v0.6.0 bridge · Phase 3.6 · drill-down detail + questlog ─────────────────

/// Lazy detail load for one cron job. The list view (read_cron_states) strips
/// `raw` to keep the wire payload small; this fetches the full state.json for a
/// single job on demand when the user drills in. Job name is treated as a single
/// path component (no separators) so it cannot escape the cron dir.
#[tauri::command]
fn read_cron_detail(palace_path: String, key: String) -> Result<serde_json::Value, String> {
    if key.is_empty() || key.contains('/') || key.contains('\\') || key.contains("..") {
        return Err(format!("invalid cron key: {key}"));
    }
    let state_file = palace_scan_root(&PathBuf::from(&palace_path))
        .join("cron")
        .join(&key)
        .join("state.json");
    let bytes = fs::read(&state_file).map_err(|e| format!("read {}: {e}", state_file.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("parse {}: {e}", state_file.display()))
}

#[derive(Debug, Serialize)]
struct QuestlogItem {
    done: bool,
    title: String,
    body: String,
    // The `## Heading` section the item sits under in TASKS.md ("Unfiled" if
    // it precedes any heading). This is the track/drive used to group the log.
    track: String,
}

/// Parse open/done items from _palace/TASKS.md. Markdown checkbox convention:
///   - [ ] **date, Title.** body...   → open
///   - [x] ...                         → done
/// Continuation lines (not a new bullet) append to the current item's body, so
/// multi-line items stay whole. Returns open items first, in file order.
#[tauri::command]
fn read_tasks(palace_path: String) -> Result<Vec<QuestlogItem>, String> {
    let __t = std::time::Instant::now();
    let scan = palace_scan_root(&PathBuf::from(&palace_path));
    // Two questlog conventions: the original TASKS.md, and the newer
    // great-hall/QUEST-LOG.md (same checkbox format, different address).
    let tasks_file = [
        scan.join("TASKS.md"),
        scan.join("great-hall").join("QUEST-LOG.md"),
    ]
    .into_iter()
    .find(|p| p.is_file());
    let Some(tasks_file) = tasks_file else {
        return Ok(Vec::new());
    };
    let text = fs::read_to_string(&tasks_file).map_err(|e| format!("read questlog: {e}"))?;
    let mut items: Vec<QuestlogItem> = Vec::new();
    let mut current_track = String::from("Unfiled");
    for line in text.lines() {
        let trimmed = line.trim_start();
        // `## Heading` opens a new track. (Not `#`, which is the file title.)
        if let Some(h) = trimmed.strip_prefix("## ") {
            current_track = h.trim().to_string();
            continue;
        }
        let (done, rest) = if let Some(r) = trimmed.strip_prefix("- [ ]") {
            (false, Some(r))
        } else if let Some(r) = trimmed.strip_prefix("- [x]").or_else(|| trimmed.strip_prefix("- [X]")) {
            (true, Some(r))
        } else {
            (false, None)
        };
        match rest {
            Some(r) => {
                let body = r.trim().to_string();
                items.push(QuestlogItem {
                    done,
                    title: extract_task_title(&body),
                    body,
                    track: current_track.clone(),
                });
            }
            None => {
                // Continuation of the previous item (wrapped line).
                if let Some(last) = items.last_mut() {
                    if !trimmed.is_empty() && !trimmed.starts_with('#') {
                        last.body.push(' ');
                        last.body.push_str(trimmed);
                    }
                }
            }
        }
    }
    // File order preserved: the frontend groups by track and orders within group.
    eprintln!("[TIMING] read_tasks: {} items in {:?}", items.len(), __t.elapsed());
    Ok(items)
}

/// Title = the first **bold** span if present, else the first 80 chars.
fn extract_task_title(body: &str) -> String {
    if let Some(start) = body.find("**") {
        if let Some(end) = body[start + 2..].find("**") {
            return body[start + 2..start + 2 + end].trim().to_string();
        }
    }
    body.chars().take(80).collect()
}

// ─── palace-update: the delta checker ─────────────────────────────────────────
//
// "What's yours to let in." A read-only, explicit update check: read the local
// methodology version anchor, fetch the published one (ONE GET, no telemetry,
// fail-closed), and report the version entries newer than the local version.
// It never writes, never polls, never auto-applies. The human pulls, sees the
// delta, decides. This is the app's only call to the public internet, and it
// runs solely on explicit user action.
//
// `main` is the always-latest methodology line; the `stable` field in the doc
// gates what surfaces by default (candidates are opt-in).

const METHODOLOGY_URL: &str =
    "https://raw.githubusercontent.com/huximaxi/loci/main/PALACE-METHODOLOGY.md";

/// The `> Key: value` version anchor at the top of PALACE-METHODOLOGY.md.
#[derive(Debug, Clone, PartialEq)]
struct MethodologyAnchor {
    /// The leading version, e.g. "1.3-candidate".
    version: String,
    /// The ratified stable version, e.g. "1.2".
    stable: String,
}

/// One bullet within a version section: `**Title.** summary.`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct DeltaItem {
    title: String,
    summary: String,
}

/// One `## vX.Y · DATE` section and its bullets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct DeltaEntry {
    version: String,
    date: String,
    is_candidate: bool,
    items: Vec<DeltaItem>,
}

/// The report handed to the UI. `status` is a plain string for a frictionless
/// WASM boundary: "current" | "behind" | "unknown" | "unavailable".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct UpdateReport {
    /// None when the palace predates the methodology anchor.
    local_version: Option<String>,
    /// The version compared against: `stable`, or `version` when candidates are on.
    latest_version: String,
    status: String,
    /// Version sections strictly newer than local, newest first.
    entries: Vec<DeltaEntry>,
    include_candidates: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UpdateStatus {
    Current,
    Behind,
    Unknown,
    Unavailable,
}

impl UpdateStatus {
    fn as_str(self) -> &'static str {
        match self {
            UpdateStatus::Current => "current",
            UpdateStatus::Behind => "behind",
            UpdateStatus::Unknown => "unknown",
            UpdateStatus::Unavailable => "unavailable",
        }
    }
}

/// Parse "1.3-candidate" / "1.2" into a `(major, minor)` ordering key. An
/// unparseable shape sorts to `(0, 0)` so it never masks a real newer version.
fn version_key(v: &str) -> (u32, u32) {
    let core = v.split('-').next().unwrap_or(v).trim();
    let mut parts = core.split('.');
    let major = parts.next().and_then(|s| s.trim().parse::<u32>().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|s| s.trim().parse::<u32>().ok()).unwrap_or(0);
    (major, minor)
}

/// Read a single `> Field: value` line from methodology markdown text. Mirrors
/// the logic of `read_palace_field` but operates on an in-memory string.
fn methodology_field(md: &str, field: &str) -> Option<String> {
    for line in md.lines() {
        let line = line.trim();
        if !line.starts_with('>') {
            continue;
        }
        let rest = line[1..].trim();
        if let Some(value) = rest.strip_prefix(field) {
            let v = value.trim();
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

/// Parse the version anchor from methodology markdown.
fn parse_anchor(md: &str) -> Option<MethodologyAnchor> {
    Some(MethodologyAnchor {
        version: methodology_field(md, "loci-core version:")?,
        stable: methodology_field(md, "stable:")?,
    })
}

/// Parse `## vX.Y · DATE` sections and their `**Title.** summary` bullets.
fn parse_sections(md: &str) -> Vec<DeltaEntry> {
    let mut out: Vec<DeltaEntry> = Vec::new();
    for line in md.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("## v") {
            // rest e.g. "1.3-candidate · 2026-06-08"
            let mut sp = rest.splitn(2, '·');
            let version = sp.next().unwrap_or("").trim().to_string();
            let date = sp.next().unwrap_or("").trim().to_string();
            let is_candidate = version.contains("candidate");
            out.push(DeltaEntry { version, date, is_candidate, items: Vec::new() });
        } else if t.starts_with("**") {
            if let Some(entry) = out.last_mut() {
                // "**Title.** summary" -> title="Title", summary="summary"
                let after = &t[2..];
                if let Some(end) = after.find("**") {
                    let title = after[..end].trim().trim_end_matches('.').trim().to_string();
                    let summary = after[end + 2..].trim().to_string();
                    if !title.is_empty() {
                        entry.items.push(DeltaItem { title, summary });
                    }
                }
            }
        }
    }
    out
}

/// Pure: the version sections strictly newer than `local`, gated by channel.
/// - candidates off: target = `stable`; candidate sections are hidden.
/// - candidates on:  target = `version` (may be a candidate); candidates show.
/// A missing local version yields `Unknown` and surfaces everything up to target.
fn compute_delta(
    local: Option<&str>,
    anchor: &MethodologyAnchor,
    sections: &[DeltaEntry],
    include_candidates: bool,
) -> (UpdateStatus, String, Vec<DeltaEntry>) {
    let target = if include_candidates { &anchor.version } else { &anchor.stable };
    let target_key = version_key(target);

    let eligible = |s: &&DeltaEntry| -> bool {
        if !include_candidates && s.is_candidate {
            return false;
        }
        version_key(&s.version) <= target_key
    };

    let mut newer: Vec<DeltaEntry> = match local {
        None => sections.iter().filter(eligible).cloned().collect(),
        Some(l) => {
            let local_key = version_key(l);
            sections
                .iter()
                .filter(eligible)
                .filter(|s| version_key(&s.version) > local_key)
                .cloned()
                .collect()
        }
    };
    newer.sort_by(|a, b| version_key(&b.version).cmp(&version_key(&a.version)));

    let status = match local {
        None => UpdateStatus::Unknown,
        Some(_) if newer.is_empty() => UpdateStatus::Current,
        Some(_) => UpdateStatus::Behind,
    };
    (status, target.clone(), newer)
}

/// Explicit, read-only update check. Returns `Ok` with a status in every
/// reachable outcome (so the UI can always show the local version); only an
/// unbuildable HTTP client yields `Err`. Fail-closed: any reach/parse failure
/// becomes `unavailable`, never a silent fallback to another source.
#[tauri::command]
async fn check_for_updates(
    palace_path: String,
    include_candidates: bool,
) -> Result<UpdateReport, String> {
    // 1. Local anchor (read-only, on-device). None if the palace predates it.
    let local_path = Path::new(&palace_path).join("PALACE-METHODOLOGY.md");
    let local_version = fs::read_to_string(&local_path)
        .ok()
        .and_then(|s| parse_anchor(&s))
        .map(|a| a.version);

    let unavailable = |local: &Option<String>| UpdateReport {
        local_version: local.clone(),
        latest_version: String::new(),
        status: UpdateStatus::Unavailable.as_str().to_string(),
        entries: Vec::new(),
        include_candidates,
    };

    // 2. Fetch the published methodology. ONE GET, no user data, 10s timeout.
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("loci")
        .build()
    {
        Ok(c) => c,
        Err(_) => return Ok(unavailable(&local_version)),
    };

    let body = match client.get(METHODOLOGY_URL).send().await {
        Ok(resp) => match resp.error_for_status() {
            Ok(r) => match r.text().await {
                Ok(b) => b,
                Err(_) => return Ok(unavailable(&local_version)),
            },
            Err(_) => return Ok(unavailable(&local_version)),
        },
        Err(_) => return Ok(unavailable(&local_version)),
    };

    let Some(anchor) = parse_anchor(&body) else {
        return Ok(unavailable(&local_version));
    };
    let sections = parse_sections(&body);

    // 3. Compute the delta (pure, on-device).
    let (status, latest_version, entries) =
        compute_delta(local_version.as_deref(), &anchor, &sections, include_candidates);

    Ok(UpdateReport {
        local_version,
        latest_version,
        status: status.as_str().to_string(),
        entries,
        include_candidates,
    })
}

#[cfg(test)]
mod palace_update_tests {
    use super::*;

    // Synthetic fixture: no real palace paths, crystals, or product terms.
    const DOC: &str = "\
> This document tracks the methodology line.

> loci-core version: 1.3-candidate
> stable: 1.2
> status: candidate

## v1.3-candidate · 2026-01-04
**Alpha seam.** does the alpha thing.

## v1.2 · 2026-01-03
**Beta overlay.** does the beta thing.

## v1.1 · 2026-01-02
**Gamma ritual.** does the gamma thing.

## v1.0 · 2026-01-01
**Delta process.** does the delta thing.
**Epsilon pin.** a second bullet.
";

    #[test]
    fn version_key_orders_numerically_not_lexically() {
        assert!(version_key("0.9") < version_key("1.0"));
        assert!(version_key("1.2") < version_key("1.3-candidate"));
        assert!(version_key("1.2") < version_key("1.10")); // not string order
        assert_eq!(version_key("garbage"), (0, 0));
    }

    #[test]
    fn parse_anchor_reads_version_and_stable() {
        let a = parse_anchor(DOC).expect("anchor");
        assert_eq!(a.version, "1.3-candidate");
        assert_eq!(a.stable, "1.2");
    }

    #[test]
    fn parse_sections_splits_title_and_summary() {
        let s = parse_sections(DOC);
        assert_eq!(s.len(), 4);
        assert_eq!(s[0].version, "1.3-candidate");
        assert!(s[0].is_candidate);
        assert_eq!(s[0].items[0].title, "Alpha seam");
        assert_eq!(s[0].items[0].summary, "does the alpha thing.");
        assert_eq!(s[3].items.len(), 2); // two bullets under v1.0
    }

    #[test]
    fn ac1_local_1_2_with_candidates_on_shows_only_v1_3() {
        let a = parse_anchor(DOC).unwrap();
        let s = parse_sections(DOC);
        let (status, target, entries) = compute_delta(Some("1.2"), &a, &s, true);
        assert_eq!(status, UpdateStatus::Behind);
        assert_eq!(target, "1.3-candidate");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].version, "1.3-candidate");
    }

    #[test]
    fn ac2_local_at_stable_default_is_current_and_empty() {
        let a = parse_anchor(DOC).unwrap();
        let s = parse_sections(DOC);
        let (status, _t, entries) = compute_delta(Some("1.2"), &a, &s, false);
        assert_eq!(status, UpdateStatus::Current);
        assert!(entries.is_empty());
    }

    #[test]
    fn default_channel_hides_candidates() {
        let a = parse_anchor(DOC).unwrap();
        let s = parse_sections(DOC);
        // local 1.1, candidates off → shows 1.2 only, never 1.3-candidate.
        let (status, target, entries) = compute_delta(Some("1.1"), &a, &s, false);
        assert_eq!(status, UpdateStatus::Behind);
        assert_eq!(target, "1.2");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].version, "1.2");
        assert!(entries.iter().all(|e| !e.is_candidate));
    }

    #[test]
    fn behind_lists_newest_first() {
        let a = parse_anchor(DOC).unwrap();
        let s = parse_sections(DOC);
        let (_st, _t, entries) = compute_delta(Some("0.9"), &a, &s, false);
        // 1.0, 1.1, 1.2 (candidate hidden), newest first.
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].version, "1.2");
        assert_eq!(entries[2].version, "1.0");
    }

    #[test]
    fn ac6_missing_local_version_is_unknown() {
        let a = parse_anchor(DOC).unwrap();
        let s = parse_sections(DOC);
        let (status, _t, entries) = compute_delta(None, &a, &s, false);
        assert_eq!(status, UpdateStatus::Unknown);
        assert!(!entries.is_empty()); // surfaces what's available for a full check
    }
}

#[cfg(test)]
mod bridge_tests {
    use super::*;

    #[test]
    fn resolver_finds_direct_schema() {
        let dir = std::env::temp_dir().join("loci_resolver_direct_test");
        let _ = fs::create_dir_all(dir.join(".schema"));
        fs::write(dir.join(".schema/manifest.json"), "{}").ok();
        let resolved = resolve_manifest_path(&dir).expect("direct resolve");
        assert!(resolved.ends_with(".schema/manifest.json"));
        assert!(resolved.starts_with(&dir));
    }

    #[test]
    fn resolver_returns_none_when_nothing_found() {
        let resolved = resolve_manifest_path(Path::new("/tmp/definitely_not_a_palace_xyz"));
        assert!(resolved.is_none());
    }

    #[test]
    fn extract_task_title_prefers_bold() {
        assert_eq!(
            extract_task_title("**2026-05-26, Write PALACES.md** body text here"),
            "2026-05-26, Write PALACES.md"
        );
        let long = "x".repeat(200);
        assert_eq!(extract_task_title(&long).chars().count(), 80);
    }

    #[test]
    fn read_cron_detail_rejects_traversal() {
        for bad in ["../secrets", "a/b", "..", ""] {
            assert!(
                read_cron_detail("/tmp/loci-test-root".into(), bad.into()).is_err(),
                "expected reject for {bad:?}"
            );
        }
    }

    #[test]
    fn read_tasks_against_live_palace() {
        let palace = PathBuf::from("/tmp/loci-test-root");
        if !palace.join("_palace/TASKS.md").is_file() {
            eprintln!("skipping: live TASKS.md not present");
            return;
        }
        let items = read_tasks(palace.to_string_lossy().to_string()).expect("read_tasks");
        assert!(!items.is_empty(), "expected questlog items");
        assert!(items.iter().any(|i| !i.done), "expected at least one open item");
        // Every item carries a track (default "Unfiled" before any ## heading).
        assert!(items.iter().all(|i| !i.track.is_empty()), "track must be populated");
    }

    #[test]
    fn read_tasks_groups_by_h2_heading() {
        let dir = std::env::temp_dir().join(format!("loci_tasks_test_{}", std::process::id()));
        let _ = fs::create_dir_all(dir.join("_palace"));
        let md = "# Palace TASKS\n- [ ] orphan before any heading\n## Loci\n- [ ] **A** build the thing\n- [x] **B** shipped\n## Webapp\n- [ ] **C** perf triage\n";
        fs::write(dir.join("_palace/TASKS.md"), md).unwrap();
        let items = read_tasks(dir.to_string_lossy().to_string()).expect("read_tasks");
        let track_of = |t: &str| items.iter().find(|i| i.body.contains(t)).map(|i| i.track.clone());
        assert_eq!(track_of("orphan").as_deref(), Some("Unfiled"));
        assert_eq!(track_of("build the thing").as_deref(), Some("Loci"));
        assert_eq!(track_of("shipped").as_deref(), Some("Loci"));
        assert_eq!(track_of("perf triage").as_deref(), Some("Webapp"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_cron_detail_against_live_job() {
        let palace = PathBuf::from("/tmp/loci-test-root");
        if !palace.join("_palace/cron").is_dir() {
            eprintln!("skipping: live cron dir not present");
            return;
        }
        let states = read_cron_states(palace.to_string_lossy().to_string()).expect("states");
        let Some(first) = states.first() else {
            eprintln!("skipping: no cron jobs");
            return;
        };
        // Use the dir key, not the display label (which may contain '/').
        let detail = read_cron_detail(palace.to_string_lossy().to_string(), first.key.clone())
            .expect("detail for a real job");
        assert!(detail.is_object() || detail.is_array());
    }

    #[test]
    fn read_cron_states_against_live_palace() {
        // The convention is palace_path/_palace/cron/.
        let palace = PathBuf::from("/tmp/loci-test-root");
        if !palace.join("_palace/cron").is_dir() {
            eprintln!("skipping: live palace cron dir absent");
            return;
        }
        let snapshots = read_cron_states(palace.to_string_lossy().to_string())
            .expect("read_cron_states must succeed");
        assert!(
            snapshots.len() > 3,
            "expected several cron jobs, got {}",
            snapshots.len()
        );
        // At least one job must publish a CIQ score (coworking-eval-daily).
        assert!(
            snapshots.iter().any(|s| s.ciq.is_some()),
            "expected at least one CIQ-publishing job"
        );
        // Most snapshots should expose at least one of {status, summary, last_run}.
        // A handful of palace-cron jobs publish metric-only state.json files that
        // carry none of these keys; we don't fail the suite over those.
        let visible = snapshots
            .iter()
            .filter(|s| s.status.is_some() || s.summary.is_some() || s.last_run.is_some())
            .count();
        let ratio = visible as f64 / snapshots.len() as f64;
        assert!(
            ratio >= 0.75,
            "expected ≥75% of cron snapshots to expose visible state; got {}/{} ({:.0}%)",
            visible,
            snapshots.len(),
            ratio * 100.0
        );
    }

    #[test]
    fn read_cron_states_empty_path() {
        // Non-existent palace should return Ok(empty), not Err.
        let snapshots = read_cron_states("/tmp/definitely_not_a_palace_xyz".into())
            .expect("read_cron_states should swallow missing cron dir");
        assert!(snapshots.is_empty());
    }

    #[test]
    fn read_handovers_against_live_palace() {
        let palace = PathBuf::from("/tmp/loci-test-root");
        if !palace.join("_palace/handovers").is_dir() {
            eprintln!("skipping: live handovers absent");
            return;
        }
        let handovers = read_handovers(palace.to_string_lossy().to_string(), Some(5))
            .expect("read_handovers must succeed");
        assert!(
            !handovers.is_empty(),
            "expected handovers under _palace/handovers"
        );
        assert!(handovers.len() <= 5, "limit must be honored");
        // Newest first.
        for pair in handovers.windows(2) {
            assert!(
                pair[0].mtime >= pair[1].mtime,
                "handovers must be sorted newest-first"
            );
        }
    }

}

// ─── Act 1: Naming Ceremony ───────────────────────────────────────────────────
//
// Commands: scaffold_palace_from_ceremony, check_ceremony_vagueness,
//           generate_greeter_names
//
// The ceremony inverts the scaffold→greet flow: the Greeter conversation runs
// first, each answer accumulates in the Leptos component, and scaffold runs
// exactly once at the end with all answers in hand. This makes the write
// transactional — no partial palace from a mid-ceremony abandon.

/// Generate a URL-safe slug from free text.
/// Takes the first 6 significant words, strips stopwords, lowercases, hyphenates.
fn make_slug(text: &str) -> String {
    const STOP: &[&str] = &[
        "a", "an", "the", "and", "or", "but", "in", "of", "to", "is", "are", "i",
    ];
    let words: Vec<String> = text
        .split_whitespace()
        .map(|w| {
            w.to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
        })
        .filter(|w| !w.is_empty() && !STOP.contains(&w.as_str()))
        .take(6)
        .collect();
    let slug = words.join("-");
    slug.chars().take(48).collect()
}

fn first_words(text: &str, n: usize) -> String {
    text.split_whitespace()
        .take(n)
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_vague_heuristic(answer: &str) -> bool {
    if answer.split_whitespace().count() < 6 {
        return true;
    }
    let lower = answer.to_lowercase();
    ["work", "stuff", "busy", "things", "life", "lots", "not much", "nothing"]
        .iter()
        .any(|w| lower.contains(w))
}

#[derive(Debug, Serialize, Deserialize)]
struct NameOption {
    name: String,
    note: String,
}

fn fallback_names() -> Vec<NameOption> {
    const FB: &[(&str, &str)] = &[
        ("Lumen", "carries light into the unlit corner"),
        ("Cipher", "finds structure others miss"),
        ("Wren", "quick, precise, slightly irreverent"),
    ];
    FB.iter()
        .map(|(n, d)| NameOption { name: n.to_string(), note: d.to_string() })
        .collect()
}

fn parse_name_options(text: &str) -> Vec<NameOption> {
    text.lines()
        .filter_map(|line| {
            let clean = line
                .trim()
                .trim_start_matches(|c: char| {
                    c.is_ascii_digit() || c == '.' || c == ')' || c == '-' || c == '*'
                })
                .trim();
            // Accept both "·" (U+00B7) and "—" as separators from different models
            let sep_pos = clean.find('·').or_else(|| clean.find(" - "));
            if let Some(pos) = sep_pos {
                let name = clean[..pos].trim().to_string();
                let note = clean[pos + 1..].trim_start_matches('·').trim_start_matches('-').trim().to_string();
                if !name.is_empty() && !note.is_empty() && name.split_whitespace().count() <= 3 {
                    return Some(NameOption { name, note });
                }
            }
            None
        })
        .take(4)
        .collect()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CeremonyArgs {
    parent_path: String,
    holder_name: String,
    present_answer: String,
    present_answer_refined: Option<String>,
    garden_seed: String,
    greeter_name: String,
    onboarding_complete: bool,
}

/// Create a full palace from ceremony answers.
/// Writes PALACE.md, garden crystals, observatory + archive rooms.
/// Returns parent_path on success so the caller can load_palace immediately.
///
/// Takes individual parameters (Tauri v2 maps camelCase IPC keys → snake_case).
/// The frontend sends CeremonyAnswers { parentPath, holderName, ... } and each
/// field lands directly as a Rust parameter — no outer `args` wrapper needed.
#[tauri::command]
fn scaffold_palace_from_ceremony(
    parent_path: String,
    holder_name: String,
    present_answer: String,
    present_answer_refined: Option<String>,
    garden_seed: String,
    greeter_name: String,
    onboarding_complete: bool,
) -> Result<String, String> {
    // Re-package into the internal struct so the function body is unchanged.
    let args = CeremonyArgs {
        parent_path,
        holder_name,
        present_answer,
        present_answer_refined,
        garden_seed,
        greeter_name,
        onboarding_complete,
    };
    let parent = Path::new(&args.parent_path);
    let palace = parent.join("_palace");
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();

    // Create directory structure
    for dir in &[
        palace.join("garden").join("plants"),
        palace.join("observatory").join("crystals"),
        palace.join("archive").join("crystals"),
        palace.join("cron"), // needed by the watcher; absence causes "cron directory missing" error
    ] {
        fs::create_dir_all(dir).map_err(|e| format!("mkdir {}: {e}", dir.display()))?;
    }

    // PALACE.md — AI-agnostic root marker (CLAUDE.md is legacy fallback)
    let greeter_name = if args.greeter_name.trim().is_empty() {
        "unnamed".to_string()
    } else {
        args.greeter_name.trim().to_string()
    };
    let holder_name = if args.holder_name.trim().is_empty() {
        "unnamed".to_string()
    } else {
        args.holder_name.trim().to_string()
    };

    let onboarded_line = if args.onboarding_complete {
        format!("> Onboarded: {date}\n")
    } else {
        "> Onboarded: false\n".to_string()
    };
    let palace_md = format!(
        "# {holder_name}'s Palace\n> Palace holder: {holder_name}\n> Named: {date}\n> Companion: {greeter_name}\n{onboarded_line}"
    );
    fs::write(parent.join("PALACE.md"), &palace_md)
        .map_err(|e| format!("write PALACE.md: {e}"))?;

    // User's first crystal (Garden)
    let (crystal_title, crystal_body) = {
        if let Some(ref refined) = args.present_answer_refined {
            let title = first_words(refined, 5);
            let body = format!(
                "---\ntype: crystal\ntier: \"◈\"\ncreated: {date}\nauthor: {holder_name}\ngreeter-header: true\n---\n\n# {title}\n\nThe question was: what are you in the middle of right now?\n\n{refined}\n\n*When the palace opened: {}*\n",
                args.present_answer
            );
            (title, body)
        } else if !args.present_answer.trim().is_empty() {
            let title = first_words(&args.present_answer, 5);
            let body = format!(
                "---\ntype: crystal\ntier: \"◈\"\ncreated: {date}\nauthor: {holder_name}\ngreeter-header: true\n---\n\n# {title}\n\nThe question was: what are you in the middle of right now?\n\n{}\n",
                args.present_answer
            );
            (title, body)
        } else {
            // Skip: empty crystal
            (String::new(), format!("---\ntype: crystal\ntier: \"◈\"\ncreated: {date}\nauthor: {holder_name}\n---\n\n*(unanswered at the door)*\n"))
        }
    };
    let crystal_slug = {
        let s = make_slug(&crystal_title);
        if s.is_empty() { "opening-moment".to_string() } else { s }
    };
    fs::write(palace.join("garden").join(format!("{crystal_slug}.md")), &crystal_body)
        .map_err(|e| format!("write first crystal: {e}"))?;

    // Greeter's own crystal
    let present_clip = first_words(&args.present_answer, 10);
    let seed_clip = first_words(&args.garden_seed, 10);
    let greeter_crystal = format!(
        "---\ntype: crystal\ntier: \"◈\"\ncreated: {date}\nauthor: {greeter_name}\nrole: greeter\nvalid_until: growing\n---\n\n# {greeter_name}\n\nI was named {greeter_name} in this palace on {date}.\n\nI am what this palace makes me.\nRight now I know:\n- {holder_name} is in the middle of: {present_clip}\n- {holder_name} is curious about: {seed_clip}\n\nThis crystal grows as the palace grows.\nCome back and read it later.\n\n---\n\n*This is the first thing I know about myself in this palace.*\n*The naming ceremony was Act 1. What comes next is not an act — it is just the place.*\n"
    );
    let greeter_slug = {
        let s = make_slug(&greeter_name);
        if s.is_empty() { "greeter".to_string() } else { s }
    };
    fs::write(palace.join("garden").join(format!("{greeter_slug}-origin.md")), &greeter_crystal)
        .map_err(|e| format!("write greeter crystal: {e}"))?;

    // Garden seed plant (skip if no answer)
    if !args.garden_seed.trim().is_empty() {
        let plant_name = first_words(&args.garden_seed, 6);
        let plant = format!(
            "---\ntype: plant\nstatus: seeded\nseeded: {date}\n---\n\n# {plant_name}\n\n*Seeded during the naming ceremony.*\n\n{}\n\n---\n\n## Waterings\n\n*(none yet — first session opens this)*\n",
            args.garden_seed
        );
        let plant_slug = {
            let s = make_slug(&args.garden_seed);
            if s.is_empty() { "first-plant".to_string() } else { s }
        };
        fs::write(palace.join("garden").join("plants").join(format!("{plant_slug}.md")), &plant)
            .map_err(|e| format!("write garden plant: {e}"))?;
    }

    // Observatory
    fs::write(
        palace.join("observatory").join("CLAUDE.md"),
        "# Observatory\n> Where patterns appear when you step back.\n",
    )
    .map_err(|e| format!("write observatory CLAUDE.md: {e}"))?;

    let obs_crystal = format!(
        "---\ntype: observatory-watch\nstatus: unnamed\nseeded: {date}\n---\n\n# The pattern without a name\n\nSomething recurs. In work, in conversations, in what you notice first when you enter a room. You have seen it more than twice. You have not named it.\n\nThis crystal is the name when it comes.\n\n---\n\n# What is one thing you keep re-deriving from scratch?\n\nYou didn't answer this at the door. Good. That question needs time.\n\nWhen you're ready: name it here.\n"
    );
    fs::write(
        palace.join("observatory").join("crystals").join("the-pattern-without-a-name.md"),
        &obs_crystal,
    )
    .map_err(|e| format!("write observatory crystal: {e}"))?;

    // Archive
    fs::write(
        palace.join("archive").join("CLAUDE.md"),
        "# Archive\n> Where what persists, persists.\n",
    )
    .map_err(|e| format!("write archive CLAUDE.md: {e}"))?;

    let thing_true = format!(
        "---\ntype: crystal\ntier: \"◇\"\nseeded: {date}\n---\n\n# The thing that has always been true\n\nWhat is something you have known so long you have stopped believing it needs saying?\n\nIt probably does. Write it here.\n"
    );
    fs::write(
        palace.join("archive").join("crystals").join("the-thing-that-has-always-been-true.md"),
        &thing_true,
    )
    .map_err(|e| format!("write archive crystal 1: {e}"))?;

    let thread = format!(
        "---\ntype: crystal\ntier: \"◇\"\nseeded: {date}\n---\n\n# The thread between\n\nTwo things you care about are connected. You have not found the connection yet.\n\nWhen you do, write it here. The title of this crystal is wrong until then — rename it.\n"
    );
    fs::write(
        palace.join("archive").join("crystals").join("the-thread-between.md"),
        &thread,
    )
    .map_err(|e| format!("write archive crystal 2: {e}"))?;

    // Persist config
    let mut config = read_loci_config();
    config.palace_path = Some(args.parent_path.clone());
    write_loci_config(config)?;

    Ok(args.parent_path)
}

/// Check whether a ceremony answer is concrete enough to proceed, or vague enough
/// to trigger the Moment2b follow-up question.
/// Tries local Ollama first with a 3s total timeout; falls back to heuristic.
#[tauri::command]
async fn check_ceremony_vagueness(
    state: tauri::State<'_, OllamaState>,
    answer: String,
) -> Result<bool, String> {
    let full_cfg = read_loci_config();
    let cfg = full_cfg.ollama.unwrap_or_default();
    if cfg.offline_mode {
        return Ok(is_vague_heuristic(&answer));
    }
    let base = match validate_ollama_url(&cfg.base_url) {
        Ok(b) => b,
        Err(_) => return Ok(is_vague_heuristic(&answer)),
    };
    // Short-timeout client: enforce the ceiling at the HTTP layer so the future
    // cancels reliably even when Ollama is running but slow. tokio::time::timeout
    // alone is unreliable because reqwest may not drop an in-flight request fast
    // enough on future cancellation — the reqwest-level timeout fires first.
    let ceremony_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .connect_timeout(std::time::Duration::from_millis(500))
        .build()
        .unwrap_or_else(|_| state.client.clone());
    let backend = OllamaBackend { client: ceremony_client, base };
    let system = "Answer only with one word: concrete or vague.";
    let prompt = format!(
        "Does this answer name something specific — a project, task, goal, or named thing? Answer: '{answer}'"
    );

    // Outer 4s belt-and-suspenders guard; the HTTP client timeout above fires at 3s.
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(4),
        async {
            if !backend.health().await { return None; }
            let model = backend.resolve_model(&cfg.chat_model).await.ok()?;
            backend.chat(system, &prompt, &model).await.ok()
        },
    )
    .await;

    match result {
        Ok(Some(resp)) => {
            let lower = resp.to_lowercase();
            if lower.contains("concrete") { Ok(false) }
            else if lower.contains("vague") { Ok(true) }
            else { Ok(is_vague_heuristic(&answer)) }
        }
        _ => Ok(is_vague_heuristic(&answer)),
    }
}

/// Generate 4 Greeter name options from the ceremony answers.
/// Tries local Ollama; falls back to a curated static list on offline.
#[tauri::command]
async fn generate_greeter_names(
    state: tauri::State<'_, OllamaState>,
    holder_name: String,
    present_answer: String,
    garden_seed: String,
) -> Result<Vec<NameOption>, String> {
    let full_cfg = read_loci_config();
    let cfg = full_cfg.ollama.unwrap_or_default();
    if cfg.offline_mode {
        return Ok(fallback_names());
    }
    let base = match validate_ollama_url(&cfg.base_url) {
        Ok(b) => b,
        Err(_) => return Ok(fallback_names()),
    };
    // Short-timeout client: same rationale as check_ceremony_vagueness.
    // Naming names should feel fast or fall back gracefully — 30s waits here
    // kill the ceremony's momentum. 2s HTTP timeout + 3s outer guard.
    let name_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .connect_timeout(std::time::Duration::from_millis(500))
        .build()
        .unwrap_or_else(|_| state.client.clone());
    let backend = OllamaBackend { client: name_client, base };
    let system = "You are naming an AI companion for a memory palace. \
        Output exactly 4 options, one per line. \
        Format: Name · one-line character note (under 10 words). \
        Names should be 1-2 words, drawn from what the user told you.";
    let prompt = format!(
        "User: {holder_name}. In the middle of: {present_answer}. Curious about: {garden_seed}. Offer 4 name options."
    );
    // Outer 3s guard; the HTTP client timeout above fires at 2s.
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        async {
            if !backend.health().await { return None; }
            let model = backend.resolve_model(&cfg.chat_model).await.ok()?;
            backend.chat(system, &prompt, &model).await.ok()
        },
    ).await;
    match result {
        Ok(Some(resp)) => {
            let parsed = parse_name_options(&resp);
            if parsed.len() >= 2 { Ok(parsed) } else { Ok(fallback_names()) }
        }
        _ => Ok(fallback_names()),
    }
}

/// Fast inference readiness probe (≤3 s per backend).
/// Called by the frontend model gate before ceremony and by the Settings page.
#[tauri::command]
async fn check_inference_available(
    state: tauri::State<'_, OllamaState>,
) -> Result<InferenceStatus, String> {
    let full_cfg = read_loci_config();
    let cfg = full_cfg.ollama.clone().unwrap_or_default();

    // ── Local (Ollama) ────────────────────────────────────────────────────────
    let (has_local, ollama_running, local_models) = if cfg.offline_mode {
        (false, false, vec![])
    } else {
        match validate_ollama_url(&cfg.base_url) {
            Err(_) => (false, false, vec![]),
            Ok(base) => {
                let client = state.client.clone();
                let result = tokio::time::timeout(
                    std::time::Duration::from_secs(3),
                    async move {
                        let url = match base.join("/api/tags") {
                            Ok(u) => u,
                            Err(_) => return (false, false, vec![]),
                        };
                        let resp = match client.get(url).send().await {
                            Ok(r) => r,
                            Err(_) => return (false, false, vec![]),
                        };
                        if !resp.status().is_success() {
                            return (false, true, vec![]);
                        }
                        let body: OllamaTagsResponse = match resp.json().await {
                            Ok(b) => b,
                            Err(_) => return (false, true, vec![]),
                        };
                        let models: Vec<String> =
                            body.models.into_iter().map(|m| m.name).collect();
                        (!models.is_empty(), true, models)
                    },
                )
                .await;
                result.unwrap_or((false, false, vec![]))
            }
        }
    };

    // ── External (Claude CLI) ─────────────────────────────────────────────────
    let has_claude = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        async {
            match resolve_claude() {
                None => false,
                Some((bin, path_env)) => ClaudeBackend { bin, path_env }.health().await,
            }
        },
    )
    .await
    .unwrap_or(false);

    Ok(InferenceStatus {
        has_local,
        ollama_running,
        has_claude,
        local_models,
    })
}

// ─── rc.3 cockpit: palace maps rail + tools gate-ledger ──────────────────────
//
// The cockpit shape (one Operations view + a tab per self-contained palace-map
// instrument) re-expressed for any palace. A "palace map" is a standalone
// *.html the palace generates for itself, self-contained by convention, with
// its live data embedded as <script id="payload"|"snapshot" type="application/json">.
// The app discovers them, never hardcodes them, and embeds via iframe srcdoc
// (no filesystem custom-protocol exposure; the read goes through a validated
// command). Trust boundary: a palace map is the user's own palace content,
// same standing as every other file this app reads.

/// Directories a palace may keep instruments in, tried in order. Non-recursive:
/// instruments live at a palace's surface, not buried in room internals.
fn cockpit_scan_dirs(root: &Path) -> Vec<PathBuf> {
    vec![
        root.to_path_buf(),
        root.join("_palace"),
        root.join("cockpit"),
        root.join("_palace").join("cockpit"),
    ]
}

#[derive(Debug, Serialize)]
struct PalaceMapEntry {
    /// Stable key = file stem. Also the tab identity.
    key: String,
    /// Path relative to the palace root, safe to hand back to read_palace_map_html.
    file: String,
    /// Display label derived from the stem ("crystal-map" → "crystal map").
    label: String,
    /// Best-effort count peeked from the embedded payload ("41 entries").
    /// None on any parse trouble; a badge is cosmetic, never an error.
    badge: Option<String>,
}

/// Peek an instrument's embedded JSON payload and derive a generic badge:
/// the length of the largest array found at the top level or one level deep.
fn peek_map_badge(html: &str) -> Option<String> {
    let marker_at = ["<script id=\"payload\"", "<script id=\"snapshot\""]
        .iter()
        .find_map(|m| html.find(m))?;
    let body_start = html[marker_at..].find('>')? + marker_at + 1;
    let body_end = html[body_start..].find("</script>")? + body_start;
    let value: serde_json::Value = serde_json::from_str(html[body_start..body_end].trim()).ok()?;
    let largest = match &value {
        serde_json::Value::Array(a) => a.len(),
        serde_json::Value::Object(o) => o
            .values()
            .filter_map(|v| v.as_array().map(|a| a.len()))
            .max()
            .unwrap_or(0),
        _ => 0,
    };
    (largest > 0).then(|| format!("{largest} entries"))
}

/// Discover self-contained palace-map instruments. dashboard.html is excluded:
/// that is a palace's generated cockpit page itself; embedding it here would
/// nest a cockpit inside the cockpit.
#[tauri::command]
fn list_palace_maps(palace_path: String) -> Result<Vec<PalaceMapEntry>, String> {
    let root = PathBuf::from(&palace_path);
    if !root.is_dir() {
        return Err("palace path is not a directory".to_string());
    }
    let mut maps = Vec::new();
    for dir in cockpit_scan_dirs(&root) {
        let Ok(entries) = fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            let is_html = path.extension().and_then(|e| e.to_str()) == Some("html");
            if !path.is_file() || !is_html {
                continue;
            }
            let stem = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            if stem.is_empty() || stem == "dashboard" {
                continue;
            }
            let Ok(html) = fs::read_to_string(&path) else { continue };
            let has_payload = html.contains("<script id=\"payload\"")
                || html.contains("<script id=\"snapshot\"");
            if !has_payload {
                continue;
            }
            let Ok(rel) = path.strip_prefix(&root) else { continue };
            // First hit wins per stem: root-level instruments shadow kit-dir copies.
            if maps.iter().any(|m: &PalaceMapEntry| m.key == stem) {
                continue;
            }
            maps.push(PalaceMapEntry {
                key: stem.clone(),
                file: rel.to_string_lossy().to_string(),
                label: stem.replace(['-', '_'], " "),
                badge: peek_map_badge(&html),
            });
        }
    }
    maps.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(maps)
}

/// Return a discovered map's HTML for srcdoc embedding. The `file` value must
/// be a relative .html path that resolves inside the palace root; traversal
/// and absolute paths are rejected before any read happens.
#[tauri::command]
fn read_palace_map_html(palace_path: String, file: String) -> Result<String, String> {
    let root = PathBuf::from(&palace_path);
    let rel = Path::new(&file);
    if rel.is_absolute()
        || rel
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
    {
        return Err("map path must be a plain relative path".to_string());
    }
    if rel.extension().and_then(|e| e.to_str()) != Some("html") {
        return Err("map path must end in .html".to_string());
    }
    let target = root.join(rel);
    // Belt over the component check: the canonical path must stay under the
    // canonical root (catches symlinked escapes the component filter cannot).
    let canon_root = root
        .canonicalize()
        .map_err(|e| format!("palace root unreadable: {e}"))?;
    let canon_target = target
        .canonicalize()
        .map_err(|e| format!("map file unreadable: {e}"))?;
    if !canon_target.starts_with(&canon_root) {
        return Err("map path escapes the palace root".to_string());
    }
    fs::read_to_string(&canon_target).map_err(|e| format!("read map: {e}"))
}

#[derive(Debug, Serialize, Deserialize)]
struct ToolLedgerEntry {
    id: String,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    room: Option<String>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    license: Option<String>,
    #[serde(default)]
    quarantine_state: Option<String>,
    #[serde(default)]
    gate_read: Option<String>,
}

/// Read the tools gate-ledger: `tools.items` from the palace's map JSON.
/// Tries palace-map.json then map.json in each cockpit scan dir. Fail-soft:
/// a palace without a ledger (or with a malformed one) gets an empty shelf,
/// never an error: the dashboard must render regardless.
#[tauri::command]
fn read_tools_ledger(palace_path: String) -> Vec<ToolLedgerEntry> {
    let root = PathBuf::from(&palace_path);
    for dir in cockpit_scan_dirs(&root) {
        for name in ["palace-map.json", "map.json"] {
            let candidate = dir.join(name);
            let Ok(bytes) = fs::read(&candidate) else { continue };
            let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
                continue;
            };
            let Some(items) = value.get("tools").and_then(|t| t.get("items")) else {
                continue;
            };
            if let Ok(entries) = serde_json::from_value::<Vec<ToolLedgerEntry>>(items.clone()) {
                return entries;
            }
        }
    }
    Vec::new()
}

#[cfg(test)]
mod cockpit_tests {
    use super::*;

    fn temp_palace(tag: &str) -> PathBuf {
        // Unique per test: tests share the process and run on parallel threads,
        // so a process-id-only path would let one test's cleanup race another's writes.
        let dir = std::env::temp_dir().join(format!("loci-cockpit-test-{}-{tag}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn map_read_rejects_traversal_and_absolute() {
        let root = temp_palace("traversal");
        let root_s = root.to_string_lossy().to_string();
        assert!(read_palace_map_html(root_s.clone(), "../evil.html".into()).is_err());
        assert!(read_palace_map_html(root_s.clone(), "/etc/hosts".into()).is_err());
        assert!(read_palace_map_html(root_s.clone(), "notes.md".into()).is_err());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn discovers_payload_instruments_and_skips_dashboard() {
        let root = temp_palace("discover");
        fs::write(
            root.join("crystal-map.html"),
            r#"<html><script id="payload" type="application/json">{"crystals":[1,2,3]}</script></html>"#,
        )
        .unwrap();
        fs::write(root.join("dashboard.html"), "<html><script id=\"payload\"></script></html>").unwrap();
        fs::write(root.join("plain.html"), "<html>no payload</html>").unwrap();
        let maps = list_palace_maps(root.to_string_lossy().to_string()).unwrap();
        assert_eq!(maps.len(), 1);
        assert_eq!(maps[0].key, "crystal-map");
        assert_eq!(maps[0].badge.as_deref(), Some("3 entries"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn dashboard_reads_resolve_rooms_at_root_layout() {
        let root = temp_palace("roomsatroot");
        let root_s = root.to_string_lossy().to_string();
        fs::create_dir_all(root.join("cron").join("demo-job")).unwrap();
        fs::write(
            root.join("cron").join("demo-job").join("state.json"),
            r#"{"job":"demo","status":"ok"}"#,
        )
        .unwrap();
        fs::create_dir_all(root.join("great-hall")).unwrap();
        fs::write(
            root.join("great-hall").join("QUEST-LOG.md"),
            "## Track\n- [ ] **2026-07-02, Demo item.** body\n",
        )
        .unwrap();
        fs::write(root.join("great-hall").join("_HANDOVER_demo.md"), "## State\nok\n").unwrap();
        fs::write(root.join("great-hall").join("MORNING-BRIEF.md"), "not a handover\n").unwrap();

        assert_eq!(read_cron_states(root_s.clone()).unwrap().len(), 1);
        assert_eq!(read_tasks(root_s.clone()).unwrap().len(), 1);
        let handovers = read_handovers(root_s.clone(), None).unwrap();
        assert_eq!(handovers.len(), 1, "great-hall must be prefix-gated to _HANDOVER*");
        assert!(handovers[0].filename.starts_with("_HANDOVER"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn ledger_is_fail_soft() {
        let root = temp_palace("ledger");
        // No file at all → empty shelf.
        assert!(read_tools_ledger(root.to_string_lossy().to_string()).is_empty());
        // Ledger present → parsed.
        fs::write(
            root.join("palace-map.json"),
            r#"{"tools":{"items":[{"id":"x","quarantine_state":"rejected"}]}}"#,
        )
        .unwrap();
        let tools = read_tools_ledger(root.to_string_lossy().to_string());
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].quarantine_state.as_deref(), Some("rejected"));
        let _ = fs::remove_dir_all(&root);
    }
}

fn main() {
    let ollama_client = OllamaState {
        client: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120)) // generous for local inference
            .build()
            .expect("failed to build reqwest client"),
    };

    // 1B — MCP server handle (no server running at startup)
    let mcp_handle = Mutex::new(McpServerHandle {
        shutdown_tx: None,
        port: None,
    });

    // v0.6.0: file watcher state (no watcher active at startup)
    let watcher_state = WatcherState {
        watcher: Mutex::new(None),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(ollama_client)
        .manage(mcp_handle)
        .manage(watcher_state)
        .invoke_handler(tauri::generate_handler![
            detect_palace,
            migrate_to_loci,
            // Palace bridge
            validate_palace_path,
            pick_palace_dir,
            scaffold_palace,
            load_palace,
            read_palace_state,
            // 1A — Ollama local inference
            check_ollama_health,
            list_ollama_models,
            call_ollama,
            embed_text,
            // 1B — MCP server
            start_mcp_server,
            stop_mcp_server,
            mcp_server_status,
            // Config persistence
            read_loci_config,
            write_loci_config,
            // v0.6.0: palace bridge
            read_manifest,
            read_manifest_summary,
            start_state_watcher,
            read_cron_states,
            read_handovers,
            // v0.6.0 Phase 3.6: drill-down detail + questlog
            read_cron_detail,
            read_tasks,
            // Phase 4a: chat as QUERY (inference trait, fail-closed)
            chat_query,
            // Act 1: Naming Ceremony
            scaffold_palace_from_ceremony,
            check_ceremony_vagueness,
            generate_greeter_names,
            // Settings: inference readiness gate
            check_inference_available,
            // palace-update: the delta checker (read-only, explicit, fail-closed)
            check_for_updates,
            // rc.3 cockpit: maps rail + tools gate-ledger
            list_palace_maps,
            read_palace_map_html,
            read_tools_ledger,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
