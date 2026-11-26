# loci-cli

A small terminal-native door into your local workfolder.

```
$ loci status
workfolder : /Users/you/workfolder
layout : rooms-at-root
rooms  : 4
         engine-room              (18 crystals: 6 ◆ · 8 ◈ · 4 ◇)
         great-hall               (37 crystals: 20 ◆ · 12 ◈ · 5 ◇ · 2 retired)
         observatory              (12 crystals: 3 ◆ · 6 ◈ · 3 ◇)
         soul                     (9 crystals: 9 ◆)
total  : 76 crystals (38 ◆ · 26 ◈ · 12 ◇ · 2 retired)
files  : 142 .md files, 66 not crystals
```

A crystal is a tiered line (`◆ Name: fact`) or a file with `type: crystal`, as
[`templates/format-2.md`](../templates/format-2.md) defines it. Other `.md` files are documents:
counted as files, never as crystals.

## What it is

`loci` is the read-shaped CLI for a loci workfolder: a small Rust binary that walks
the workfolder structure (PALACE.md / CLAUDE.md / rooms / crystals) and prints what
it finds. Most commands only read. Two write: `loci new` lays down a workfolder and
`loci init` writes `~/.config/loci/config.toml`. The CLI never starts an agent. No
network, no inference, no daemons. Your AI does the thinking; the CLI shows the files.

## Install

Prerequisite: a stable Rust toolchain via [rustup](https://rustup.rs) (one
command: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`).
Then, from the repository root:

```bash
cargo install --path loci-cli
```

## Layouts it accepts

Three shapes:

- **rooms-inside-`_palace`/**: the original loci layout
- **rooms-inside-`rooms/`**: the shape the templates kit and the setup guides
  build (`rooms/<room>/CLAUDE.md`)
- **rooms-at-root**: workfolders ported from older organic structures (rooms grew
  at root, never moved into `_palace/`)

Any of the three works. The root must hold a `WORKFOLDER.md`, `PALACE.md` or `CLAUDE.md`, and the
three are read the same way. `PALACE.md` and `CLAUDE.md` stay readable until 3.0 at least; neither prints a warning. Migrate by renaming,
not copying: a crystal line in two marker files is read twice.

## Workfolder resolution

Each command resolves the workfolder in this order:

1. `--workfolder <path>` (explicit override; `--palace` is the same flag)
2. `$LOCI_WORKFOLDER`, then `$LOCI_PALACE` (environment variables; the first one set and non-empty is used)
3. Walk up from the current directory until a workfolder marker is found

The old spellings (`--palace`, `$LOCI_PALACE`) stay readable until 3.0 at least, with no warning. Tools
that set only `LOCI_PALACE` need no change. If both variables are set, `LOCI_WORKFOLDER` wins and
nothing is printed about the other, so unset it (or use `--workfolder`) for a per-run override.

## Commands

| Command | What it does |
|---|---|
| `loci status` | Workfolder path, layout, and crystal counts by room and tier, with the `.md` file count beside them |
| `loci crystals` | List every crystal across all rooms, with its tier |
| `loci crystals --room <name>` | Restrict to one room |
| `loci read <slug>` | Print a crystal's contents |
| `loci read <slug> --room <name>` | Disambiguate when the same slug lives in two rooms |
| `loci handover` | Print the most recent handover by mtime |
| `loci check` | Ask the workfolder whether it still agrees with itself: findings as `path:line`, and how many rules ran and crystals were read. Read-only; exit 4 when a rule found a `fail`, and a `warn` is printed with exit 0. `--scope room\|workfolder` and `--as-of <YYYY-MM-DD>` |
| `loci tokens` | Approximate status of your agent runtime's 5-hour session window, read from local transcripts (timing, not quota) |
| `loci reads` | How often your agent read the workfolder: user turns followed by a read, counts only, from local transcripts. `--days <n>` (default 7) |
| `loci recall <query>` | Ask the files a question, no model, no network (experimental): BM25 ranked hits as `path:line`, each with its crystal state (current, due, superseded, retired). `--root <dir>` reads any folder of notes; `--limit`, `--room`, `--as-of` narrow it |
| `loci rain` | Garden watering weather: window headroom, garden state, last rain. Start the round yourself, in your own agent |
| `loci new <path>` | Scaffold a starter workfolder at `<path>`, then offer `loci init`. `--force` writes into a non-empty directory |
| `loci init` | Interactive wizard. Writes `~/.config/loci/config.toml` |
| `loci audit` | Egress receipt (experimental): what a runtime recorded as leaving the device, grouped by class, with a hash-chain check. `--wal <path>` and `--since <ISO-8601>` narrow it |
| `loci wal verify <bundle>` | Verify an exported proof bundle offline (experimental). `--expect-key <hex>` pins the signer for provenance |
| `loci overview` | Every command, grouped, with one example each (alias `loci commands`) |

Add `--json` to any read command for machine-readable output.

`status --json` keeps its old keys with their old meaning: `crystal_total` and each room's
`crystals` are still the number of `.md` files. The tiered counts are new keys beside them
(`tiered`, per room and in total; `files_total`; `documents`; `legacy_lines_not_counted`).
`crystals --json` keeps each room's `crystals` slug list and adds `entries`, the crystals as the
spec reads them.

## The egress receipt

**Experimental.** Nothing in loci writes this log yet: the writer left with the
desktop app in v0.7. A runtime that wants receipts calls `record_egress` from
the `loci-wal` crate. Until something does, `loci audit` reads an empty log, and
an empty receipt is not evidence that nothing left the device.

`~/.loci/wal/egress.jsonl` is an append-only write-ahead log of what left the
device: one frame per outbound call, recording that a call went out, to which
host, and under which egress class (`local`, `external_cloud`,
`channel_egress`, `profile_write`). It is payload-free by construction: the
prompt bytes are never kept, and the optional content hash and byte count stay
in the local log and never travel in an exported bundle. Frames are
hash-chained, so an interior edit, a reorder, or a dropped frame breaks the
chain; `loci audit` reports the break by sequence number. The chain alone is
not keyed tamper-evidence (a tail edit or a wholesale re-chain is not caught
without a key); for cryptographic proof, a runtime that holds the signing key exports a signed bundle, and you check it
with `loci wal verify`. The primitive lives in the `loci-wal` crate; see its
crate docs for the exact guarantees.

## Exit codes

| Code | Meaning |
|---|---|
| `0` | Clean |
| `1` | I/O or unexpected error |
| `2` | Workfolder or crystal not found (nothing else) |
| `3` | Bad input: a usage error (unknown flag or command, missing argument), an ambiguous slug |
| `4` | Ran, found problems: the command read your files and its findings are its output (not an error) |

Errors go to stderr and leave stdout empty, with or without `--json`. The full contract, with each
command's flags and `--json` keys, is [`CONTRACT.md`](CONTRACT.md); tests hold it.

## What's not here

This is the safest sure-needed slice. The following live behind future
releases:

- One-shot inference (`-z "<prompt>"`)
- Non-local AI backends (Anthropic, OpenAI)
- Chat TUI (`loci chat`)
- MCP server (`loci serve`)
- Sync / reindex (`loci sync`)
- Full diagnostic feedback flow

The CLI's job at this tier is to be the door into your workfolder from the
terminal, not to host the conversation. The templates kit is one door;
this is the other.

## License

MIT. See the workspace `LICENSE`.
