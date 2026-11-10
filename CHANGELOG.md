# Changelog

All notable changes to this project are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

---

## [Unreleased]

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
