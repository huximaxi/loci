# 1B · Goose MCP Plugin
## Jump-In Brief

**Persona:** Cipher — security-first architect
**Tier:** 1 · **Target:** Q3 2026
**Status:** `🔴 not-started`
**Branch:** `feat/1B-goose-mcp`
**Last updated:** 2026-05-09

---

## Context
The MCP server (Model Context Protocol) is already specced as a Tauri command (`start_mcp_server`) in the desktop scope. This feature implements it. When shipped, Loci becomes a knowledge source that any MCP-compatible AI agent (Goose, Continue.dev, Claude Code, etc.) can query. The integration is with the **MCP spec**, not Goose specifically — Goose is the named distribution hook.

This is Loci's first external API. Since there's no prior API surface, the MCP resource schema *is* the design — built sovereign-first.

**THREAT-01 gate (critical):** `Locus` nodes (user-authored) are safe to expose immediately. Raw `Conversation` objects from IndexedDB carry prompt injection risk — AI-generated content could execute via MCP context. Conversation context ships only after the sanitisation sprint is complete (parallel track). This brief covers Locus + Room exposure. Conversation context is a follow-on.

---

## Current State

The Tauri scope doc (`_archive/loci-interim/loci-tauri-scope.md`) already defines:
- `start_mcp_server` as a planned Tauri command
- `Locus` type in `packages/core/src/types.ts`
- `Room` type with `contextMd` field

**Relevant files:**
- `packages/core/src/types.ts` → `Locus`, `Room`, `LociConfig.mcp_config`
- `desktop/src-tauri/src/main.rs` → add MCP server Tauri command
- `desktop/src-tauri/Cargo.toml` → add `rmcp` or custom MCP server crate

---

## What Needs to Be Built

1. **MCP server process** — spawned by Tauri on startup or on demand. Listens on a configurable port (default: `localhost:3456`). Exposes MCP resources and tools.
2. **MCP resources:**
   - `loci://locus/{id}` — single Locus node (title, content, tags, roomId, createdAt)
   - `loci://room/{id}/loci` — all Loci in a Room
   - `loci://search?q={query}` — MiniSearch over Loci titles + content
3. **MCP tools:**
   - `create_locus(title, content, tags, room_id)` — write a new insight node from agent
   - `tag_locus(id, tags)` — agent can tag existing Loci
4. **`loci-mcp-server` repo** — extracted as a standalone open-source package (Apache 2.0). Can run independently of the Tauri app for power users.
5. **Auth (optional):** MCP requests may carry a Nostr pubkey header for consistency. Not required — local-only servers are implicitly trusted. Honour the auth principle: available, never gated.

---

## Technical Entry Points

```
desktop/src-tauri/src/main.rs      ← start_mcp_server Tauri command
desktop/src-tauri/Cargo.toml       ← add MCP server dependency
packages/core/src/types.ts         ← Locus, Room types (already defined)
desktop/src-tauri/src/mcp/         ← create this module
  mod.rs
  resources.rs    ← loci://* resource handlers
  tools.rs        ← create_locus, tag_locus
  server.rs       ← HTTP server (axum or tiny-http)
```

**MCP resource schema (JSON):**
```json
{
  "uri": "loci://locus/locus-2026-05-09-sovereignty",
  "name": "Cognitive Sovereignty — initial framing",
  "mimeType": "text/markdown",
  "text": "...",
  "metadata": {
    "tags": ["sovereignty", "loci"],
    "roomId": "research",
    "createdAt": "2026-05-09T10:00:00Z"
  }
}
```

---

## Cipher's Gate

**THREAT-01 (CRITICAL — blocks conversation context only):**
- `Locus` resources: safe. User-authored Markdown.
- `Room.contextMd`: safe. User-authored.
- `Conversation` objects: **NOT safe until sanitised**. Do not expose `Conversation` turns via MCP until THREAT-01 sprint is complete.
- Prepend to all MCP responses: `X-Loci-Content-Trust: user-authored` or `untrusted` header.

**THREAT-06 (future — native host):** If MCP server runs as a standalone binary rather than in-process, sign the binary. Minimal protocol surface only — no shell command interpretation.

**Port binding:** Default `localhost:3456`. Document in settings. Never bind to `0.0.0.0` by default — localhost only. Tailscale users set `base_url` explicitly.

---

## Acceptance Criteria
- [ ] `start_mcp_server` Tauri command spawns server on `localhost:3456`
- [ ] `loci://locus/{id}` resource returns correct Markdown + metadata
- [ ] `loci://room/{id}/loci` returns all Loci in a Room
- [ ] `loci://search?q=` returns ranked results via MiniSearch
- [ ] `create_locus` tool writes a new Locus to IndexedDB / `~/.loci/`
- [ ] Raw `Conversation` objects NOT exposed (THREAT-01 gate)
- [ ] Goose can connect and query the garden
- [ ] `loci-mcp-server` published to GitHub (Apache 2.0)
- [ ] Listed on MCP registry

---

## Changelog
- 2026-05-09: Brief created. THREAT-01 gate documented.

---

## First Move
> `git checkout -b feat/1B-goose-mcp` → create `desktop/src-tauri/src/mcp/` module → implement `start_mcp_server` Tauri command returning server port → write `loci://locus/{id}` resource handler reading from `~/.loci/loci/` → test with Goose MCP connection
