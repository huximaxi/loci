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

use serde::{Deserialize, Serialize};

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
    pub client: reqwest::Client,
    pub base: url::Url,
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
}

// ─── Claude (external brain — the "online garden" tier) ─────────────────────────
//
// Shells out to the Claude Code CLI (`claude -p`), which authenticates with the
// user's own Pro/Max subscription via OAuth: no API key, no separate billing.
// EXTERNAL by definition — it leaves the local garden — so it is only ever
// constructed on an explicit, UI-marked opt-in, never as a silent fallback.
pub struct ClaudeBackend {
    /// Absolute path to the `claude` binary (a GUI app can't see shell aliases).
    pub bin: std::path::PathBuf,
    /// PATH for the spawned process, augmented with the node dir. The CLI is a
    /// node script whose shebang needs `node`, which lives in nvm/homebrew and is
    /// invisible to a bundled app's minimal PATH.
    pub path_env: std::ffi::OsString,
}

impl InferenceBackend for ClaudeBackend {
    async fn chat(&self, system: &str, prompt: &str, model: &str) -> Result<String, String> {
        let mut cmd = tokio::process::Command::new(&self.bin);
        cmd.env("PATH", &self.path_env)
            .arg("-p")
            .arg(prompt)
            .arg("--model")
            .arg(model)
            .arg("--output-format")
            .arg("text");
        if !system.trim().is_empty() {
            // Replace (not append) the default coding-agent system prompt so the
            // external brain answers as Vesper, not as Claude Code.
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
}

#[derive(Deserialize)]
struct TagsResponse {
    models: Vec<TagModel>,
}

#[derive(Deserialize)]
struct TagModel {
    name: String,
}
