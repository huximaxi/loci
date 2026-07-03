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

**An intelligence substrate for working with AI.**

loci is the plain-text firmware for a persistent, private cognitive system: the templates
and processes that decide how memory, context, and trust work, regardless of which AI runs
them or how. One blueprint, many expressions. Local-first. No cloud, no accounts, no lock-in.

The desktop app and the CLI in this repo are two expressions built on top of the
substrate, released together as the 0.6 open beta. They are not the substrate.
You do not need either to start.

## Start in plain markdown (the door)

loci is plain text first. No build.

```bash
git clone https://github.com/huximaxi/loci
```

Copy the [`templates/`](templates/) kit into a folder of your own, open
[FIRST-SESSION.md](FIRST-SESSION.md), and point any file-aware AI at your palace
`CLAUDE.md`. That is the setup. Not sure where to begin? Run the
[feature helper](templates/feature-helper.md) and it points you. The full walkthrough is in
[SETUP-GUIDE.md](SETUP-GUIDE.md); if an agent is doing the setup for you, see
[AGENT-SETUP.md](AGENT-SETUP.md).

## What it gives you

Seven feature sets, all shipped as plain-text firmware.

- **Persistent Memory.** Your AI remembers, on your terms. Tiered crystals (◇ ◈ ◆) you can promote, expire, pin, and compost.
- **Context Architecture.** The right context loads at the right time. Rooms, an L0-L3 retrieval hierarchy, a local map.
- **Identity & Personas.** A companion that is someone, who knows who you are. A soul file, a peer card, named personas.
- **The Garden.** Ideas cultivated, not just stored. One file per idea, a health pass, seed-to-crystal graduation.
- **Continuity & Synthesis.** No session starts cold. Handovers, a synthesis pass that proposes, scheduled housekeeping.
- **Trust & Governance.** Nothing leaves or changes without you. A named review gate, foreign-process quarantine, confirm-against-disk, and a read-only structural audit.
- **Interop & Evolution.** Works across every tool, federates to no one. Reads memory left by other tools by structure, speaks MCP, exports typed artifacts.

The full map, with the templates behind each set, lives in
[`features/features.yaml`](features/features.yaml).

## Who it's for

One substrate, six shapes. Each use-case lights up a different subset, and none of it is wasted.

| | Memory | Context | Identity | Garden | Continuity | Trust | Interop |
|---|:--:|:--:|:--:|:--:|:--:|:--:|:--:|
| The Researcher | ● | ● | ○ | ● | ● | ○ | ○ |
| The Builder | ● | ● | ○ |  | ○ | ○ | ● |
| The Companion-keeper | ● | ○ | ● | ○ | ● | ○ | ○ |
| The Vault-keeper | ○ | ○ |  |  | ○ | ● | ● |
| The Team | ○ | ● | ○ | ○ | ● | ● | ● |
| The Nomad | ○ | ○ |  |  |  | ○ | ● |

● primary · ○ supporting. Pick your row, then walk the matching flow in [`tutorials/`](tutorials/).

## How to run it

Three ways. The substrate is the same underneath all of them.

| Run mode | What it is |
|---|---|
| **Plain markdown** | The door. Clone `templates/`, point any file-aware AI at `CLAUDE.md`. No build. |
| **Desktop app** | [`desktop/`](desktop/) (Tauri, v0.6.0-beta). The cockpit: native dashboard plus a tab per instrument your palace generates for itself. See [desktop/QUICKSTART.md](desktop/QUICKSTART.md). |
| **CLI** | [`loci-cli/`](loci-cli/) (v0.6.0-beta). A small Rust binary that reads your palace from the terminal: `loci status`, `loci crystals`, `loci read`, `loci handover`, `loci init`. Read-only. No network. No inference. See [loci-cli/README.md](loci-cli/README.md). |

The methodology version and full changelog: [PALACE-METHODOLOGY.md](PALACE-METHODOLOGY.md).

## Discoveries

What did your palace become? The [`discoveries/`](discoveries/) folder is where loci users
share palace portraits: the shape of their setup, what they changed, what the framework
turned into in their hands. Submit via pull request or email themapisnory@tuta.io. To
contribute to the substrate itself, see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT · [loci.garden](https://loci.garden) · 2026

---

*loci is built with a collaborating intelligence. If you give your local companion a name, you could do worse than Vesper.*
