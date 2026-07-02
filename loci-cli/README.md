# loci-cli

A small terminal-native door into your local palace.

```
$ loci status
palace : /Users/you/palace
layout : rooms-at-root
rooms  : 4
         engine-room              (18 crystals)
         great-hall               (37 crystals)
         observatory              (12 crystals)
         soul                     (9 crystals)
total  : 142 crystals
```

## What it is

`loci` is the read-shaped CLI for a loci palace: a small Rust binary that walks
the palace structure (PALACE.md / CLAUDE.md / rooms / crystals) and prints what
it finds. Read-only. No network, no inference, no daemons. Your AI does the
thinking; the CLI just shows the firmware.

## Install

```bash
cargo install --path loci-cli
```

A pre-built binary will follow once release tagging stabilises.

## Layouts it accepts

Two shapes, matched by the desktop app:

- **rooms-inside-`_palace`/**: the original loci layout
- **rooms-at-root**: palaces ported from older organic structures (rooms grew
  at root, never moved into `_palace/`)

Either works.

## Palace resolution

Each command resolves the palace in this order:

1. `--palace <path>` (explicit override)
2. `$LOCI_PALACE` (environment variable)
3. Walk up from the current directory until a palace marker is found

## Commands

| Command | What it does |
|---|---|
| `loci status` | Palace path, layout, room and crystal counts |
| `loci crystals` | List every crystal slug across all rooms |
| `loci crystals --room <name>` | Restrict to one room |
| `loci read <slug>` | Print a crystal's contents |
| `loci read <slug> --room <name>` | Disambiguate when the same slug lives in two rooms |
| `loci handover` | Print the most recent handover by mtime |
| `loci init` | Interactive wizard. Writes `~/.config/loci/config.toml` |

Add `--json` to any read command for machine-readable output.

## Exit codes

| Code | Meaning |
|---|---|
| `0` | Clean |
| `1` | I/O or unexpected error |
| `2` | Palace or crystal not found |
| `3` | Bad input (ambiguous slug, malformed args) |

## What's not here

This is the safest sure-needed slice. The following live behind future
releases:

- One-shot inference (`-z "<prompt>"`)
- Non-local AI backends (Anthropic, OpenAI)
- Chat TUI (`loci chat`)
- MCP server (`loci serve`)
- Sync / reindex (`loci sync`)
- Full diagnostic feedback flow

The CLI's job at this tier is to be the door into your palace from the
terminal, not to host the conversation. The companion app is one door; the
templates kit is another; this is the third.

## License

MIT. See the workspace `LICENSE`.
