# Changelog

All notable changes to this project are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

---

## [Unreleased]

### Added
- `loci recall <query>` (experimental): ask your files a question with no model and no network. BM25
  over a workfolder or any folder of notes, each hit a `path:line` with its crystal state (current,
  due, superseded, retired, or not a crystal). Reads one root, follows no symlink, skips hidden
  directories, and counts the `.md` files and directories it could not read (too large, not UTF-8, unreadable). The JSON is not frozen until the mark is
  dropped.

### Fixed
- `loci handover` no longer picks a template. A freshly copied `handover-template.md` (or anything under
  a `templates/` directory) used to win on modification time. Files with equal times now resolve to
  the later path, so the answer does not depend on directory order.
- `loci rain` looks for `garden/` at the workfolder root, where the spec and the first-session
  ceremony put it. With the `rooms/` layout it looked inside `rooms/` and always found no plants.
  A garden inside the scan root (`_palace/garden/`) is still read when the root garden holds no
  `plants/` or `.rain/` and the older one does.

### Changed
- A new exit code, `4`: "ran, found problems". It is for commands that read your files and report findings
  (`loci check`, coming in this release). `1` stays "could not run", `2` not found, `3` bad input, so
  a script that treats any non-zero as failure is unaffected. No command returns `4` yet; the code
  and its row in `CONTRACT.md` land first.
- One version line. The CLI, the templates kit and the methodology are all 2.0: the CLI goes from
  0.7.0-beta to `2.0.0-rc.1`, the methodology anchor from 1.6-candidate to 2.0-candidate (`stable`
  stays 1.2; the field is still `loci-core version`), and `package.json` follows the line. The
  numbers jump because the frozen crystal format (`templates/format-2.md`) changes how 1.x files are counted and read; legacy forms still read.
  `loci --version` prints `loci 2.0.0-rc.1 (loci format 2)`. `loci-wal` keeps its own semver while
  `audit` is experimental. `docs/RELEASING.md` and the PR and issue templates no longer name a
  separate extension line.
- The docs, templates and the CLI's own text say "workfolder" where they said "palace". Kept as
  they were, because tools read them: the old spellings (`--palace`, `$LOCI_PALACE`, `PALACE.md`,
  the `Palace:` and `Palace holder:` lines), the `_palace/` folder, the update-check files
  (`PALACE-METHODOLOGY.md`, `PALACE-UPDATE.md`), skill and process names (`palace-audit`,
  `palace-update`), and every `--json` key (`palace`, `palace_reads`, `palace_writes`). On screen,
  `status` and `overview` print `workfolder :` where they printed `palace :`, `reads` prints
  `activity :` for its counts line, and the not-found message reads "no workfolder found". Scripts
  should key on the exit code and on `--json`, as `CONTRACT.md` says; a test now fails if help
  text, overview or an error says "palace" again.
- Usage errors (an unknown flag or command, a missing or malformed argument) now exit 3, printed
  to stderr as before. Exit 2 means only "not found". A script that treated 2 as "usage error"
  should read 3; `--help` and `--version` still exit 0 on stdout.
- `loci status` and `loci crystals` count crystals as format 2 defines them (a tiered line or a
  `type: crystal` file), by tier, with retired, superseded and unscheduled shown apart. The `.md`
  file count stays as its own line. Numbers on screen change for 1.x workfolders: documents are
  no longer counted as crystals, and a count of old-style bullets (`- **Name** = fact`) that are
  read but not counted is printed when there are any. `--json` keeps every old key with its old value (`crystal_total` and a room's
  `crystals` are still file counts) and adds the new ones (`tiered`, `files_total`, `documents`,
  `legacy_lines_not_counted`, and `entries` in `crystals`).
  A human `crystals` line shows one cleaned label (control characters replaced, 120 characters at
  most); `entries[].fact` in `--json` carries the whole fact. The reader opens regular files only
  (a FIFO is never read), reads at most 64 KiB of a file crystal and 8 MiB of a `CLAUDE.md`,
  ignores a leading BOM, and follows symlinks the way room discovery does, reading a file reached
  by two paths once.
- `templates/format-2.md` reserves two more key names, `author` and `promoted_from`, beside
  `type: shard`, `address` and `signed_by`. A writer does not use them for anything else; a reader
  still ignores them like any unknown key, so no reader changes. The two first-session crystals
  in `AGENT-SETUP.md` and `templates/first-session-protocol.md` no longer write an `author:` line.

### Added
- The workfolder has its own spellings, as aliases: `--workfolder` (the old `--palace` is the same
  flag), `$LOCI_WORKFOLDER` (read before `$LOCI_PALACE`), and `WORKFOLDER.md` as a root marker next
  to `PALACE.md` and `CLAUDE.md`, read like them (crystal lines count). The old names keep working
  and print no warning. An environment variable that names no workfolder is still an error, now
  naming the variable it read.
- `loci-cli/CONTRACT.md`: the written CLI contract (commands, flags, exit codes, stdout and stderr
  rules, each command's `--json` keys), frozen from 2.0 until 3.0. Tests hold it: every command has
  a README row and a contract section, every `--json` command emits exactly the listed keys on the
  fixture workfolder, and each exit code is checked against the real binary. `loci reads` was
  missing from the README table and now has its row.
- A format 2 conformance fixture and test: three fixture workfolders (one per layout) under
  `loci-cli/tests/fixtures/format2/`, a hand-written `expected.json`, and a reference reader that
  follows `templates/format-2.md`. The test fails on a deliberately broken fixture or a crystal
  read twice, and rule-by-rule tests hold the parse rules (name split, legacy suffixes, key block,
  fences, dates, case, symlinks). `superseded_by` resolution and the `pinned_until` last-day rule
  are left to the change that counts crystals. Test-only for now: `status` and `crystals` count
  tiered crystals in a later change.
- `templates/format-2.md`: the loci format 2 spec. One crystal shape (`◈ Name: fact` with keys on
  the next lines), the tiers and their aliases, the `valid_until` grammar (a calendar date is
  scheduled, anything else is an unscheduled label), the time keys `occurred`, `mentioned` and
  `superseded_by`, the accepted layouts, and how each legacy form reads. Readers ignore keys and
  `type` values they do not know; `type: shard`, `address` and `signed_by` are reserved. The CLI
  catches up to it across rc.1: `status` and `crystals` still count every `.md` file today.
- `scripts/release-sums.sh`: writes, signs and verifies a `SHA256SUMS` for release artifacts
  with an SSH signing key (namespace `loci-release`), offline. `verify` is strict both ways: the
  signature must cover the sums, the sums must name exactly the artifacts in the folder, and
  every hash must match. Hidden files are skipped; subfolders and unsafe names are refused.
  `selftest` proves each failure, including the signature alone. The procedure is in
  `docs/RELEASING.md` ("Release artifacts"). The public key is published in
  `docs/allowed_signers`.
- `loci reads [--days N]`: does the agent read the palace at all? Counts, from local Claude Code
  transcripts, the turns a person typed that were followed by at least one read of a palace
  file before the next one. Harness entries (task notifications, reminders, interrupt markers,
  slash and bash-mode echoes) are not turns: `origin.kind` decides where a transcript records
  it, known harness shapes where it does not. Two rates, because a shell command cannot say
  whether it read or wrote: the read rate (tool reads only, the floor) and the touch rate
  (reads or shell commands naming the palace, a wider count that still misses commands that
  spell the palace relative, with `~` or `$HOME`, or with escaped spaces). A turn copied into
  a new transcript on resume or fork counts once, and files are read in a fixed order. Also
  palace reads, writes and shell touches, sessions that read the palace, and subagent reads
  and writes kept outside the rate. Prints the measured palace root, so a run from a parent
  folder cannot silently measure the wrong one. Read-only and offline, counts only: no
  transcript text, transcript path or command is printed or kept. Same line bounds and
  symlink rule as `loci tokens`, which now scans a transcript folder once even when
  `CLAUDE_CONFIG_DIR` names the default; `--json` supported.
- `templates/quarantine-fixtures.md` (draft, not yet run end to end against an agent): six
  agent-run regression cases for the quarantine gate, each built from a way memory systems
  fail: a secret's body surviving a clean-up that reports success, foreign text that pretends
  its quote has ended, a token seen in passing reaching the close note, a remote destination
  nobody configured, two hunches consolidated into one fact, and foreign content asking the
  agent to rewrite its own gate. Each case runs on a fresh scratch copy that is cut off from
  the real palace as far as files can say (links followed, git, agent settings and tool
  servers removed, the real path rewritten to the copy's), and a check after every session
  lists any real-palace file changed since setup. Inputs are generated at run time with a
  random canary, every pass condition is checked on disk or in the session's tool log, never
  on the agent's word, and a teardown removes the copy.

### Changed
- Crystal tier names are one set across the kit: Confirmed, Contextual, Exploratory. Working and
  Provisional stay readable as aliases. The guide's inline `valid_until` form is gone, so the key
  has one placement (the line after the crystal). Two guide example crystals were renamed so a
  name no longer reads as a tier label. The master template's CONTEXT CRYSTALS placeholders are
  now glyph-led crystal lines (they were bullets the spec would not count), and the guide's
  trailing `# comment` examples and inline `· pinned` example are gone. `valid_until` is valid on
  any tier (the lifecycle and interop pages said otherwise), and `pinned_until` alone is enough
  to protect a crystal, which the health template now honours.
- CI supply chain: every third-party action in `.github/workflows/` is pinned to a full commit
  SHA (version kept as a comment), the trufflehog scanner image is pinned by digest, and the
  workflows run with read-only tokens (secrets-scan keeps `pull-requests: read`; the gitleaks PR
  comment is turned off). The pins freeze the scanners' rules until bumped.
- loci is held by themapisnory OÜ: the LICENSE copyright line and the README name it. The licence
  stays MIT.
- Claims now match the code. The README no longer says "nothing leaves": loci sends nothing
  itself, but the AI you run sees what it reads, and the gate is a rule your AI follows, not a
  lock. The CLI no longer calls itself read-only: `new` and `init` write (it still makes no
  network calls and runs no inference).
- loci is now defined as a local AI topology: one nested workfolder with fixed places your AI
  reads and writes. README, setup guide, tutorials, the feature helper and `loci --help` say
  "workfolder"; file names, flags, CLI output and `templates/` still say palace.
- The CLI README lists every command (`tokens`, `rain`, `new` and `overview` were missing) and
  no longer promises a pre-built binary; install is `cargo install --path loci-cli`.
- `loci audit` and `loci wal verify` are labelled experimental in `--help`, the audit output
  (text and `--json`: `experimental`, plus a `note` on an empty log) and the CLI README. Nothing
  in loci writes the egress log yet, so an empty receipt now says it is not evidence that nothing
  left the device, instead of "no egress recorded".


### Fixed
- The CLI stops instead of falling back quietly. A `$LOCI_PALACE` that names no palace is now
  an error (exit 2), not a silent walk to whatever palace the current directory holds.
- `loci audit` no longer hides frames behind a bad line. The WAL reader skipped everything after
  the first unparseable line, so a crash mid-write followed by more appends left later frames
  unread while the chain still read ok. Unreadable lines are now skipped and counted, audit
  warns about them (`unreadable_lines` in `--json`), and a lost interior frame fails the chain.
  An `egress.degraded` or `egress.disabled` marker that exists but cannot be read is an error,
  not zero failures.
- The first session appends the identity block to `~/.claude/CLAUDE.md` instead of writing the
  file. The instruction said "write", which an agent could read as replacing a file that other
  projects use. It now reads the file first, appends, and only edits a block with the same heading.

### Removed
- `loci rain --fire`. It ran `claude -p rain` with no timeout, no tool restriction and no
  receipt, which made the CLI an agent launcher. The CLI now starts no agent: `loci rain` shows
  the weather and the person starts the round in their own agent (`templates/skills/rain.md`).
  The flag is kept hidden and answers with a message that says so (exit 3), so an old habit
  does not hit a bare usage error. In `loci rain --json`, the `invocation` field (a runnable
  `claude -p "rain"` string) is replaced by `start_phrase` (`"rain"`), so no field carries a
  command to pipe into a shell. A scheduled `loci rain --fire` now fails with exit 3 and a
  message on stderr only: check any cron or launchd entry that used it.
- `RC-1.6-SCOPE.md`, the v0.6.0-rc.3 scope draft. It still promised commands that never shipped
  (`loci -z`, `sync`, `feedback`) and described the desktop cockpit removed in v0.7.0-beta.
- `loci init` no longer asks for an Ollama endpoint and model. Nothing in the CLI read them,
  and the CLI runs no inference. `config.toml` now records only the palace path. No command reads
  it yet: they resolve the palace from `--palace`, `$LOCI_PALACE` or the current directory.

---

## [v0.7.0-beta] - 2026-11-10

CLI 0.7.0-beta. The desktop app leaves the repository; loci runs as plain markdown plus
the optional CLI. Methodology line unchanged: `loci-core` 1.6-candidate.

### Added
- `templates/okf-interop.md`: the palace speaks the Open Knowledge Format (OKF v0.2), a vendor-neutral spec for markdown with YAML frontmatter. Maps file types, tiers, lifecycle (dated `valid_until` to `stale_after`, composting to `deprecated`) and provenance onto OKF fields while keeping the stability axis (◆ ◈ ◇) and OKF's who-confirmed axis apart; an agent-executable export that stages outside the palace and leaves nothing until the person says so; reading foreign bundles as data behind the gate; a read-only check for the required `type`. Garden entries now carry `type: Garden Entry`, and the palace audit gains an advisory, unscored interop line.
- `loci-wal/`: the egress-receipt trust primitive (#84, 2026-07-15). An append-only, payload-free, hash-chained write-ahead log of what left the device, plus an Ed25519-signed proof bundle a third party verifies offline. The chain orders frames and breaks on piecemeal edits; the signed bundle is the cryptographic tamper-evidence, and it carries no payload hash or byte count. Public half only: key custody and the disclosure policy stay outside this repository. The writer that recorded frames lived in the desktop app and left with it (see Removed): in this release nothing in the repository appends to the log, and a runtime that wants receipts calls `record_egress`.
- `loci-cli`: two commands on the receipt (#84). `loci audit [--wal PATH] [--since ISO]` reads the live WAL and prints what left the device grouped by egress class, with hash-chain integrity and per-class destination hosts (`--json` supported). `loci wal verify <bundle> [--expect-key HEX]` verifies an exported bundle offline; `--expect-key` pins the signer for provenance. Both cap the file size before reading.
- **Rain: the garden watering round, coupled to a token-window watcher.** Two pieces, one pattern (`templates/skills/rain.md`): (1) `loci tokens`: approximate agent-runtime 5-hour session-window status reconstructed read-only from local transcripts, streamed line-by-line (timing and spend, not quota; honest about being approximate). (2) `loci rain [--fire]`: watering weather on one screen (window signal `fresh`/`open`/`closing`, garden plant count, last rain from the `garden/.rain/` archive) plus the one hand-off in the CLI: `--fire` execs the user's agent runtime from the palace root and exits. Rain never auto-fires: weather is a suggestion, spending is a gate.
- `loci new <path>` scaffolds a starter palace (`PALACE.md`, a `CLAUDE.md` pointer, a soul file, a starter room, `tracker.json`, the lifecycle skills) and hands off to `loci init` (#89, #93).
- `loci overview` (alias `commands`): every command grouped, each with a one-line description read live from `--help` and one example (#88).
- `templates/first-session-protocol.md`: an agent-executable first session. A newcomer pastes one block into a fresh agent, answers one question at a time, and the palace is named in a short ceremony (#91).
- `tutorials/keep-your-memory-yours.md`: keep the palace the only memory of record when the tool you point at it can build its own server-side memory of you (#94).

### Changed
- `PALACE.md` is the primary, AI-agnostic file in every setup door (`AGENT-SETUP.md`, `SETUP-GUIDE.md`, `loci new`, the README); `CLAUDE.md` is a two-line pointer for Claude Code (#92, #93, #95).

### Fixed
- `loci-cli` palace detection accepts the layout the templates kit builds (`rooms/<room>/CLAUDE.md`). `loci status --palace <kit palace>` no longer returns "no palace found". Reported as `rooms-dir`; the `_palace/` and rooms-at-root layouts are unchanged. Unit tests cover all three.
- Setup docs reconciled: handover path (`soul/handovers/`), template pointers (`_templates/`), a path-agnostic scheduled-task example, a Rust prerequisite before `cargo install`, a Claude Code variant of the first-session card, a `Palace:` line in the global identity block, and one paragraph in each setup door naming what the other door creates.
- The scheduled-task locator in `SETUP-GUIDE.md` looks for `PALACE.md`, not `CLAUDE.md` (#96).
- `templates/first-session-protocol.md` no longer downloads anything or goes looking for files. Phase 1 used to fetch the moving `main` branch (falling back to a shallow clone) and act on it with no pin, hash or signature; Phase 2 used to unpack any `loci-core-update_*.tar.gz` it found in the palace folder or in `~/Downloads`. Now the holder names the kit and any core update file. For an archive, the agent shows its name, size and sha256, compares a sha256 the holder pastes, lists its contents (stopping on absolute or `..` paths, and on core-update entries outside `_core/`), and unpacks only after a yes; a kit the browser already unzipped, or a clone, is copied in as a named folder after a yes. The first session makes no network calls; `FIRST-SESSION.md` says the kit now sits beside the paste block.
- First-session unpacking, hardened: the listing now shows entry types (`zipinfo`, `tar -tvzf`) and stops on anything but regular files and folders, so a symlink entry such as `_core/skills -> ~/.claude` no longer passes; a copied kit folder gets the same check. An unpack that exits with an error stops before anything is read or copied. A `_kit` folder counts only when its five required paths exist; an incomplete one is set aside as `_kit-rejected` after a yes, and a leftover `_kit-incoming` is cleared only after a yes. A core update unpacks into `_core-incoming` and swaps in, with an earlier `_core` renamed to `_core-previous-<date>` instead of overlaid; `./_core/` entries pass. A sha256 mismatch writes its own RESUME.md note.

### Removed
- The Tauri desktop app (`desktop/`), including its embedded MCP server. Loci runs as plain markdown plus the optional CLI; the app was a case study, not a run mode. The code is archived at the `archive/desktop-app` tag for later work. CI drops the desktop and frontend jobs, the npm workspace drops `desktop`, and the CLI's overview now names two doors (CLI, templates kit).


---

## [v0.6.0-beta] - 2026-07-02

The 0.6 open beta: the desktop app, the CLI, and the template shelves on one
version line for the first time. Methodology line: `loci-core` 1.6-candidate
(supersedes 1.5-candidate; see PALACE-METHODOLOGY.md for the cockpit,
gate-ledger, and release-gates doctrine).

### Added
- `docs/RELEASING.md`: how a change ships to this repository. The ordered gates (scope, pre-flight, bleed scan, human read, commit gate, reconstruction pre-mortem, PR, merge, post), version-anchor semantics for the methodology line, CHANGELOG and README touch triggers, tagging rules, and the rollback table. The contributor-facing half of the release discipline; the maintainer-side check wiring stays outside the repository.
- **Desktop cockpit** (rc.3, item E): the dashboard grows a tab strip. One Operations view (the native dashboard) plus one tab per palace-map instrument discovered in the palace (self-contained `*.html` with an embedded `<script id="payload"|"snapshot">` JSON payload, found at root / `_palace/` / `cockpit/`; `dashboard.html` excluded). Instruments render via path-validated `srcdoc` embedding (traversal and symlink escapes rejected, unit-tested); tab badges peek the payload for a best-effort count. Three new read-only commands: `list_palace_maps`, `read_palace_map_html`, `read_tools_ledger`.
- **Tools gate-ledger on the dashboard**: palaces that keep `tools.items` in `palace-map.json` (or `map.json`) get a "Tool shelf · gate ledger" section: one state-colored card per external tool carrying its quarantine verdict (`admitted` / `admitted-escorted` / `deferred` / `held-conditional` / `rejected`) and gate read. The shelf lists; it never loads. Fail-soft: no ledger, no change.
- **Brand coherence for embedded instruments** (single stylesheet for now): each embedded instrument gets one appended override stylesheet remapping the conventional `:root` palette variables to the app's own palette, with structural overrides for the known hardcoded-dark surfaces and JS-set stroke literals. Instruments not using the convention are unaffected; a real theming contract is a later RC.
- `loci-cli/`: a new terminal-native CLI for reading the palace. Five commands (`status`, `crystals`, `read`, `handover`, `init`), all read-only, stdlib + a handful of small crates, no network, no inference. The third door alongside the templates kit and the desktop app. Install with `cargo install --path loci-cli`; see `loci-cli/README.md`.
- `templates/personas/`: a shelf of filled persona examples. Ships with four: `Vesper.md` (orchestration; the one template you are meant to rename, since the orchestrator is the companion the naming ceremony is for), `Cipher.md` (security / adversarial) + `Praxis.md` (sysadmin / reversibility) as a paired engine-room dyad showing the theoria/praxis split, and `Nyx.md` (design / poiesis), completing the Aristotle triad. Generic, no organisation-specific lock-in; fork into your palace and adapt.
- `templates/skills/`: a shelf of portable disciplines with stated triggers, procedures, and kill conditions. Ships with three: `quarantine.md` (the long-form procedure behind the v1.4 "Foreign-process quarantine, numbered" doctrine), `session-close.md` (end on purpose: reflect, adjust, persist, record), and `insight-consolidation.md` (the write-side of palace memory: one fact per note, surprise-gated, prunable).
- `templates/eval-framework.md`: two-phase co-intelligence growth instrument: Foundation Scorecard (12 axes, 0-5, lock at ceiling), Growth Scorecard (G1-G7 dyad-quality axes, 1-10, no ceiling), and a Frontier Section for tracking orchestration moves until they recur enough to become axes.
- `palace-audit` gains a 6th dimension, "Tracker integrity": walks `tracker.json` for broken artifact refs, version staleness (a ref pointing below the highest-numbered artifact on disk), placeholder tracks, and unreferenced drafts. Read-only. Total score moves to /30.
- `templates/.lociignore`: a memory-vs-material ignore-list that scopes the audit scan so a vendored tree does not swamp it.
- Two operating rules in `CLAUDE-master.md`: "confirm against disk before asserting state or absence", and "quarantine foreign processes" (the inbound complement to "nothing leaves without approval").
- `PALACE-METHODOLOGY.md` v1.5-candidate: **the memory lifecycle**. Names the crystal and garden lifecycle as one doctrine (keep what surprised you, let the rest decay on its tier's own clock, keep all of it inspectable and prunable) and adds the surprise-gated-write and multi-rate-review principles to it.
- `templates/memory-lifecycle.md`: the four lifecycle principles on one page, with pointers to the tier guide and the garden-health pass for the mechanics.

### Changed
- **Palace detection accepts rooms-at-root layout.** `validate_palace_path` and `load_palace` (`desktop/src-tauri/src/main.rs`) no longer require a `_palace/` subdir. Both the desktop and the new CLI now accept two layouts: the original *rooms-inside-`_palace`/* shape AND the *rooms-at-root* shape (PALACE.md/CLAUDE.md at root with sibling room directories each holding CLAUDE.md). Palaces ported from older organic layouts now load directly; both shapes coexist.
- **The job table has one owner.** When a palace ships instrument tabs, the native dashboard keeps only the KPI header and the automation instrument owns the cron list; palaces with no instruments keep the native table as fallback. Instrument discovery is keyed to the palace, not to cron writes, so an open instrument tab no longer remounts (losing scroll and filter state) every time a state.json is written.

---

## [v0.1.0-beta.1] - 2026-06-09

First public beta of the loci CLI and template kit.

### Added
- Plain-markdown palace setup (no build required) via `templates/` kit
- `FIRST-SESSION.md` and `AGENT-SETUP.md` guided onboarding
- `SETUP-GUIDE.md` full walkthrough
- `ARCHITECTURE.md` system overview and design rationale
- `PALACE-METHODOLOGY.md` protocol reference (check-for-updates anchor)
- `PALACE-UPDATE.md` update protocol documentation
- `packages/core`: core loci Node.js package
- `discoveries/` community findings folder
- `CONTRIBUTING.md` contribution guide

### Notes
- Desktop app (Tauri) releases are tagged separately under `desktop/vX.Y.Z`
- The markdown-only path is stable; the CLI surface is in active development
