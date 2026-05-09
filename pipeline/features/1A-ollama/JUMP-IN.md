# 1A · Ollama Local Inference
## Jump-In Brief

**Persona:** Kata — trench-worker, infrastructure first
**Tier:** 1 · **Target:** Q3 2026
**Status:** `🔴 not-started`
**Branch:** `feat/1A-ollama`
**Last updated:** 2026-05-09

---

## Context
Every AI feature in Loci currently requires an external API call — OpenAI, Anthropic, or similar. That's not sovereignty. Ollama is the local LLM runtime the sovereignty community actually runs on: `ollama run llama3` is as simple as it gets. It exposes an OpenAI-compatible API at `localhost:11434`, which means integration cost is minimal — the same interface, different host.

This is the plumbing everything else depends on. `nomic-embed-text` via Ollama also provides local semantic indexing of `Locus` and `Conversation` objects without any note leaving the device. The MCP server (1B) benefits from local embeddings. Private sync (2D) benefits from local AI. Everything downstream is cleaner once this ships.

---

## Current State

The extension (`extension/src/shared/search.ts`) uses MiniSearch for keyword + fuzzy search — no embeddings, no semantic layer. The Tauri `LociConfig` type has `llm_config` as a field, already anticipating a pluggable LLM backend. No Ollama integration exists yet.

**Relevant files:**
- `packages/core/src/types.ts` → `LociConfig.llm_config` — define Ollama config here
- `extension/src/shared/search.ts` → MiniSearch, candidate for embedding upgrade
- `desktop/src-tauri/` → Rust backend, will host Ollama API bridge
- `desktop/src-tauri/tauri.conf.json` → allowlist for localhost:11434 requests

---

## What Needs to Be Built

1. **Ollama API bridge** — Tauri command `call_ollama(prompt, model)` that hits `localhost:11434/api/chat` (OpenAI-compatible). Returns streamed or full response.
2. **Embedding endpoint** — `embed_text(text: string): number[]` via Ollama's `/api/embeddings`. Default model: `nomic-embed-text`.
3. **Model selector UI** — Wizard settings panel: fetch available models from `localhost:11434/api/tags`, display as dropdown. Scholar gets a single "Use local AI" toggle.
4. **Offline badge** — UI indicator (small icon, status bar) showing when Loci is running fully local. Disappears if Ollama is unreachable — never silently falls back to external API.
5. **Graceful degradation** — If Ollama is offline: surface a clear message, disable AI features, do NOT silently call external API.

---

## Technical Entry Points

```
desktop/src-tauri/src/main.rs       ← add #[tauri::command] fn call_ollama()
packages/core/src/types.ts          ← extend LociConfig with OllamaConfig
desktop/src/settings/               ← add OllamaSettings component
extension/src/shared/search.ts      ← phase 2: upgrade to embedding search
```

**Ollama API surface used:**
```
GET  localhost:11434/api/tags          → list models
POST localhost:11434/api/chat          → chat completion (OpenAI-compatible)
POST localhost:11434/api/embeddings    → embed text
```

---

## Implementation Spec

```typescript
// packages/core/src/types.ts — add to LociConfig
interface OllamaConfig {
  enabled: boolean;
  base_url: string;           // default: "http://localhost:11434"
  chat_model: string;         // default: "llama3"
  embed_model: string;        // default: "nomic-embed-text"
  offline_mode: boolean;      // if true, never fallback to external API
}
```

```rust
// desktop/src-tauri/src/main.rs
#[tauri::command]
async fn call_ollama(prompt: String, model: String) -> Result<String, String> {
    // POST to localhost:11434/v1/chat/completions
    // Return response or Err("ollama_offline")
}

#[tauri::command]
async fn embed_text(text: String) -> Result<Vec<f32>, String> {
    // POST to localhost:11434/api/embeddings
    // model: from config (default nomic-embed-text)
}
```

**Tauri allowlist** — add to `tauri.conf.json`:
```json
"allowlist": {
  "http": { "request": true, "scope": ["http://localhost:11434/**"] }
}
```

---

## Dependencies
None. This is the foundation — ships first.

---

## Cipher's Gate
- `localhost:11434` is in the Tauri allowlist — no broader network permission required.
- If `offline_mode: true`, the Tauri command must return `Err("ollama_offline")` and the frontend must surface this clearly — no silent external fallback.
- User-supplied `base_url` (for Tailscale-hosted Ollama) must be validated — no arbitrary URL injection.

---

## Acceptance Criteria
- [ ] Users can run Loci with zero external API calls when Ollama is running
- [ ] Model selector shows available models from Ollama
- [ ] `nomic-embed-text` is default embedding model
- [ ] If Ollama is unreachable, UI shows clear error — no silent fallback
- [ ] `offline_mode` toggle in settings works as described
- [ ] "Run offline" documented in README and Scholar onboarding

---

## Changelog
- 2026-05-09: Brief created

---

## First Move
> `git checkout -b feat/1A-ollama` → add `OllamaConfig` to `packages/core/src/types.ts` → write `call_ollama` Tauri command → test against a local Ollama instance
