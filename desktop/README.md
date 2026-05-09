# loci Desktop App

Tauri v2 desktop application. Scholar-themed. Local-first.

**v0.4.0** — Ollama local inference (1A) + Goose MCP server (1B)

## What it does

Three things:

1. **Palace detection + migration** — finds existing memory palace setups (`_palace/`, `mila-mempalace/`, Karpathy `LLM/`) and migrates them to `~/.loci/` format.
2. **Ollama local inference** — connects to a local Ollama instance for AI features with zero API keys. Fails closed — never silently calls external APIs.
3. **MCP server** — exposes your knowledge garden as MCP resources so any compatible agent (Goose, Continue.dev, Claude Code) can query and extend it.

## Requirements

- Node.js 18+
- Rust 1.70+
- Tauri CLI 2.0+
- Optional: [Ollama](https://ollama.ai) for local AI features
- Optional: Goose or any MCP-compatible agent to consume the MCP server

## Development

```bash
npm install
npm run tauri:dev   # dev mode
npm run tauri:build # production build
```

## Architecture

- **Frontend**: Vite + TypeScript + Scholar theme (green/cream)
- **Backend**: Rust + Tauri v2
- **HTTP**: axum 0.7 (MCP server, localhost only)
- **Plugins**: `tauri-plugin-dialog`, `tauri-plugin-opener`

## Files

```
desktop/
├── index.html             # UI — Scholar status bar + settings overlay
├── src/
│   └── main.ts            # Frontend — Ollama state machine, MCP toggle, health poll
└── src-tauri/
    ├── Cargo.toml         # reqwest, axum, tokio, url, dirs, chrono
    ├── tauri.conf.json    # v0.4.0
    └── src/
        ├── main.rs        # Tauri commands (see table below)
        └── mcp/
            ├── mod.rs     # Module root + Cipher gate docs
            ├── server.rs  # axum JSON-RPC 2.0 server (MCP spec 2024-11-05)
            ├── resources.rs # loci://locus/{id}, loci://room/{id}/loci, loci://search
            └── tools.rs   # create_locus, tag_locus
```

## Tauri commands

| Command | Description |
|---------|-------------|
| `detect_palace` | Find existing palace at a given path |
| `migrate_to_loci` | Copy palace to `~/.loci/` |
| `check_ollama_health` | Probe `localhost:11434` — returns bool |
| `list_ollama_models` | Fetch available models from Ollama |
| `call_ollama` | Chat completion via Ollama OpenAI-compat API |
| `embed_text` | Embed text via `nomic-embed-text` (or custom model) |
| `start_mcp_server` | Start MCP server at `localhost:{port}` (default 3456) |
| `stop_mcp_server` | Graceful shutdown |
| `mcp_server_status` | Returns `{running, port}` |

## MCP server

JSON-RPC 2.0 over HTTP POST at `http://localhost:3456/`.

**Resources:** `loci://locus/{id}` · `loci://room/{roomId}/loci` · `loci://search?q={query}`

**Tools:** `create_locus(title, content, tags?, room_id?)` · `tag_locus(id, tags)`

**Goose config:**
```json
{"mcpServers": {"loci": {"url": "http://localhost:3456"}}}
```

**Security:** `127.0.0.1` bind only. `X-Loci-Content-Trust: user-authored` on all responses. Conversation objects never exposed (THREAT-01 gate). Port range: 1024–65535.

## Ollama integration

URL validation: `localhost` and Tailscale CGNAT (`100.64.x.x–100.127.x.x`) only. `offline_mode: true` by default — no silent fallback to external APIs.

## Storage

`~/.loci/loci/{id}.md` — YAML frontmatter + Markdown body.

## Window

- **Size**: 680×520px (not resizable)
- **Theme**: Scholar (green `#4a6b54` / cream `#faf9f6`)

## Security gates (Cipher)

- THREAT-01: Conversation objects never exposed via MCP
- SSRF: Ollama `base_url` validated before every HTTP call
- Path traversal: all Locus IDs validated (alphanumeric + hyphens only)
- Port: MCP server refuses to bind below 1024
