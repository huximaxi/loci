# 2E · IPFS Garden Export
## Jump-In Brief

**Persona:** Kata — storage infrastructure, content addressing
**Tier:** 2 · **Target:** Q4 2026
**Status:** `🔴 not-started`
**Branch:** `feat/2E-ipfs`
**Last updated:** 2026-05-09

---

## Context
IPFS (InterPlanetary File System) is content-addressed, decentralized storage. A garden exported to IPFS is accessible forever — without Loci's servers, without the user's device being online, without any single point of failure. The CID (Content Identifier) is the address; anyone with it can retrieve the content.

This is the deepest storage sovereignty feature. Less visible to end users than identity or agent integrations, but it carries high credibility in the infrastructure community. "Your garden lives on IPFS" is a meaningful statement.

**Avoid Filecoin dependency** — IPFS the protocol is clean and stable. Filecoin incentive dynamics are complex. This feature uses IPFS for export/backup, not Filecoin for persistence. Users who want pinning use a pinning service (Pinata, Web3.Storage, or self-hosted).

---

## What Needs to Be Built

1. **Garden serialisation to CAR format** — package a Room (or full garden) as a Content Addressable Archive (CAR file). Each Locus = one IPFS block. Room manifest = root CID.
2. **IPFS export Tauri command** — `export_to_ipfs(room_id?: string)` — serialises selected Room(s) or full garden, adds to local IPFS node (if running) or returns CAR bytes for upload.
3. **Local IPFS node detection** — check for local Kubo node at `localhost:5001`. If present, use it directly. If absent: export CAR file for manual upload or pinning service.
4. **Pinning service integration** (optional, v1) — API key input for Pinata or Web3.Storage. One-click pin after export.
5. **CID display** — show root CID after export. Copy button. IPFS gateway link (ipfs.io or user-configured gateway).
6. **"Export to IPFS" action** — in Room context menu and Settings → Backup.

---

## Technical Entry Points

```
desktop/src-tauri/src/ipfs.rs        ← create: CAR serialisation, Kubo API client
desktop/src-tauri/Cargo.toml         ← add: iroh or ipld crates for CAR building
desktop/src-tauri/src/main.rs        ← expose export_to_ipfs() Tauri command
packages/core/src/types.ts           ← extend Room/Locus with ipfsCid?
```

**CAR structure:**
```
root CID (Room manifest)
├── locus/locus-2026-05-09-sovereignty.md  (CID)
├── locus/locus-2026-04-22-mixnet.md       (CID)
└── manifest.json                          (room metadata, list of CIDs)
```

**Kubo API call:**
```rust
// POST to localhost:5001/api/v0/dag/import
// Body: CAR bytes
// Returns: { "Root": { "/": "bafy..." } }
```

---

## Dependencies
None. Fully independent.

---

## Cipher's Gate
- **No Filecoin dependency.** Export is IPFS-only. Pinning is optional and user-initiated.
- **CID displayed to user** — they own the content address. Loci never holds the only copy.
- **Pinning service API keys** stored in OS keychain (same pattern as other credentials). Never in `~/.loci/config.json`.
- **Content is public once pinned.** The export UI must clearly communicate: "Content pinned to IPFS is publicly accessible by anyone with the CID." Explicit consent step before pinning.

---

## Acceptance Criteria
- [ ] "Export to IPFS" action in Room context menu
- [ ] Garden serialised as valid CAR file with correct CIDs
- [ ] If local Kubo running: adds to local node, returns CID
- [ ] If no local Kubo: saves CAR file for manual upload
- [ ] Root CID displayed with copy button and gateway link
- [ ] Public-content warning shown before pinning
- [ ] Pinata/Web3.Storage optional integration with OS keychain credential storage

---

## Changelog
- 2026-05-09: Brief created.

---

## First Move
> `git checkout -b feat/2E-ipfs` → implement garden-to-CAR serialisation in Rust using `iroh` or `car` crate → test CID generation with a sample Locus → expose `export_to_ipfs` Tauri command → test end-to-end with local Kubo node
