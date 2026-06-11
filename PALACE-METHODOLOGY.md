<!--
  loci-core methodology version tracker.
  Version anchor is the parseable blockquote below (loci-core version / stable / status).
-->

# loci-core · palace methodology

Palace methodology version tracker. The structural and intellectual core of loci, independent of the app, MCP server, and site versions.

`loci-core` is the **firmware** of the palace: the templates, processes, and concepts that define how a palace works. The methodology is plain text and engine-agnostic. Any LLM with file access can run it. When you check for updates, this is the version you are comparing.

> This document tracks the *methodology* line of loci-core: the plain-text firmware, versioned independently of the apps.

> loci-core version: 1.4-candidate
> stable: 1.2
> status: candidate
> updated: 2026-06-11

---

## v1.4-candidate · 2026-06-11

**Co-intelligence eval framework.** A two-phase growth instrument for measuring
how well a palace and its AI companion operate together over time. Phase 1
(Foundation Scorecard, 12 axes, 0-5) establishes the working basis. Phase 2
(Growth Scorecard, G1-G7, 1-10) measures dyad quality and learning velocity with
no ceiling. A Frontier Section tracks specific orchestration moves to try in the
next eval window; demonstrated moves graduate to Growth axes. Template:
`templates/eval-framework.md`.

---

## v1.3-candidate · 2026-06-08

**Portability seam (cross-provider detection).** Palace detection recognises memory laid down by other tools, by structure only: Claude Code (`CLAUDE.md`), Codex / kilo (`AGENTS.md`), Cursor (`.cursor/rules`, `.cursorrules`), Windsurf (`.windsurfrules`). Foreign content is never trusted by construction; reading it is a separate, quarantined step. The principle: loci reads *across* providers without anyone federating up to a central server. Portability, not centralisation, is the moat.

**`palace-update`, revived.** The update path returns as the methodology behind the app's check-for-updates: read this version anchor, compare against the latest published loci-core, and report what changed. Read-only and explicit by default. The check runs when you ask it to, never as a background poll, and it shows the delta rather than applying it. You decide what enters your palace.

---

## v1.2 · 2026-05-11

**Palace Dashboard overlay.** Local palace-state tracker rendered as a dashboard: rooms grid with state dots, active tracks (persona-tagged), a gate-aware phase roadmap, blockers, and a crystal counter. All state read locally, nothing leaves the machine. Wireframe lives in the app at `desktop/src/palace-dashboard.html`.

---

## v1.1 · 2026-05-11

**Node-entry ritual.** Four protocol levels (0 = none, 1 = phrase, 2 = breath + visual, 3 = full focus engine). Always skippable. Sovereignty principle: content is always one tap away. The ritual is a threshold, never a gate.

---

## v1.0 · 2026-05-08

**Garden Health process.** A structured health pass over crystals, plants, handovers, and rooms. Surfaces stale or dormant nodes for human review. Never auto-archives. Produces a health report per run.

**Crystal pin.** A `pinned: true` field (or `· pinned` inline marker) protects any crystal from garden-health flagging. `pinned_until: DATE` supports time-windowed protection. Full crystal lifecycle: Seeded → Contextual → Confirmed → Pinned → Composted.

**Session Strategy as a room attribute.** A `session_strategy` block in every room declares its context-loading scope: `always-load`, `on-demand`, or `per-session`. `auto_detect: true` lets the agent read the first message for room signals before asking.

**Observation Scope.** A section in the soul file declaring what the AI tracks about the user, about itself, and what it deliberately ignores. Turns memory from accidental into intentional.

**Synthesis Automation.** A full template for the palace synthesis pass. Five synthesis questions, four verbosity modes (verbose / quick / pattern-only / connections). `auto_apply: false` is non-negotiable: synthesis proposes, the human confirms.

**Peer Cards.** A human-authored user representation, written with intention rather than auto-extracted. Covers identity, working style, domain expertise, values, constraints, and working history. The companion to Observation Scope.

---

## v0.9 · 2026-05-08

**Output primitives.** Typed, named formats for exporting palace work as archivable artifacts. First primitive: `git-log-incident`, a session arc rendered as an annotated commit log.

**`palace-update`.** The update entry point for existing palace holders: a delta analysis of your palace against the current loci-core version, with a verbose gap report. *(This is the methodology behind the app's "check for updates": it reads this version anchor and reports what changed.)*

**`loci-feature-release` process.** End-to-end release pipeline for palace features: template → feature card → changelog → branch → gate → PR → deploy.

---

## v0.8 · May 2026

**Persona Roster + Self-Starter Orchestration Loop.** The quartet pattern (an orchestrator plus specialist personas) with an explicit decision loop. *Józan paraszti ész* gate before escalating to multi-agent: ask what the simplest shape is that still works, and escalate only when it demonstrably fails. Never auto-invoke; offer to the human first.

**`<success_criteria>` in the prompt wrapper.** Evaluation-first prompting: define what done looks like before writing the task instruction.

**`palace-audit` process.** A structural self-scan: reads the CLAUDE.md files, soul files, and handover chain for staleness, duplication, broken references, and coverage gaps. Scores out of 25.

**Local map.** An ASCII palace architecture diagram. Five layers: global behavioral → project state → rooms → knowledge core → connections.

---

## v0.7 · May 2026

**Crystal tiers formalised (◆ ◈ ◇).** Confirmed / contextual / exploratory, with `valid_until` expiry and promotion criteria between tiers.

**Entanglement tracking.** An experimental log of resonance peaks, named unknowns, fruits, and patterns.

**The Gate (`[username]GATE`).** A named human-review checkpoint before anything ships, sends, or becomes irreversible. The right balance of human-AI attention is never fixed; the gate is where it is continuously refined.

**Garden-memory generator.** A pass that assesses each plant's arc (seed / sapling / plant / crystal-ready / fork / stale) and proposes promote / retire / fork / new-question.

**Individual garden files.** One file per plant, with full archaeology and no merge conflicts.

---

## v0.6 · April 2026

**`session-delta` process.** A structured handover written at session close: artifact list, TL;DR, state snapshot, decisions, open blockers, and the exact next-session opener.

---

## v0.5 · April 2026

**`palace-update` process.** Delta analysis of a user's palace against current loci features, with verbose gap reports (why it matters, the exact fix, an effort estimate). Verbosity modes: full / quick / area-specific / summary.

**Cherry-pick onboarding flow.** Opt-in questions: morning check-in, autodream, skill-eval cadence, insight-decay rules. One question at a time; `skip` and `skip all` always valid.

---

## v0.4 · April 2026

**ASCII logo.** Four rooms, one per letter, drawn in the same box characters as the palace walls. One visual language throughout.

**Engine-agnostic "works with".** The palace is plain text; any LLM with file access can run it. The naming convention is not a lock-in.

---

## v0.3 · April 2026

**Naming ceremony.** The agent's name is shaped by what it has learned about the user, not offered cold at the start.

**Daily routine check-in.** A personalised morning brief, seeded by how the user actually starts their day.

**Autodream.** Weekly scheduled palace housekeeping. Runs without you.

---

## v0.2 · March-April 2026

**Garden first-class.** The garden moves from optional appendix to core feature.

**L0-L3 retrieval hierarchy.** Context loads in priority layers: soul identity → active state → room context → deep history.

**Persona templates.** Named thinking modes with their own soul files and gardens.

**Crystal expiry.** A `valid_until: YYYY-MM-DD` field keeps stale facts from calcifying as ground truth.

---

## v0.1 · March 2026

Initial release. Room structure, context crystals, the soul file, session deltas, and the CLAUDE master template. Four-room default layout. Basic handover format. Crystal tiers (◆ ◈ ◇).

---

*loci-core tracks the palace, not the apps. Local-first, plain text, yours.*
