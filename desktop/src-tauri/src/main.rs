// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::Mutex;

// ─── 1B: MCP server ───────────────────────────────────────────────────────────
mod mcp;

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

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let actual_port = mcp::server::start_server(requested_port, base, shutdown_rx)
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

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(ollama_client)
        .manage(mcp_handle)
        .invoke_handler(tauri::generate_handler![
            detect_palace,
            migrate_to_loci,
            // 1A — Ollama local inference
            check_ollama_health,
            list_ollama_models,
            call_ollama,
            embed_text,
            // 1B — MCP server
            start_mcp_server,
            stop_mcp_server,
            mcp_server_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
