# 2H · Tailscale Local Brain
## Jump-In Brief

**Persona:** Cipher — architecture, network topology, sovereign compute
**Tier:** 2 · **Target:** Q4 2026 (ADR Q3, prototype Q4)
**Status:** `📋 specced`
**Branch:** `feat/2H-tailscale`
**Last updated:** 2026-05-09

---

## Context
Two independent sources arrived at the same architecture in 2026: Hux's "Tailscale-driven local brains, VPS-hosted full autonomous palace instances" and Max's (Nym integration engineer) Slide 7 — "have dev env on a specific Tailscale partition accessible remotely, CC and dev env lives there."

This is the pattern: Loci (Tauri desktop app + MCP server) running on a persistent Tailscale node — home server or VPS. The browser extension on any device connects to the same brain over the Tailscale mesh. Agents (Goose, Claude Code, scheduled tasks) connect to the same MCP server. One sovereign brain, multiple clients, no cloud intermediary.

This also resolves the open architecture question from the Tauri scope doc: **native messaging vs local HTTP**. Answer: local HTTP over Tailscale. The same interface that serves the browser extension also serves remote agents. One surface, multiple clients.

For Hux's palace instances: a persistent autonomous Vesper-type agent running on a Tailscale VPS, holding the full garden, running workflows on schedule. This is that infrastructure.

---

## Architecture

```
[Browser extension — any device on Tailscale mesh]
        ↕  local HTTP  (ws://100.x.x.x:7891)
[Loci Tauri app — Tailscale node: home server or VPS]
        ↕  MCP server  (http://100.x.x.x:3456)
[Goose / Claude Code / cron agent — anywhere on mesh]
        ↕  filesystem
[~/.loci/ — SQLite + markdown files — persistent garden]
```

Key properties:
- **Tailscale IP** is stable within the mesh. `100.x.x.x` address doesn't change.
- **No open ports to internet.** Tailscale mesh is authenticated and encrypted.
- **Extension config:** single `LOCI_HOST` env — defaults to `localhost`, can be set to Tailscale IP of the brain node.
- **MCP server** already on `localhost:3456` (from 1B) — on a Tailscale node, same port, accessible to the whole mesh.

---

## What Needs to Be Built

### Phase 1 — ADR (Q3 2026)
Write Architecture Decision Record: native messaging vs local HTTP.
- **Decision: local HTTP over Tailscale.**
- Rationale: portable (works local and remote), no native host registration required, same surface for browser extension and AI agents, Tailscale handles auth and encryption.
- Document Tailscale reference config.

### Phase 2 — Prototype (Q4 2026)
1. **`LOCI_HOST` config** in extension and desktop — make the host configurable. Default: `localhost`. Override: Tailscale IP of brain node.
2. **Extension → remote Loci connection** — extension `service-worker.ts` uses `chrome.storage.local` for `loci_host` setting. All local HTTP calls use this host.
3. **Tauri app binding** — on non-localhost installs, Tauri HTTP server binds to Tailscale interface only (never `0.0.0.0`).
4. **Tailscale setup guide** — `pipeline/features/2H-tailscale-brain/SETUP-GUIDE.md`. Step-by-step: install Tailscale → install Loci on server → configure `LOCI_HOST` in extension → verify connection.
5. **Palace instance pattern** — document how to run an autonomous agent (Claude Code / Goose) against the Tailscale Loci brain on a schedule. This is the foundation for Hux's palace instances.

---

## Technical Entry Points

```
extension/src/background/service-worker.ts    ← add LOCI_HOST config
extension/src/shared/db.ts                    ← HTTP client for remote Loci
desktop/src-tauri/src/server.rs               ← bind to Tailscale interface
pipeline/features/2H-tailscale-brain/ADR.md   ← write this first
pipeline/features/2H-tailscale-brain/SETUP-GUIDE.md
```

**`LOCI_HOST` in extension:**
```typescript
// service-worker.ts
const LOCI_HOST = (await chrome.storage.local.get('loci_host')).loci_host
  ?? 'localhost';
const LOCI_MCP_URL = `http://${LOCI_HOST}:3456`;
```

**Tauri binding (Rust):**
```rust
// Bind only to Tailscale interface (100.x.x.x) when running as brain node
let bind_addr = if is_tailscale_node() {
    get_tailscale_ip().unwrap_or("127.0.0.1".into())
} else {
    "127.0.0.1".into()
};
```

---

## Dependencies
- **1B (MCP server)** — the MCP server is the primary remote-accessible surface.
- **Tailscale** installed on both the brain node and client devices (user-managed; not a Loci dependency).

---

## Cipher's Gate

**Binding invariant:** Loci HTTP server MUST bind only to `127.0.0.1` (local) or the Tailscale interface (`100.x.x.x`). Never `0.0.0.0`. Never a public interface. Tailscale handles auth — Loci doesn't need to implement its own auth for the mesh (all mesh peers are authenticated by Tailscale's WireGuard layer).

**No ACL bypass:** The setup guide must clearly state: configure Tailscale ACLs so only authorised devices can reach the Loci node. A misconfigured Tailscale ACL = MCP server accessible to all mesh peers including less-trusted devices.

**Extension origin:** If the extension connects to a remote Loci over Tailscale, `manifest.json` host permissions must include the Tailscale IP range (`100.64.0.0/10`). This is a permission scope that Cipher must review before shipping.

---

## Acceptance Criteria
- [ ] ADR written and decided (local HTTP over Tailscale)
- [ ] `LOCI_HOST` configurable in extension settings
- [ ] Extension connects to remote Loci Tauri app over Tailscale IP
- [ ] Tauri server binds to Tailscale interface only (never 0.0.0.0)
- [ ] Tailscale setup guide written and tested
- [ ] Palace instance pattern documented
- [ ] Cipher ACL review complete before shipping

---

## Changelog
- 2026-05-09: Brief created. Architecture decision documented.

---

## First Move
> `git checkout -b feat/2H-tailscale` → write `ADR.md` in this folder → document the native-messaging vs local-HTTP decision → get Hux sign-off → then implement `LOCI_HOST` config in extension
