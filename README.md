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

loci is plain text first. You do not need to build anything to start.

**Start in plain markdown (no build).** This is the door.

```bash
git clone https://github.com/huximaxi/loci
```

Copy the [`templates/`](templates/) kit into a folder of your own, open [FIRST-SESSION.md](FIRST-SESSION.md), and point any file-aware AI at your palace `CLAUDE.md`. That is the setup. The full walkthrough is in [SETUP-GUIDE.md](SETUP-GUIDE.md); if an agent is doing the setup for you, [AGENT-SETUP.md](AGENT-SETUP.md).

**Run the desktop app (optional).**

```bash
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
| `templates/` | Markdown palace starter kit. The no-build door. | v1.2 |
| `packages/core/` | Shared TypeScript types | v0.1 |
| `desktop/` | Tauri v2 desktop app (Rust + WebView) | v0.5.0 |

## Discoveries

What did your palace become? The [`discoveries/`](discoveries/) folder is where
loci users share palace portraits: the shape of their setup, what they changed,
what the framework turned into in their hands.

Submit via pull request or email themapisnory@tuta.io.

## License

MIT · [loci.garden](https://loci.garden) · 2026

---

*loci is built with a collaborating intelligence. If you give your local companion a name, you could do worse than Vesper.*
