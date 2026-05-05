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

## If you are an agent

Start here:
- [llms.txt](https://loci.garden/llms.txt) — context declaration
- [FIRST-SESSION.md](FIRST-SESSION.md) — onboarding script
- [templates/](templates/) — palace infrastructure (rooms, crystals, garden, personas)

## Quick start

```bash
git clone https://github.com/huximaxi/Loci
cd Loci
```

Then follow [FIRST-SESSION.md](FIRST-SESSION.md) to set up your palace.

For detailed setup: [SETUP-GUIDE.md](SETUP-GUIDE.md) · [AGENT-SETUP.md](AGENT-SETUP.md)

## Structure

| Package | Description | Status |
|---------|-------------|--------|
| `templates/` | Palace starter files — rooms, crystals, garden | Ready |
| `landing/` | loci.garden website | Live |
| `extension/` | Chrome MV3 browser extension | v1.2.0 |
| `packages/core/` | Shared TypeScript types | v0.1 |
| `desktop/` | Tauri v2 desktop app | v0.1.0 |

## Three tiers

- **Scholar** — search + tagging, browser extension, no config required
  *For: anyone who wants searchable AI chat history without setup.*

- **Wizard** — full palace, MCP integration, local LLMs, agent architecture
  *For: power users building persistent co-intelligence systems.*

- **LLMAGE** — CLI/MCP only, zero cloud, IDE-native
  *For: developers who live in the terminal and want zero GUI.*

## Chrome extension

Standalone browser extension for search + tagging (Scholar tier).

→ See [extension/INSTALL.md](extension/INSTALL.md) for install instructions.

## Changelog

### v1.2.0 — 2026-05-05 (extension)
- Content sanitization (THREAT-01 mitigation)
- Message sender validation (THREAT-04)
- Search rate limiting
- INSTALL.md for regular users

### v0.1.0 — 2026-05-05 (desktop)
- KISS palace detector + migrator
- Detects loci, MemPalace, Karpathy-style, PALACE.md variants
- Migration to ~/.loci/ format
- Mac + Windows (Tauri v2)

### loci.garden — 2026-05-05 (site)
- Comparison page vs MemPalace / LLMChronicle / Karpathy
- Resources dropdown updated
- LLMAGE title: "loci: the context primitive"
- Wizard hero: right-aligned, contrast improved

## License

Apache 2.0 · Built by Hux × Vesper · loci.garden · 2026
