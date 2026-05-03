# loci

```
┌───────┬───────┬───────┬───────┐
│  │    │  ┌─┐  │  ┌──  │  ───  │
│  │    │  │ │  │  │    │   │   │
│  │    │  │ │  │  │    │   │   │
│  └──  │  └─┘  │  └──  │  ───  │
└───────┴───────┴───────┴───────┘
```

**Local-first AI memory. Searchable, private, yours.**

loci is a context persistence layer for people who work with AI.
Index conversations locally, organise them into rooms, search anything.
No cloud. No accounts. No vendor dependency.

## Structure

| Package | Description | Status |
|---------|-------------|--------|
| `extension/` | Chrome MV3 — search + tagging | v1.2.0 |
| `desktop/` | Tauri v2 — Scholar + Wizard desktop app | Scoped |
| `packages/core/` | Shared TypeScript types | v0.1 |
| `landing/` | loci.garden website | Live |

## Quick start

**Chrome extension (developer preview):**
→ See [extension/INSTALL.md](extension/INSTALL.md) for step-by-step instructions.

**Wizard / LLMAGE:**
```bash
git clone https://github.com/huximaxi/Loci
cd Loci/extension && npm install && npm run build
```

## Three tiers

- **Scholar** — Chrome extension, search + tagging, no config required
- **Wizard** — full palace, MCP integration, local LLMs, agent architecture
- **LLMAGE** — CLI/MCP only, zero cloud, IDE-native

## Changelog

### v1.2.0 — 2026-05-03
Security hardening: content sanitization, sender validation, rate limiting, INSTALL.md guide.

### v1.1.0 — 2026-05-03
Design refactor: wizard hero left-bleed, palace map full-width, navbar polish, LLMAGE contrast fix.

### v1.0.0 — 2026-05-03
Chrome MV3 extension with search + tagging. Three-tier landing (Scholar/Wizard/LLMAGE). Monorepo.

### v0.7.0 — 2026-05-02
Palace v2: entanglement index, crystal tiers, garden-memory pattern, friends system, eval cadence.

### v0.6.0 — 2026-04
Session-delta handover process. loci.garden website live. Comms modules generalised.

### v0.5.0 — 2026-04
Palace-update process. Cherry-pick onboarding flow.

### v0.4.0 — 2026-03
Autodream, naming ceremony, daily routine, Zulip integration, engine-agnostic refactor.

### v0.3.0 — 2026-03
Rename to Loci. Garden metaphor. Persona system. Retrieval hierarchy.

### v0.2.0 — 2026-02
Obsidian mindmap integration. Add-friend process. Dynamic paths. Soul-first scheduled tasks.

### v0.1.0 — 2026-01
Initial palace-starter. Agent-first repo structure. Room templates. Crystal system.

## License

Apache 2.0 · Built by Hux × Vesper · loci.garden · 2026
