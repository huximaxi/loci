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

**A local AI topology, in one folder you own.**

loci is a local AI topology: one nested workfolder on your machine, with fixed places your
AI reads and writes like a small local database. It ships as a kit of plain-text templates
any file-aware AI can run. What you settle once stays settled, in files you can read, move
or delete. Local-first. No cloud, no accounts, no lock-in.

The CLI in this repo (v0.7 beta) is an optional door into the workfolder. You do not need it
to start.

## Start in plain markdown (the door)

loci is plain text first. No build.

```bash
git clone https://github.com/huximaxi/loci
```

Copy the [`templates/`](templates/) kit into a folder of your own, open
[FIRST-SESSION.md](FIRST-SESSION.md), and point any file-aware AI at your workfolder's
`PALACE.md`. That is the setup. (Claude Code users also get a two-line `CLAUDE.md`
beside it, which just says to read `PALACE.md` first.) Not sure where to begin? Run the
[feature helper](templates/feature-helper.md) and it points you. The full walkthrough is in
[SETUP-GUIDE.md](SETUP-GUIDE.md); if an agent is doing the setup for you, see
[AGENT-SETUP.md](AGENT-SETUP.md).

## What it gives you

Seven feature sets, all shipped as plain-text templates.

- **Persistent Memory.** Your AI remembers, on your terms. Tiered crystals (◇ ◈ ◆) you can promote, expire, pin, and compost.
- **Context Architecture.** The right context loads at the right time. Rooms, an L0-L3 retrieval hierarchy, a local map.
- **Identity & Personas.** A companion that is someone, who knows who you are. A soul file, a peer card, named personas.
- **The Garden.** Ideas cultivated, not just stored. One file per idea, a health pass, seed-to-crystal graduation.
- **Continuity & Synthesis.** No session starts cold. Handovers, a synthesis pass that proposes, scheduled housekeeping.
- **Trust & Governance.** You decide what changes and what goes out. A named review gate, foreign-process quarantine, confirm-against-disk, and a read-only structural audit. The gate is a rule your AI reads, not a lock. loci itself sends nothing anywhere; the AI you run sees what it reads, so pick a local model when nothing may leave.
- **Interop & Evolution.** Works across every tool, federates to no one. Reads memory left by other tools as data behind a gate, exports typed artifacts.

The full map, with the templates behind each set, lives in
[`features/features.yaml`](features/features.yaml).

## Who it's for

One cognitive substrate, six shapes. Each use-case lights up a different subset, and none of it is wasted.

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

Two ways, and the door is the first. The workfolder is the same underneath both.

| Run mode | What it is |
|---|---|
| **Plain markdown** | The door. Clone `templates/`, point any file-aware AI at `PALACE.md`. No build. |
| **CLI** | [`loci-cli/`](loci-cli/) (v0.7.0-beta, optional). A small Rust binary that reads your workfolder from the terminal: `loci status`, `loci crystals`, `loci read`, `loci handover`, `loci tokens`, `loci rain`, `loci init`. Read-only, with one hand-off: `loci rain --fire` execs your agent runtime. No network. No inference. See [loci-cli/README.md](loci-cli/README.md). |

The methodology version and full changelog: [PALACE-METHODOLOGY.md](PALACE-METHODOLOGY.md).

## Discoveries

What did your workfolder become? The [`discoveries/`](discoveries/) folder is where loci users
share workfolder portraits: the shape of their setup, what they changed, what the framework
turned into in their hands. Submit via pull request or email themapisnory@tuta.io. To
contribute to the cognitive substrate itself, see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT · [loci.garden](https://loci.garden) · 2026

---

*loci is built with a collaborating intelligence. If you give your local companion a name, you could do worse than Vesper.*
