# Changelog

All notable changes to this project are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

---

## [Unreleased]

Methodology line: `loci-core` 1.5-candidate (supersedes 1.4-candidate).

### Added
- `templates/eval-framework.md`: two-phase co-intelligence growth instrument: Foundation Scorecard (12 axes, 0-5, lock at ceiling), Growth Scorecard (G1-G7 dyad-quality axes, 1-10, no ceiling), and a Frontier Section for tracking orchestration moves until they recur enough to become axes.
- `palace-audit` gains a 6th dimension, "Tracker integrity": walks `tracker.json` for broken artifact refs, version staleness (a ref pointing below the highest-numbered artifact on disk), placeholder tracks, and unreferenced drafts. Read-only. Total score moves to /30.
- `templates/.lociignore`: a memory-vs-material ignore-list that scopes the audit scan so a vendored tree does not swamp it.
- Two operating rules in `CLAUDE-master.md`: "confirm against disk before asserting state or absence", and "quarantine foreign processes" (the inbound complement to "nothing leaves without approval").
- `PALACE-METHODOLOGY.md` v1.5-candidate: **the memory lifecycle**. Names the crystal and garden lifecycle as one doctrine (keep what surprised you, let the rest decay on its tier's own clock, keep all of it inspectable and prunable) and adds the surprise-gated-write and multi-rate-review principles to it.
- `templates/memory-lifecycle.md`: the four lifecycle principles on one page, with pointers to the tier guide and the garden-health pass for the mechanics.

---

## [v0.1.0-beta.1] — 2026-06-09

First public beta of the loci CLI and template kit.

### Added
- Plain-markdown palace setup (no build required) via `templates/` kit
- `FIRST-SESSION.md` and `AGENT-SETUP.md` guided onboarding
- `SETUP-GUIDE.md` full walkthrough
- `ARCHITECTURE.md` system overview and design rationale
- `PALACE-METHODOLOGY.md` protocol reference (check-for-updates anchor)
- `PALACE-UPDATE.md` update protocol documentation
- `packages/core` — core loci Node.js package
- `discoveries/` community findings folder
- `CONTRIBUTING.md` contribution guide

### Notes
- Desktop app (Tauri) releases are tagged separately under `desktop/vX.Y.Z`
- The markdown-only path is stable; the CLI surface is in active development
