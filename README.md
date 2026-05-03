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
          +---------------+---------------+
          |               |               |
     [ DEV ROOM ]   [ GREAT HALL ]  [ DESIGN ROOM ]
      terminal blue  amber lantern   candlelit warm
          |               |               |
          +-------+--------+-------+-------+
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
| `extension/` | Chrome MV3 — search + tagging | v0.1 — build ready |
| `desktop/` | Tauri v2 — Scholar + Wizard desktop app | Scoped |
| `docs/` | VitePress documentation | In progress |
| `packages/core/` | Shared TypeScript types + utilities | v0.1 |
| `landing/` | loci.garden website | Live |

## Quick start

### Chrome extension (development)
```bash
cd extension
npm install
npm run build
# Load dist/ as unpacked extension in Chrome
```

### Docs site
```bash
cd ../loci-docs
npm install
npm run docs:dev
```

## Three tiers

- **Scholar** — search + tagging, browser extension, no AI config required
- **Wizard** — full palace, MCP integration, agent architecture, local LLMs
- **LLMAGE** — CLI/MCP only, zero cloud, IDE-native

## Architecture

→ [Full architecture docs](https://docs.loci.garden)
→ [Security analysis](../loci-cipher-security-analysis.md)

## Changelog

### v0.1.0 — 2026-05-03
- Chrome MV3 extension: search + tagging for Claude.ai + ChatGPT
- Three-tier landing page (Scholar / Wizard / LLMAGE theme switcher)
- 9 wizard feature cards with RPG tone and group dividers
- Resources nav dropdown + contact modal
- Monorepo structure: extension, packages/core, landing, docs
- Cipher security analysis: 7 threats documented, pre-CWS checklist
- VitePress docs skeleton
- loci.garden deployed: SEO, OG tags, sitemap, robots.txt, llms.txt updated
- Build: 0 TypeScript errors, zero network calls in bundle

### v0.0.1 — 2026 (prior)
- loci methodology: palace, rooms, crystals, garden, soul files
- loci.garden live at 1984 Hosting, Iceland
- llms.txt + llms-full.txt context declarations
- Original wizard landing with hero image

## License

Apache 2.0 · Built by Hux × Vesper · loci.garden · 2026
