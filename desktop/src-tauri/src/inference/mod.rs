// ─── Phase 4a: pluggable local inference ──────────────────────────────────────
//
// Roadmap constraint (locked 2026-05-26): "the chat talks to an inference TRAIT,
// not a vendor." Everything above this module asks an `InferenceBackend` to
// answer; nothing knows whether Ollama, LM Studio, or llama.cpp replied. Adding
// a provider means implementing this one trait — the chat UI never changes.
//
// Privacy contract (Cipher gate): every backend is fail-closed. `Err` is never a
// silent external fallback. URL validation (localhost / Tailscale only) stays in
// main.rs::validate_ollama_url; this module receives an already-validated base.

use loci_wal::EgressClass;
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// A local-first inference provider. One-shot completion plus a cheap probe.
pub trait InferenceBackend {
    /// One-shot chat completion. `system` carries the soul/persona grounding and
    /// is applied above the vendor, so every backend inherits the same identity
    /// (pass "" for no grounding). `Err` fails closed — callers must NOT fall back
    /// to any non-local provider on error.
    async fn chat(&self, system: &str, prompt: &str, model: &str) -> Result<String, String>;

    /// Reachability probe. Returns false when unreachable; never errors, so
    /// callers can use it inside a fail-closed decision without unwrapping.
    async fn health(&self) -> bool;

    /// Names of models the backend can serve right now. Empty on failure (a
    /// vendor that can't enumerate is treated as "nothing available").
    async fn available_models(&self) -> Vec<String>;

    /// Resolve `requested` against what's actually installed so the brain works
    /// without a pixel-perfect config. Exact match wins; else a stem match
    /// ("llama3" → "llama3.2:latest"); else the first available model. Errs only
    /// when nothing is pulled — an honest, actionable message.
    async fn resolve_model(&self, requested: &str) -> Result<String, String> {
        let models = self.available_models().await;
        if models.is_empty() {
            return Err("no local models pulled — run e.g. `ollama pull llama3.2`".into());
        }
        if let Some(exact) = models.iter().find(|m| *m == requested) {
            return Ok(exact.clone());
        }
        if let Some(stem) = models.iter().find(|m| m.starts_with(requested)) {
            return Ok(stem.clone());
        }
        Ok(models[0].clone())
    }

    /// This backend's egress class — the single classifier the WAL + gate read.
    /// No default: a new backend must declare whether its calls leave the device
    /// (so a provider can't be added without being classified).
    fn egress_class(&self) -> EgressClass;

    /// The effective destination host recorded in the egress receipt.
    fn dest_host(&self) -> String;
}

// ─── Ollama (first implementation) ────────────────────────────────────────────
//
// Uses the OpenAI-compatible /v1/chat/completions endpoint so the same impl
// works against any drop-in compatible local server.

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<Msg<'a>>,
    stream: bool,
}

#[derive(Serialize)]
struct Msg<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChoiceMsg,
}

#[derive(Deserialize)]
struct ChoiceMsg {
    content: String,
}

/// Ollama-backed inference. Holds a clone of the app-wide reqwest client
/// (cheap: reqwest::Client is Arc internally) and a pre-validated base URL.
pub struct OllamaBackend {
    client: reqwest::Client,
    base: url::Url,
}

impl InferenceBackend for OllamaBackend {
    async fn chat(&self, system: &str, prompt: &str, model: &str) -> Result<String, String> {
        let url = self
            .base
            .join("/v1/chat/completions")
            .map_err(|e| e.to_string())?;
        let mut messages = Vec::with_capacity(2);
        if !system.trim().is_empty() {
            messages.push(Msg {
                role: "system",
                content: system,
            });
        }
        messages.push(Msg {
            role: "user",
            content: prompt,
        });
        let body = ChatRequest {
            model,
            messages,
            stream: false,
        };
        let resp = self
            .client
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|_| "ollama_offline".to_string())?;
        if !resp.status().is_success() {
            // Surface the response BODY, not just the status: a 404 here means
            // "model 'X' not found", which is only legible from the body. Never
            // trust the status code alone (the 200-OK-or-404 trap).
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            let msg = serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|v| {
                    v.get("error")
                        .and_then(|e| e.get("message").or(Some(e)))
                        .map(|m| m.as_str().unwrap_or(&body).to_string())
                })
                .unwrap_or(body);
            return Err(format!("ollama {status}: {msg}"));
        }
        let parsed: ChatResponse = resp.json().await.map_err(|e| e.to_string())?;
        parsed
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| "ollama returned empty choices".to_string())
    }

    async fn health(&self) -> bool {
        match self.base.join("/api/tags") {
            Ok(url) => self
                .client
                .get(url)
                .send()
                .await
                .map(|r| r.status().is_success())
                .unwrap_or(false),
            Err(_) => false,
        }
    }

    async fn available_models(&self) -> Vec<String> {
        let Ok(url) = self.base.join("/api/tags") else {
            return Vec::new();
        };
        let Ok(resp) = self.client.get(url).send().await else {
            return Vec::new();
        };
        if !resp.status().is_success() {
            return Vec::new();
        }
        resp.json::<TagsResponse>()
            .await
            .map(|t| t.models.into_iter().map(|m| m.name).collect())
            .unwrap_or_default()
    }

    fn egress_class(&self) -> EgressClass {
        // Local means loopback ONLY. A permitted non-loopback host (a Tailscale
        // peer) physically leaves this machine, so it is LocalNetwork, not Local.
        match self.base.host_str() {
            Some("localhost") | Some("127.0.0.1") | Some("::1") | Some("[::1]") => {
                EgressClass::Local
            }
            _ => EgressClass::LocalNetwork,
        }
    }

    fn dest_host(&self) -> String {
        self.base.host_str().unwrap_or("localhost").to_string()
    }
}

// ─── Claude (external brain — the "online garden" tier) ─────────────────────────
//
// Shells out to the Claude Code CLI (`claude -p`), which authenticates with the
// user's own Pro/Max subscription via OAuth: no API key, no separate billing.
// EXTERNAL by definition — it leaves the local garden — so it is only ever
// constructed on an explicit, UI-marked opt-in, never as a silent fallback.
pub struct ClaudeBackend {
    /// Absolute path to the `claude` binary (a GUI app can't see shell aliases).
    bin: std::path::PathBuf,
    /// PATH for the spawned process, augmented with the node dir. The CLI is a
    /// node script whose shebang needs `node`, which lives in nvm/homebrew and is
    /// invisible to a bundled app's minimal PATH.
    path_env: std::ffi::OsString,
}

impl InferenceBackend for ClaudeBackend {
    async fn chat(&self, system: &str, prompt: &str, model: &str) -> Result<String, String> {
        let mut cmd = tokio::process::Command::new(&self.bin);
        cmd.env("PATH", &self.path_env)
            // The spawned CLI inherits the user's global Claude config, which can
            // carry MCP connectors (Jira, Figma, …) that handshake third-party
            // hosts from a session that needs none of them. --strict-mcp-config
            // with no --mcp-config loads zero MCP servers; the env var drops the
            // Statsig/Sentry/auto-update phone-home. Egress stays one host: the API.
            .env("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", "1")
            .arg("-p")
            .arg(prompt)
            .arg("--model")
            .arg(model)
            .arg("--strict-mcp-config")
            .arg("--output-format")
            .arg("text");
        if !system.trim().is_empty() {
            // Replace (not append) the default coding-agent system prompt so the
            // external brain answers in the assistant voice, not as the base coding agent.
            cmd.arg("--system-prompt").arg(system);
        }
        let out = cmd
            .output()
            .await
            .map_err(|e| format!("claude spawn failed: {e}"))?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            let msg = if err.trim().is_empty() { "(no stderr)" } else { err.trim() };
            return Err(format!("claude exited unsuccessfully: {msg}"));
        }
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    async fn health(&self) -> bool {
        // Actually run it: a present binary that can't find node is not "awake".
        tokio::process::Command::new(&self.bin)
            .env("PATH", &self.path_env)
            .env("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", "1")
            .arg("--version")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    async fn available_models(&self) -> Vec<String> {
        // The CLI resolves aliases server-side; we offer the stable aliases.
        vec!["sonnet".to_string(), "opus".to_string(), "haiku".to_string()]
    }

    fn egress_class(&self) -> EgressClass {
        EgressClass::ExternalCloud
    }

    fn dest_host(&self) -> String {
        // Effective destination: the CLI authenticates to Anthropic's API.
        "api.anthropic.com".to_string()
    }
}

#[derive(Deserialize)]
struct TagsResponse {
    models: Vec<TagModel>,
}

#[derive(Deserialize)]
struct TagModel {
    name: String,
}

// ─── Egress chokepoint ──────────────────────────────────────────────────────
//
// Wraps any backend so a WAL frame is written at the single point content
// leaves via `chat`. This is the chokepoint the egress-receipt feature rests on:
// content egress is recorded here, not as a convention each call site remembers.
// The frame is written BEFORE the inner call — over-reporting an attempt is the
// safe direction for a privacy receipt; never under-report what may have left.

/// The egress receipt WAL path, shared with `loci audit`: ~/.loci/wal/egress.jsonl.
pub fn egress_wal_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".loci")
        .join("wal")
        .join("egress.jsonl")
}

/// Decorator that records an egress frame on every `chat`, then delegates.
/// Not built directly by call sites — obtain a backend via `ollama()` / `claude()`,
/// the ONLY constructors, so no site can egress content without this chokepoint.
pub struct EgressLogged<B: InferenceBackend> {
    inner: B,
    wal_path: PathBuf,
}

impl<B: InferenceBackend> EgressLogged<B> {
    pub fn new(inner: B, wal_path: PathBuf) -> Self {
        Self { inner, wal_path }
    }

    fn record(&self, prompt: &str) {
        // Kill switch: LOCI_WAL_DISABLED turns the writer off without a rebuild.
        if std::env::var_os("LOCI_WAL_DISABLED").is_some() {
            return;
        }
        if let Some(parent) = self.wal_path.parent() {
            let _ = create_dir_all(parent);
        }
        if let Err(e) = loci_wal::record_egress(
            &self.wal_path,
            chrono::Utc::now().to_rfc3339(),
            "chat",
            self.inner.egress_class(),
            &self.inner.dest_host(),
            prompt.as_bytes(),
            None,
        ) {
            // A dropped write leaves NO gap in the chain, so it is invisible to
            // `loci audit`. Surface it: a degraded marker the audit reads + warns on.
            let marker = self.wal_path.with_file_name("egress.degraded");
            let _ = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&marker)
                .and_then(|mut f| writeln!(f, "{} {}", chrono::Utc::now().to_rfc3339(), e));
        }
    }
}

impl<B: InferenceBackend> InferenceBackend for EgressLogged<B> {
    async fn chat(&self, system: &str, prompt: &str, model: &str) -> Result<String, String> {
        // Record BEFORE the call: over-report an attempt, never under-report an egress.
        self.record(prompt);
        self.inner.chat(system, prompt, model).await
    }

    async fn health(&self) -> bool {
        self.inner.health().await
    }

    async fn available_models(&self) -> Vec<String> {
        self.inner.available_models().await
    }

    fn egress_class(&self) -> EgressClass {
        self.inner.egress_class()
    }

    fn dest_host(&self) -> String {
        self.inner.dest_host()
    }
}

/// The ONLY constructors for a callable backend. Because the backend struct
/// fields are private to this module, a call site cannot build a raw (unlogged)
/// backend — it comes through here, wrapped in the egress chokepoint.
pub fn ollama(client: reqwest::Client, base: url::Url) -> EgressLogged<OllamaBackend> {
    EgressLogged::new(OllamaBackend { client, base }, egress_wal_path())
}

pub fn claude(bin: PathBuf, path_env: OsString) -> EgressLogged<ClaudeBackend> {
    EgressLogged::new(ClaudeBackend { bin, path_env }, egress_wal_path())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeBackend {
        class: EgressClass,
        host: String,
    }

    impl InferenceBackend for FakeBackend {
        async fn chat(&self, _s: &str, _p: &str, _m: &str) -> Result<String, String> {
            Ok("ok".to_string())
        }
        async fn health(&self) -> bool {
            true
        }
        async fn available_models(&self) -> Vec<String> {
            vec!["m".to_string()]
        }
        fn egress_class(&self) -> EgressClass {
            self.class
        }
        fn dest_host(&self) -> String {
            self.host.clone()
        }
    }

    #[tokio::test]
    async fn egress_logged_records_a_classed_payload_free_frame() {
        let dir = std::env::temp_dir().join(format!("loci_egress_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let wal_path = dir.join("egress.jsonl");
        let backend = EgressLogged::new(
            FakeBackend {
                class: EgressClass::ExternalCloud,
                host: "api.anthropic.com".to_string(),
            },
            wal_path.clone(),
        );

        let out = backend.chat("sys", "the secret prompt", "sonnet").await.unwrap();
        assert_eq!(out, "ok");

        let frames = loci_wal::Wal::open(&wal_path).read().unwrap();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].egress_class, EgressClass::ExternalCloud);
        assert_eq!(frames[0].dest_host, "api.anthropic.com");
        // The prompt bytes must NOT be on disk — only a hash.
        let raw = std::fs::read_to_string(&wal_path).unwrap();
        assert!(!raw.contains("the secret prompt"), "payload must not be stored");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn ollama_tailscale_is_local_network_not_local() {
        let c = reqwest::Client::new();
        let loopback = OllamaBackend {
            client: c.clone(),
            base: url::Url::parse("http://127.0.0.1:11434").unwrap(),
        };
        assert_eq!(loopback.egress_class(), EgressClass::Local);
        // A Tailscale peer physically leaves this machine — must not read as Local.
        let tailnet = OllamaBackend {
            client: c,
            base: url::Url::parse("http://100.64.1.5:11434").unwrap(),
        };
        assert_eq!(tailnet.egress_class(), EgressClass::LocalNetwork);
    }
}
