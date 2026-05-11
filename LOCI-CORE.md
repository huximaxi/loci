# loci-core

Palace methodology version tracker. The structural and intellectual core of loci - independent of app, extension, and site versions.

`loci-core` is the "firmware" of the palace: the templates, processes, and concepts that define how a palace works. When you run `palace-update`, you are upgrading your loci-core version.

---

## loci-core v1.2 candidate - 2026-05-11

**Palace Dashboard overlay** - Tauri RPG overlay for local palace state tracking. Wireframe at `desktop/src/palace-dashboard.html`. Sections: rooms grid (8 rooms, state dots), active tracks (persona-tagged), phase roadmap (gate-aware), blockers/HUXGATEs, crystal counter. Skin slots annotated inline (`<!-- SKIN: ... -->`). Tauri invoke hooks annotated inline (`<!-- TAURI: ... -->`). Build order: static wireframe → wire Tauri commands → apply skins.

---

## loci-core v1.1 candidate - 2026-05-11

**Node-entry ritual** - four protocol levels (0=none, 1=phrase, 2=breath+visual, 3=full CSE). Always skippable. YAML spec in room soul file. Sovereignty principle: content always one tap away. CSE widget is the Level 3 engine. Spec: `pipeline/features/node-entry-ritual.md`.

---

## loci-core v1.0 - 2026-05-08

**Garden Health process** - structured health pass for crystals, plants, handovers, and rooms. Surfaces stale/dormant nodes for human review. Never auto-archives. Produces a `_health.md`-style report per run.

**Crystal pin feature** - `pinned: true` field (or `· pinned` inline marker) protects any crystal from garden health flagging. `pinned_until: DATE` supports time-windowed protection. Full crystal lifecycle: Seeded → Contextual → Confirmed → Pinned → Composted.

**Session Strategy as room attribute** - `session_strategy` YAML block in every room declares context loading scope: `always-load`, `on-demand`, or `per-session`. `auto_detect: true` makes the agent read the first user message for room signals before asking.

**Observation Scope** - new `## Observation Scope` section in `SOUL.md` declares what the AI tracks about the user, about itself, and what it deliberately ignores. Transforms memory from accidental to intentional.

**Synthesis Automation** - full process template for the palace synthesis pass. Five synthesis questions. Verbosity modes (verbose / quick / pattern-only / connections). Scheduled task setup. `auto_apply: false` is non-negotiable - synthesis proposes, human confirms.

**Peer Cards** - human-authored user representation at `_USER.md`. Unlike auto-extracted profiles, the peer card is written with intention. Covers identity, working style, domain expertise, values, constraints, working history. Companion to Observation Scope.

---

## loci-core v0.9 - 2026-05-08

**Output primitives** - typed, named formats for exporting palace work as archivable artifacts. First primitive: `git-log-incident` - session arcs as annotated commit logs.

**`PALACE-UPDATE.md`** - agent update script for existing palace holders. Entry point for running `palace-update`.

**`loci-feature-release` process** - end-to-end release pipeline for palace features: template → feature card → changelog → branch → gate → PR → deploy.

---

## loci-core v0.8 - May 2026

**Persona Roster + Self-Starter Orchestration Loop** - quartet pattern (orchestrator + specialist personas) with ASCII decision loop. Jozan paraszti esz gate before escalating to multi-agent.

**`<success_criteria>` in prompt wrapper** - evaluation-first prompting: define what done looks like before writing the task instruction.

**`palace-audit` process** - structural autodream: scans CLAUDE.md files, soul files, skills, handover chain for staleness / duplication / broken refs / coverage gaps. Scores /25.

**`local-map-template.md`** - ASCII palace architecture diagram. Five layers: global behavioral → project state → rooms → knowledge core → connections.

---

## loci-core v0.7 - May 2026

**Crystal tiers formalised (◆◈◇)** - confirmed / contextual / exploratory with `valid_until` expiry fields and promotion criteria.

**Entanglement tracking** - experimental log of resonance peaks, named unknowns, fruits, and patterns.

**`[username]GATE`** - named human review checkpoint before anything ships, sends, or becomes irreversible.

**Garden-memory generator** - mnemonic conductor assessing plant arcs: seed / sapling / plant / crystal-ready / fork / stale.

**Individual garden files** - numbered per-plant session files (`garden/[plant]-NNN.md`) with full archaeology.

---

## loci-core v0.6 - April 2026

**`session-delta` process** - structured handover written at session close. Mandatory artifact listing, TL;DR, state snapshot, decisions, open blockers, next session opener.

---

## loci-core v0.5 - April 2026

**`palace-update` process** - delta analysis of user's palace vs. current loci features. Verbose gap reports.

**Cherry-pick onboarding flow** - opt-in questions: morning check-in, autodream, skill eval cadence, insight decay rules.

---

## loci-core v0.4 - April 2026

**ASCII logo** - four rooms, one per letter. One visual language throughout.

**Engine-agnostic "Works with"** - palace is plain text; any LLM with file access can run it.

---

## loci-core v0.3 - April 2026

**Naming ceremony** - agent name shaped by what the agent has learned about the user, not offered cold.

**Daily routine check-in** - personalised morning brief process.

**Autodream** - weekly scheduled palace housekeeping. Runs without you.

---

## loci-core v0.2 - March-April 2026

**Garden first-class** - garden moved from optional appendix to core feature.

**L0-L3 retrieval hierarchy** - context loads in priority layers: soul identity → active state → room context → deep history.

**Persona templates** - named thinking modes with their own soul files and gardens.

**Crystal expiry** - `valid_until: YYYY-MM-DD` field prevents stale facts from calcifying.

---

## loci-core v0.1 - March 2026

Initial release. Room structure, context crystals, soul file, session deltas, CLAUDE-master template. 4-room default layout. Basic handover format. Crystal tiers (◆ ◈ ◇).

---

*loci-core tracks the palace, not the apps. For extension, desktop, and site versions see [CHANGELOG.md](CHANGELOG.md).*
