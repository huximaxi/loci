# loci CLI contract

What a script may rely on. This page and the tests beside it (`tests/contract.rs`, and the
`contract_tests` module in `src/main.rs`) are the contract; the README teaches, this page promises.
If the two disagree, this page wins and the README is the bug.

From 2.0 the commands, flags, exit codes and `--json` keys below are frozen until 3.0. A release may
**add** a command, a flag, an optional argument or a `--json` key. It may not remove or rename one,
or change what one means. Experimental commands (`audit`, `wal verify`) are marked, and the freeze
covers their shape only from the release that drops the mark.

## Resolving the workfolder

Commands that read a workfolder find it in this order: `--palace <path>`, `$LOCI_PALACE`, then a
walk up from the current directory to the first directory holding `PALACE.md` or `CLAUDE.md`. None
found is exit 2. (`--palace` and `$LOCI_PALACE` keep these names; the workfolder spellings arrive as
aliases and the old names stay.)

## Exit codes

| Code | Meaning |
|---|---|
| `0` | Success. `--help` and `--version` are success. |
| `1` | I/O or unexpected error (unreadable file, malformed JSON input). |
| `2` | Not found: no workfolder, no such crystal, no handover, no room. Nothing else. |
| `3` | Bad input: a usage error (unknown flag or command, missing or malformed argument), an ambiguous slug, a removed flag (`rain --fire`), refusing to overwrite. |

A usage error is 3, never 2. (Before 2.0 clap's own exit 2 made a mistyped flag look like a missing
workfolder.)

## Output streams

- A command's result goes to **stdout**. With `--json` it is one JSON document and nothing else.
- Every error goes to **stderr** and leaves stdout empty. A runtime error is one line,
  `loci: <message>`. A usage error is clap's text (`error: ...` and a usage line).
- `--help` and `--version` go to **stdout**.
- `--json` never changes an exit code and never moves an error to stdout.

## Global flags

| Flag | Meaning |
|---|---|
| `--palace <path>` | Workfolder path. Overrides `$LOCI_PALACE` and the walk up. |
| `--json` | Machine-readable output, one document on stdout. Accepted by every command; `init` ignores it (interactive). |
| `--help`, `-h` | Help, exit 0. |
| `--version`, `-V` | `loci <version>`, exit 0. |

## Commands

Each section lists the arguments, the exit codes the command can return beyond `0` and `1`, and the
top-level keys of `--json`. A key listed here is always present. Extra keys may appear in a later
release. The tests compare the key sets exactly, so adding one is a change to this page too.

### `loci status`

Layout, rooms and crystal counts by tier. Exit 2: no workfolder.
`--json` keys: `palace`, `layout`, `rooms`, `crystal_total`, `files_total`, `tiered`, `documents`,
`legacy_lines_not_counted`. `crystal_total` and each room's `crystals` are `.md` file counts (their
1.x meaning); `tiered` is the format 2 count.

### `loci crystals`

List crystals. Argument: `--room <name>`. Exit 2: no workfolder, no such room.
`--json` keys: `palace`, `rooms`, `unrooted`. Each room has `room`, `crystals` (slugs) and `entries`.

### `loci read`

Print one crystal's file. Arguments: `<slug>` (case-insensitive), `--room <name>`.
Exit 2: no workfolder or no such crystal. Exit 3: the slug is in several rooms and no `--room`.
`--json` keys: `slug`, `path`, `content`.

### `loci handover`

Print the newest handover by modification time. Exit 2: no workfolder, no handover.
`--json` keys: `path`, `content`.

### `loci tokens`

Approximate state of the agent runtime's 5-hour window, read from local transcripts. Needs no
workfolder.
`--json` keys: `state`, `signal`, `tokens`, `messages`, `files_scanned`, `window_start_utc`,
`window_reset_utc`, `minutes_remaining`, `note`.

### `loci reads`

How often the agent read the workfolder, counts only. Argument: `--days <n>` (default 7; a value outside 1 to 90 is clamped, not refused).
Exit 2: no workfolder. Exit 3: `--days` is not a number.
`--json` keys: `root`, `days`, `sessions`, `sessions_with_read`, `files_scanned`, `user_turns`,
`turns_with_read`, `turns_with_touch`, `palace_reads`, `palace_writes`, `delegated_reads`,
`delegated_writes`, `shell_touches`, `read_rate_pct`, `touch_rate_pct`, `note`.

### `loci rain`

Garden watering weather. It starts nothing: the person starts the round in their own agent.
Exit 2: no workfolder. Exit 3: `--fire` (removed in 2.0).
`--json` keys: `window`, `garden`, `start_phrase`.

### `loci new`

Scaffold a starter workfolder. Arguments: `<path>`, `--force`. Writes files; exit 3 when the target is
not empty and `--force` is absent.
`--json` keys: `palace`, `created`.

### `loci init`

Interactive setup wizard; writes `~/.config/loci/config.toml`. Takes no `--json`. Not for scripts.

### `loci audit`

**Experimental.** Egress receipt read from the local log. Arguments: `--wal <path>`, `--since
<ISO-8601 UTC>`. An empty receipt is not evidence that nothing left the device.
`--json` keys: `wal`, `since`, `frames`, `classes`, `chain_ok`, `chain_break_seq`, `degraded`,
`degraded_latest`, `disabled_at`, `unreadable_lines`, `experimental`, `note`.

### `loci wal verify`

**Experimental.** Verify an exported proof bundle offline. Arguments: `<bundle>`, `--expect-key <hex>`.
Exit 3: bundle over the size cap. Exit 1: unreadable or malformed bundle.
`--json` keys: `bundle`, `result` (`pass` or `fail`), `signer_pubkey`, `range`, `frames`, `detail`.

### `loci overview`

Every command, grouped, with one example each. Alias: `loci commands`. Works with no workfolder.
`--json` keys: `tool`, `version`, `palace`, `groups`, `doors`.

## What the tests hold

- Every command and subcommand in the CLI has a row in the README table and a section above.
- Every command that takes `--json` produces one parseable document with exactly the keys above,
  run against the format 2 fixture workfolder with an empty home directory.
- The exit code of each usage error, each not-found and each bad-input case above, and that errors
  leave stdout empty and `--help` and `--version` print to stdout.
- `init` and `wal verify` are not run with `--json` in the fixture test: `init` is interactive and
  `wal verify` needs a signed bundle (the `loci-wal` tests cover bundles).
