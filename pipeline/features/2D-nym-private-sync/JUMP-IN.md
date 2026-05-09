# 2D · Nym Private Sync
## Jump-In Brief

**Persona:** Kata + Cipher — mixnet SDK integration, fail-closed architecture
**Tier:** 2 · **Target:** Q4 2026
**Status:** `⏸ blocked on 1D (partnership confirmed)`
**Branch:** `feat/2D-nym-sync`
**Last updated:** 2026-05-09

---

## Context
Loci protects the content of your thinking. Nym protects the *fact* that you're thinking — who you're syncing with, when, how often. Together they close a gap neither project can close alone.

When this ships, a user who turns on Nym routing in Loci settings has their sync traffic, search queries, and AI calls routed through the Nym mixnet — metadata-invisible to any network observer, including Nym's own relays (cover traffic architecture). Nothing changes visually. That's the point.

**Fail-closed is non-negotiable.** If Nym routing fails for any reason, Loci must NOT silently fall back to unrouted sync. The user must be notified. This is Cipher's hard rule.

---

## Current State

No sync mechanism exists (git-based sync experiment closed). The Tauri app operates purely local. This feature adds an *optional* transport layer for sync operations (Tauri ↔ any future sync endpoints, or extension ↔ Tauri app communication over Tailscale).

**Hux controls both sides** — Head of UX/Product at Nym and co-building Loci. The Nym SDK integration should be scoped with the Nym engineering team directly.

---

## What Needs to Be Built

1. **Nym SDK integration** (Tauri/Rust) — integrate `nym-sdk` Rust crate. Configure a Nym client that routes HTTP requests through the mixnet.
2. **SOCKS5 proxy mode** (simpler path) — Nym can expose a local SOCKS5 proxy. Tauri HTTP requests can be routed through this proxy. Less code, same privacy properties.
3. **Routing toggle** — Wizard settings panel: "Route Loci through Nym mixnet" toggle. Enabled state clearly indicated in UI.
4. **Fail-closed behaviour** — if Nym routing unavailable: disable sync, show "Nym routing unavailable — sync paused" notification. Never silently fall back. User explicitly re-enables or explicitly disables Nym routing.
5. **Scope sync surface** — what exactly gets routed? In v1: any future Tauri ↔ remote communication (backup, optional cloud features). Local-only users are unaffected.

---

## Technical Entry Points

```
desktop/src-tauri/Cargo.toml        ← add nym-sdk or SOCKS5 proxy client
desktop/src-tauri/src/nym.rs        ← create: Nym client init, routing layer
desktop/src-tauri/src/main.rs       ← expose enable_nym_routing() Tauri command
packages/core/src/types.ts          ← extend LociConfig with NymConfig
```

**NymConfig type:**
```typescript
interface NymConfig {
  enabled: boolean;
  mode: 'socks5' | 'sdk';  // v1: socks5 is simpler
  proxy_port: number;       // default: 1080
  gateway?: string;         // optional: specific Nym gateway
}
```

**Fail-closed pattern:**
```rust
async fn sync_with_nym_routing(data: SyncPayload) -> Result<(), LociError> {
    if !nym_client_available() {
        return Err(LociError::NymUnavailable);
        // Never fall through to unrouted sync
    }
    // proceed with routed sync
}
```

---

## Dependencies
- **1D (Nym Partnership)** — scope with Nym engineering team before implementing. Don't build against an unstable SDK version.
- **No existing sync** — this feature's scope is the transport layer for future sync, not a new sync feature itself. Coordinate with any future "Loci sync" feature spec.

---

## Cipher's Gate
- **Fail-closed is absolute.** If `nym_routing: enabled` and routing fails: error state, not silent fallback. This is the key invariant.
- **No Nym credentials stored in Loci.** Nym client uses gateway credentials — these must stay in the Nym SDK's own storage, not in `~/.loci/`.
- **SOCKS5 port binding:** localhost only (`127.0.0.1:1080`). Not `0.0.0.0`.
- **Tauri capability scope:** HTTP allowlist should not broaden when Nym routing is enabled — Nym handles routing transparently.

---

## Acceptance Criteria
- [ ] Nym routing toggle in Wizard settings
- [ ] When enabled, all Loci sync traffic routes through Nym SOCKS5 proxy
- [ ] If Nym unavailable: sync paused, clear user notification, no silent fallback
- [ ] UI shows "Nym routing active" badge when enabled
- [ ] Works with NymVPN also running (no port conflict)
- [ ] Scoped with Nym engineering team before implementation

---

## Changelog
- 2026-05-09: Brief created. Blocked pending 1D partnership confirmation.

---

## First Move
> After 1D confirmed: sync with Nym engineering team on SDK version and preferred integration approach (SOCKS5 vs SDK) → scope `NymConfig` in `LociConfig` → implement SOCKS5 proxy path first (simpler, lower risk)
