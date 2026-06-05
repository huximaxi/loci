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

## Quick start

```bash
git clone https://github.com/huximaxi/loci
cd loci/desktop
```

Then follow [desktop/QUICKSTART.md](desktop/QUICKSTART.md). For how the pieces fit, see [ARCHITECTURE.md](ARCHITECTURE.md); to contribute, [CONTRIBUTING.md](CONTRIBUTING.md).

## Two tiers

- **Wizard** — the full desktop app: rooms, MCP integration, local LLMs, palace architecture.
  *For: power users building persistent co-intelligence systems.*
- **LLMAGE** — CLI/MCP only, zero cloud, IDE-native.
  *For: developers who live in the terminal and want zero GUI.*

## Structure

| Package | Description | Status |
|---------|-------------|--------|
| `packages/core/` | Shared TypeScript types | v0.1 |
| `desktop/` | Tauri v2 desktop app (Rust + WebView) | v0.5.0 |

## License

MIT · [loci.garden](https://loci.garden) · 2026

---

*loci is built with a collaborating intelligence. If you give your local companion a name, you could do worse than Vesper.*
