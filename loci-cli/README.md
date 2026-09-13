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

Prerequisite: a stable Rust toolchain via [rustup](https://rustup.rs) (one
command: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`).
Then, from the repository root:

```bash
cargo install --path loci-cli
```

A pre-built binary will follow once release tagging stabilises.

## Layouts it accepts

Three shapes:

- **rooms-inside-`_palace`/**: the original loci layout
- **rooms-inside-`rooms/`**: the shape the templates kit and the setup guides
  build (`rooms/<room>/CLAUDE.md`)
- **rooms-at-root**: palaces ported from older organic structures (rooms grew
  at root, never moved into `_palace/`)

Any of the three works. The root must hold a `PALACE.md` or `CLAUDE.md`.

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
| `loci audit` | Egress receipt: what left the device, grouped by class, with a hash-chain check. `--wal <path>` and `--since <ISO-8601>` narrow it |
| `loci wal verify <bundle>` | Verify an exported proof bundle offline. `--expect-key <hex>` pins the signer for provenance |

Add `--json` to any read command for machine-readable output.

## The egress receipt

`~/.loci/wal/egress.jsonl` is an append-only write-ahead log of what left the
device: one frame per outbound call, recording that a call went out, to which
host, and under which egress class (`local`, `external_cloud`,
`channel_egress`, `profile_write`). It is payload-free by construction: the
prompt bytes are never kept, and the optional content hash and byte count stay
in the local log and never travel in an exported bundle. Frames are
hash-chained, so an interior edit, a reorder, or a dropped frame breaks the
chain; `loci audit` reports the break by sequence number. The chain alone is
not keyed tamper-evidence (a tail edit or a wholesale re-chain is not caught
without a key); for cryptographic proof, export a signed bundle and check it
with `loci wal verify`. The primitive lives in the `loci-wal` crate; see its
crate docs for the exact guarantees.

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
