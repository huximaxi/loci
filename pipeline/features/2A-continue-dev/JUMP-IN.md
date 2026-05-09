# 2A · Continue.dev Context Provider
## Jump-In Brief

**Persona:** Kata — trench-worker, low-cost high-value
**Tier:** 2 · **Target:** Q4 2026 (spec Q3, near-free after 1B)
**Status:** `⏸ blocked on 1B`
**Branch:** `feat/2A-continue`
**Last updated:** 2026-05-09

---

## Context
Continue.dev is an open-source AI coding assistant (VS Code + JetBrains) that supports "context providers" — pluggable sources of grounding knowledge. Once Loci ships an MCP server (1B), registering as a Continue context provider is nearly free: Continue supports MCP context providers directly. A developer in their editor asks a question; Continue pulls relevant Loci nodes as context.

This is one of the highest leverage integrations per engineering hour. It places Loci inside the daily workflow of sovereignty-aligned developers without requiring them to change how they work.

---

## Current State
MCP server (1B) not yet shipped. Once it is, the Continue integration is a documentation task + one config schema. No new backend code required.

---

## What Needs to Be Built

1. **Verify MCP context provider support** in Continue.dev — confirm current version supports MCP as a context provider source (expected: yes, via `contextProviders` config).
2. **Write `.continue/config.json` snippet** — the exact config a user pastes to connect their Loci MCP server:
   ```json
   {
     "contextProviders": [
       {
         "name": "mcp",
         "params": {
           "serverUrl": "http://localhost:3456",
           "resources": ["loci://search", "loci://room"]
         }
       }
     ]
   }
   ```
3. **Integration guide** — `pipeline/features/2A-continue-dev/INTEGRATION-GUIDE.md`. Published to Loci docs site.
4. **Test matrix** — verify with VS Code + JetBrains, Continue.dev version pinned.

---

## Technical Entry Points
```
1B MCP server must be live at localhost:3456
Continue.dev config docs: continue.dev/docs/customization/context-providers
```

---

## Dependencies
- **1B (Goose MCP Plugin)** must ship first — this is a documentation + config task on top of the MCP server.

---

## Cipher's Gate
- The MCP server port (3456) is already localhost-only (from 1B brief). No new attack surface.
- Continue.dev has telemetry opt-out — document this for sovereignty-conscious users.

---

## Acceptance Criteria
- [ ] Loci registers as a Continue.dev context provider via MCP
- [ ] Works in both VS Code and JetBrains
- [ ] Integration guide published
- [ ] `.continue/config.json` snippet tested and correct
- [ ] Telemetry opt-out documented

---

## Changelog
- 2026-05-09: Brief created. Blocked on 1B.

---

## First Move
> After 1B ships: verify Continue MCP context provider API → write config snippet → test in VS Code → write integration guide
