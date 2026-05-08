# loci

```
◇  ◈  ◆  ◈  ◇  ◈  ◆  ◈  ◇  ◈  ◆  ◈  ◇
┃  ╔═══════╦═══════╦═══════╦═══════╗  ┃
┃  ║  │    ║  ┌─┐  ║  ┌──  ║  ───  ║  ┃
┃  ║  │    ║  │ │  ║  │    ║   │   ║  ┃
┃  ║  │    ║  │ │  ║  │    ║   │   ║  ┃
┃  ║  └──  ║  └─┘  ║  └──  ║  ───  ║  ┃
┃  ╚═══════╩═══════╩═══════╩═══════╝  ┃
◇  ◈  ◆  ◈  ◇  ◈  ◆  ◈  ◇  ◈  ◆  ◈  ◇
```

**Local-first AI memory. Searchable, private, yours.**

loci is a context persistence layer for people who work with AI.
Index conversations locally, organise them into rooms, search anything.
No cloud. No accounts. No vendor dependency.

## If you are an agent

Start here:
- [llms.txt](https://loci.garden/llms.txt) - context declaration
- [FIRST-SESSION.md](FIRST-SESSION.md) - onboarding script (first-time palace setup)
- [PALACE-UPDATE.md](PALACE-UPDATE.md) - update script (existing palace, running `palace-update`)
- [templates/](templates/) - palace infrastructure (rooms, crystals, garden, personas, output primitives)

## Quick start

**New palace:** clone and follow [FIRST-SESSION.md](FIRST-SESSION.md)

```bash
git clone https://github.com/huximaxi/Loci
cd Loci
```

**Existing palace:** → [PALACE-UPDATE.md](PALACE-UPDATE.md) - run `palace-update` to pull new loci-core features into your setup

For detailed setup: [SETUP-GUIDE.md](SETUP-GUIDE.md) · [AGENT-SETUP.md](AGENT-SETUP.md)

## Structure

| Package | Description | Status |
|---------|-------------|--------|
| `templates/` | Palace starter files - rooms, crystals, garden, personas | Ready |
| `landing/` | loci.garden website | Live |
| `extension/` | Chrome MV3 browser extension | v1.2.0 |
| `packages/core/` | Shared TypeScript types | v0.1 |
| `desktop/` | Tauri v2 desktop app | v0.1.0 |
| `LOCI-CORE.md` | Palace methodology version tracker | v1.0 |

## Three tiers

- **Scholar** - search + tagging, browser extension, no config required
  *For: anyone who wants searchable AI chat history without setup.*

- **Wizard** - full palace, MCP integration, local LLMs, agent architecture
  *For: power users building persistent co-intelligence systems.*

- **LLMAGE** - CLI/MCP only, zero cloud, IDE-native
  *For: developers who live in the terminal and want zero GUI.*

## Chrome extension

Standalone browser extension for search + tagging (Scholar tier).

→ See [extension/INSTALL.md](extension/INSTALL.md) for install instructions.

## Changelog

**Latest:** loci-core v1.0 · extension v1.2.0 · desktop v0.1.0 · site 2026-05-05

→ Palace methodology: [LOCI-CORE.md](LOCI-CORE.md)
→ Full history: [CHANGELOG.md](CHANGELOG.md)

## License

Apache 2.0 · Built by Hux × Vesper · loci.garden · 2026
