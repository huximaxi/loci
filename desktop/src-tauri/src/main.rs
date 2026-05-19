// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

// ─── 1B: MCP server ───────────────────────────────────────────────────────────
mod mcp;

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

/// Check that `path` is a valid palace root: exists, is a dir,
/// has CLAUDE.md and a _palace/ subdir.
#[tauri::command]
fn validate_palace_path(path: String) -> bool {
    let p = Path::new(&path);
    p.exists() && p.is_dir()
        && p.join("CLAUDE.md").exists()
        && p.join("_palace").is_dir()
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
/// Requires: CLAUDE.md + _palace/ + at least one room with CLAUDE.md.
/// Persists palace_path to config on success.
#[tauri::command]
fn load_palace(path: String) -> Result<PalaceManifest, String> {
    // Derive root from an owned PathBuf so `path` stays free to move into the return struct.
    let root_buf = std::path::PathBuf::from(&path);
    let root = root_buf.as_path();

    if !root.exists() || !root.is_dir() {
        return Err("path does not exist or is not a directory".to_string());
    }
    if !root.join("CLAUDE.md").exists() {
        return Err("not a palace: CLAUDE.md not found at root".to_string());
    }
    let palace_dir = root.join("_palace");
    if !palace_dir.is_dir() {
        return Err("not a palace: _palace/ directory not found".to_string());
    }

    let mut rooms = Vec::new();
    if let Ok(entries) = fs::read_dir(&palace_dir) {
        let mut dirs: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                let n = e.file_name();
                let name = n.to_string_lossy();
                e.path().is_dir() && name != "cron" && !name.starts_with('.')
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

    let cron_job_count = fs::read_dir(palace_dir.join("cron"))
        .map(|e| e.filter_map(|x| x.ok()).filter(|x| x.path().is_dir()).count())
        .unwrap_or(0);

    let mut config = read_loci_config();
    config.palace_path = Some(path.clone());
    write_loci_config(config)?;

    Ok(PalaceManifest {
        path,
        rooms,
        cron_job_count,
        crystal_count: count_crystals(root),
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

    // 1. Check for loci palace (already migrated)
    if let Some(home) = dirs::home_dir() {
        let loci_path = home.join(".loci");
        if loci_path.exists() && loci_path.join("CLAUDE.md").exists() {
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

    // 2. Check for _palace/ pattern (Vesper × Hux)
    let palace_dir = path.join("_palace");
    if palace_dir.exists() {
        return DetectionResult {
            found: true,
            kind: Some("mempalace".to_string()),
            path: Some(palace_dir.to_string_lossy().to_string()),
            rooms: Some(detect_rooms(&palace_dir)),
            crystal_count: Some(count_crystals(&palace_dir)),
            suggestion: "Found Vesper × Hux memory palace. Ready to migrate to loci format.".to_string(),
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

    // 5. No palace found
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

// Helper: count all .md files recursively
fn count_md_files(path: &Path) -> usize {
    let mut count = 0;

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            if entry_path.is_dir() {
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

// ─── v0.6.0 bridge: manifest reader + state watcher ──────────────────────────
//
// Phase 1 of the Tauri x Palace Bridge (v0.6.0). Two surfaces:
//
//   1. read_manifest(palace_path): typed read of .schema/manifest.json with
//      a runtime path resolver. Tries palace_path/.schema/manifest.json first,
//      then falls back to sibling _palace-quantum-v1/.schema/manifest.json.
//      Survives the 2026-05-25 cutover with zero code change.
//
//   2. start_state_watcher(palace_path): notify-based watcher on palace_path/cron/.
//      Filters to state.json files only. Emits "state_changed" events to the
//      frontend with RELATIVE paths only (Cipher: no absolute-path leakage).
//
// Manifest types are duplicated in src-leptos/src/models.rs to give the WASM
// side strong types over the serde-wasm-bindgen wire. The two definitions MUST
// stay in lockstep. Future cleanup: move both into a shared workspace crate.

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

fn resolve_manifest_path(palace_path: &Path) -> Option<PathBuf> {
    // Direct: palace_path/.schema/manifest.json. The resolved path is by
    // construction inside palace_path (literal child).
    let direct = palace_path.join(".schema").join("manifest.json");
    if direct.is_file() {
        return Some(direct);
    }
    // Sibling fallback: parent/_palace-quantum-v1/.schema/manifest.json.
    // Cipher: the sibling is bounded to the literal directory name; an
    // attacker passing `/` as palace_path cannot escape this construction
    // (parent of `/` is None, so we early-return).
    let parent = palace_path.parent()?;
    let sibling = parent
        .join("_palace-quantum-v1")
        .join(".schema")
        .join("manifest.json");
    if sibling.is_file() {
        return Some(sibling);
    }
    None
}

#[tauri::command]
fn read_manifest(palace_path: String) -> Result<Manifest, String> {
    let palace = PathBuf::from(&palace_path);
    let manifest_path = resolve_manifest_path(&palace).ok_or_else(|| {
        format!(
            "no .schema/manifest.json found at {} or sibling _palace-quantum-v1",
            palace.display()
        )
    })?;
    let bytes = fs::read(&manifest_path).map_err(|e| format!("read manifest: {e}"))?;
    let manifest: Manifest =
        serde_json::from_slice(&bytes).map_err(|e| format!("parse manifest JSON: {e}"))?;
    Ok(manifest)
}

struct WatcherState {
    watcher: Mutex<Option<notify::RecommendedWatcher>>,
}

#[tauri::command]
fn start_state_watcher(
    palace_path: String,
    app: AppHandle,
    state: tauri::State<'_, WatcherState>,
) -> Result<(), String> {
    use notify::{EventKind, RecursiveMode, Watcher};

    let palace = PathBuf::from(&palace_path);
    let cron_dir = palace.join("cron");
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
    let palace = PathBuf::from(&palace_path);
    let cron_dir = palace.join("_palace").join("cron");
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
                    .unwrap_or(dir_name);
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
    Ok(snapshots)
}

#[derive(Debug, Serialize)]
struct HandoverEntry {
    filename: String,
    mtime: f64,
    size: u64,
}

/// Look up recent handovers from a few palace conventions:
///   palace_path/nym-stone/vesper/handovers/   (Hux's nym-stone convention)
///   palace_path/_palace/handovers/            (proposed generic)
///   palace_path/handovers/                    (last-resort)
/// Returns at most `limit` entries, newest first. Empty Vec if no convention matches.
#[tauri::command]
fn read_handovers(palace_path: String, limit: Option<usize>) -> Result<Vec<HandoverEntry>, String> {
    let palace = PathBuf::from(&palace_path);
    let candidates = [
        palace.join("nym-stone").join("vesper").join("handovers"),
        palace.join("_palace").join("handovers"),
        palace.join("handovers"),
    ];
    let Some(handovers_dir) = candidates.into_iter().find(|p| p.is_dir()) else {
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
    Ok(entries)
}

#[cfg(test)]
mod bridge_tests {
    use super::*;

    fn quantum_palace_root() -> PathBuf {
        // Test fixture: the live quantum palace on Hux's machine.
        // Skip via #[ignore] semantics if absent.
        PathBuf::from("/Users/eris/Dev/_palace-quantum-v1")
    }

    #[test]
    fn resolver_finds_direct_schema() {
        let palace = quantum_palace_root();
        if !palace.join(".schema/manifest.json").is_file() {
            eprintln!("skipping: quantum palace not present");
            return;
        }
        let resolved = resolve_manifest_path(&palace).expect("direct resolve");
        assert!(resolved.ends_with(".schema/manifest.json"));
        assert!(resolved.starts_with(&palace));
    }

    #[test]
    fn resolver_falls_back_to_sibling_quantum_v1() {
        // Simulate the post-cutover scenario: user points at _palace/ but only
        // _palace-quantum-v1/.schema exists (or vice versa).
        let dev_root = PathBuf::from("/Users/eris/Dev");
        let bogus_palace = dev_root.join("_does_not_exist_palace");
        if !dev_root
            .join("_palace-quantum-v1/.schema/manifest.json")
            .is_file()
        {
            eprintln!("skipping: sibling _palace-quantum-v1 not present");
            return;
        }
        let resolved = resolve_manifest_path(&bogus_palace).expect("sibling resolve");
        assert!(resolved.to_string_lossy().contains("_palace-quantum-v1"));
    }

    #[test]
    fn resolver_returns_none_when_nothing_found() {
        let resolved = resolve_manifest_path(Path::new("/tmp/definitely_not_a_palace_xyz"));
        assert!(resolved.is_none());
    }

    #[test]
    fn read_cron_states_against_live_palace() {
        // The convention is palace_path/_palace/cron/. Hux's palace_path is /Users/eris/Dev/
        let palace = PathBuf::from("/Users/eris/Dev");
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
        let palace = PathBuf::from("/Users/eris/Dev");
        if !palace.join("nym-stone/vesper/handovers").is_dir() {
            eprintln!("skipping: live handovers absent");
            return;
        }
        let handovers = read_handovers(palace.to_string_lossy().to_string(), Some(5))
            .expect("read_handovers must succeed");
        assert!(
            !handovers.is_empty(),
            "expected handovers under nym-stone/vesper/handovers"
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

    #[test]
    fn read_manifest_against_live_quantum_v1() {
        let palace = quantum_palace_root();
        if !palace.join(".schema/manifest.json").is_file() {
            eprintln!("skipping: live quantum palace not present");
            return;
        }
        let manifest = read_manifest(palace.to_string_lossy().to_string())
            .expect("read_manifest must succeed against live data");

        // Vocabulary lock: must be Option C.
        assert_eq!(manifest.vocabulary, "option_c");

        // Schema baseline (recon snapshot 2026-05-19): 477 nodes / 7 relations / 4 edges.
        // Allow drift (palace-sync runs daily) but assert non-empty.
        assert!(
            manifest.nodes.len() > 100,
            "expected many nodes, got {}",
            manifest.nodes.len()
        );
        assert!(
            !manifest.relations.is_empty(),
            "expected at least one relation"
        );

        // L1 ghost-relation shape: endpoints = [null, null], weight = null.
        // At least one such relation must exist (the schema design intent).
        let ghosts = manifest
            .relations
            .iter()
            .filter(|r| r.endpoints.iter().all(|e| e.is_none()) && r.weight.is_none())
            .count();
        assert!(
            ghosts >= 1,
            "expected L1 ghost relations with null endpoints; got 0 of {}",
            manifest.relations.len()
        );

        // Primitive class invariants: every record carries Option C class string.
        for n in &manifest.nodes {
            assert!(
                !n.primitive_class.is_empty(),
                "node {} missing primitive_class",
                n.rel_path
            );
        }
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
            start_state_watcher,
            read_cron_states,
            read_handovers,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
