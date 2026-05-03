# loci

```
┌───────┬───────┬───────┬───────┐
│       │  ┌─┐  │  ┌──  │  ───  │
│  │    │  │ │  │  │    │   │   │
│  │    │  │ │  │  │    │   │   │
│  └──  │  └─┘  │  └──  │  ───  │
└───────┴───────┴───────┴───────┘
```

**Local-first AI memory. Searchable, private, yours.**

```
              [ THE GARDEN ]
                    |
    +───────────────+───────────────+
    |               |               |
[ DEV ROOM ]  [ GREAT HALL ]  [ DESIGN ROOM ]
  terminal     amber lantern    candlelit
    |               |               |
    +───────+────────+────────+──────+
            |                 |
      [ RESEARCH ]      [ HATCHERY ]
       forest green       earthy amber
```

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
- Content sanitization in extension (THREAT-01 mitigation)
- Message sender validation (THREAT-04)
- Search rate limiting
- INSTALL.md for regular users
- Security notes in README

### v1.0.0 — 2026-05-03
- Chrome MV3 extension: search + tagging for Claude.ai + ChatGPT
- Three-tier landing page (Scholar / Wizard / LLMAGE)
- 9 wizard feature cards with RPG tone
- Monorepo structure
- VitePress docs skeleton

### v0.0.1 — 2026 (prior)
- loci methodology: palace, rooms, crystals, garden, soul files
- loci.garden live, llms.txt, original wizard landing

## License

Apache 2.0 · Built by Hux × Vesper · loci.garden · 2026
