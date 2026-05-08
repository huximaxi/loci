# Changelog

All notable changes to loci are documented here.

---

## v0.9 — 2026-05-08 (palace)
- **Git Log Incident output primitive** — `templates/git-log-incident.md`: structured format for documenting session arcs as annotated git logs. Each discrete action = one commit. Tags vocabulary: `[DISCOVERY]`, `[FATAL DISCOVERY]`, `[PLOT TWIST]`, `[FALSE NEGATIVE]`, `[SALVATION]`, `[FORENSICS]`, `[HTTP 400/500]`, `[DELIVERED]`, `[HUXGATE]`, `[NOISE]`, `[SIGNAL]`. Vocabulary is open — add tags as arcs demand. Output: HTML widget (monospace card layout, hash in accent color, tags as badges) or diffable MD file. Atomic principle: the 400 before the 200 is part of the story.
- **Output primitives feature card** — landing page (`index.html`): new feature card across all three themes (Scholar/Wizard/LLMAGE) documenting the output primitives concept and git-log-incident as the first example.

---

## v1.2.0 — 2026-05-05 (extension)
- Content sanitization (THREAT-01 mitigation)
- Message sender validation (THREAT-04)
- Search rate limiting (10/sec per tab)
- INSTALL.md for non-technical users
- Security analysis completed (Cipher audit)

## v0.1.0 — 2026-05-05 (desktop)
- KISS palace detector + migrator
- Detects loci, MemPalace, Karpathy-style, PALACE.md variants
- Migration to ~/.loci/ format
- Mac + Windows (Tauri v2)
- Scholar theme (green/cream)

## loci.garden — 2026-05-05 (site)
- Comparison page vs MemPalace / LLMChronicle / Karpathy
- Resources dropdown: "Compare" link added
- LLMAGE title: "loci: the context primitive"
- Wizard hero: right-aligned, contrast improved
- sitemap.xml updated

---

## v1.1.0 — 2026-05-03 (extension)
- Side panel search UI
- Tag management (add/remove/filter)
- Platform detection (Claude.ai, ChatGPT)
- MiniSearch integration
- IndexedDB storage layer

## v1.0.0 — 2026-05-01 (extension)
- Initial Chrome MV3 extension release
- Conversation extraction from Claude.ai and ChatGPT
- Local indexing with MiniSearch
- Overlay UI for quick search
- Core types package (`@loci/core`)

## loci.garden v3 — 2026-04-28 (site)
- Three-theme landing (Scholar, Wizard, LLMAGE)
- Theme switcher in navbar
- Palace map illustration
- SEO: meta tags, JSON-LD, sitemap.xml
- llms.txt agent context declaration
- The Seed dispatch archive

---

## v0.8 — May 2026 (palace)
- **Persona Roster + Self-Starter Orchestration Loop** — `templates/CLAUDE-master.md`: quartet pattern (orchestrator + specialist personas) with an ASCII decision loop. Józan paraszti ész gate before escalating to multi-agent. Never auto-invoke — offer to human first. Light swarm capped at 2–3 agents.
- **Józan paraszti ész** — new principle in CLAUDE-master.md: before committing to a complex multi-agent approach, ask what the simplest shape is that still works. Escalate only when the simple version demonstrably fails.
- **`<success_criteria>` in prompt wrapper** — CLAUDE-master.md now includes a `success_criteria` field between `context` and `task`. Evaluation-first prompting: define what done looks like before writing the task instruction.
- **Clarifying questions protocol** — wrapper note: if a field cannot be inferred, ask one focused question. Human can reply "keep it broad" — valid answer, not a non-answer. One question max.
- **Model selection principle** — match model to task weight. Specific model name lives in project CLAUDE.md (goes stale); the principle lives in the global layer (stable).
- **`palace-audit` process** — `templates/palace-audit-process.md` + `PROCESSES.md` entry. Structural autodream for the palace setup: scans CLAUDE.md files, soul files, skills, handover chain for staleness / duplication / broken refs / coverage gaps / architectural drift. Scores /25. Reports to `[palace]/audits/YYYY-MM-DD.md`. Enforces the two-layer rule: global = behavioral constants only; project CLAUDE.md = living state only.
- **`local-map-template.md`** — `templates/local-map-template.md`: ASCII palace architecture diagram template. Five layers: global behavioral → project state → rooms → knowledge core → connections. Filled once, updated by palace-audit on each structural autodream.

## v0.7 — May 2026 (palace)
- **Crystal tiers formalised (◆◈◇)** — `templates/crystals-guide.md`: confirmed / contextual / exploratory tiers with `valid_until` expiry fields, promotion criteria between tiers, and example crystal blocks. Morning check-in now surfaces expiring ◈ crystals for human decision (delete / migrate / renew).
- **Entanglement tracking** — `templates/entanglement-template.md`: experimental log of resonance peaks, named unknowns, fruits, and patterns. `entanglement-housekeeping` process with 12-question rotating bank. Added as highly-recommended onboarding option with explicit experimental caveat.
- **`[username]GATE`** — new atomic process: named human review checkpoint before anything ships, sends, or becomes irreversible. Named per user (e.g. HuxGATE). Core principle: the right balance of human-AI attention is never fixed — the gate is where it's continuously refined. Added to onboarding as Block Q10h.
- **Garden-memory generator** — `garden-memory-generator` process: mnemonic conductor that assesses plant arcs (seed / sapling / plant / crystal-ready / fork / stale), detects cross-plant chords, and proposes promote / retire / fork / new-question for each plant.
- **Individual garden files** — `templates/garden-file-template.md`: numbered per-plant session files (`garden/[plant]-NNN.md`) with richer archaeology, git history per plant, and no merge conflicts.
- **`_PALACE_CONTEXT.md`** — `templates/_PALACE_CONTEXT.md`: living session pointer bridging cold-starts. Tracks active corridors (hot/warm/cold), memory scrolls, pending decisions, and entanglement signal.
- **Friends template** — `templates/friends/friend-template.md`: structured friend soul format for palace-to-palace context sharing. Working portrait, cognitive style, key crystals, collaboration notes.
- **Tracker schema v2** — `templates/tracker.json` palace-generic: added `palace` / `owner` / `ai` top-level fields, `tier` and `artifacts` per track, `tier1` / `tier2` protocol split. Was project-scoped; now orchestrates all palace workstreams.
- **Retrieval soft guideline** — `templates/retrieval-hierarchy.md` extended with a human-facing section: how to use L0–L3 when tired, context-switching mid-day, or returning after a week away.
- **loci.garden** — nine-card feature section added to homepage (`What's in the palace`): allied hero, low-tech memory palace, garden, insight crystals, process blueprints, personal tutor, personas, friends, entanglement (coming soon). Material Icons + Inter typography.

## v0.6 — April 2026 (palace)
- **`session-delta` process** — structured handover written at session close. Mandatory artifact listing (all files created/edited/deleted), TL;DR, state snapshot, decisions, open blockers, and exact next session opener. Established after a high-volume build sprint where implicit tracking was insufficient.
- **Website** — [loci.garden](https://loci.garden) live. Public face of the methodology: palace map, three doors, dispatch archive, llms.txt agent declaration.
- **Communication modules** — `modules/zulip-crawler/` generalised; docs now describe optional integrations for any team communication tool rather than Zulip-specific setup.

## v0.5 — April 2026 (palace)
- **`palace-update` process** — delta analysis of user's palace vs. current Loci features. Verbose gap reports (why it matters, exact fix, effort estimate). Verbosity modes: full / quick / area-specific / summary.
- **Cherry-pick onboarding flow** — Block 9 of `AGENT-SETUP.md` expanded with four opt-in questions: morning check-in, autodream, skill eval cadence, insight decay rules. One question at a time. `skip` and `skip all` always valid. Revisitable any time via `palace-update`.

## v0.4 — April 2026 (palace)
- **ASCII logo** — four rooms, one per letter. Letters drawn in the same box-drawing characters as the palace walls (`│ ┌─┐ └─┘ ───`). One visual language throughout.
- **Engine-agnostic "Works with"** — palace is plain text; any LLM with file access can run it. Works across multiple accounts (work + personal) seamlessly. `CLAUDE-master.md` is a naming convention, not a lock-in.
- **Changelog** — added to README; covers v0.1 through v0.4.

## v0.3 — April 2026 (palace)
- **Naming ceremony** — agent name moved to Block 8 (after garden + daily routine). Names are now shaped by what the agent has learned about the user, not offered cold at the start.
- **Daily routine** — new onboarding question asks how the user actually starts their day. Stored as a crystal; seeds every morning check-in with real context instead of a generic template.
- **Autodream** — weekly scheduled palace housekeeping (garden round + pattern scan + stale tracker check). On by default. Runs without you.
- **Daily routine check-in** — personalised morning brief process. Pulls from your comms tool and/or Jira if configured.
- **Communication modules** — optional integrations (Slack, Discord, etc.) for pulling digests into the morning check-in. Drop any compatible module into `modules/`.
- **Cross-environment portability** — palace is file-based; works identically across Claude Code, Cowork desktop, and web. Documented in onboarding and README.

## v0.2 — March–April 2026 (palace)
- **Garden first-class** — garden moved from optional appendix to core feature. Competitive differentiator: no other co-intelligence scaffold has it.
- **L0–L3 retrieval hierarchy** — context loads in priority layers (soul identity → active state → room context → deep history). Documented in `templates/retrieval-hierarchy.md`.
- **Persona templates** — named thinking modes with their own soul files and gardens. Personas collaborate via tea sessions.
- **Crystal expiry** — `valid_until: YYYY-MM-DD` field added to crystal format. Prevents stale facts from calcifying as ground truth.
- **Scheduled tasks** — `templates/scheduled-task-template.md`: morning briefs, garden rounds, deep synthesis. Dynamic path finding (no hardcoded session paths).
- **Comparison table** — honest positioning vs MemPalace (benchmarked, vector search) and Karpathy-style (simplest). Different tools for different needs.
- **Renamed to Loci** — was `palace-starter`. Method of Loci. Classical, 4 letters.

## v0.1 — March 2026 (palace)
- Initial release: room structure, context crystals, soul file, session deltas, CLAUDE-master template.
- 4-room default layout (Great Hall, Dev, Design, Hatchery).
- Basic handover format. Tracker JSON. Crystal tiers (◆ ◈ ◇).

---

*Built by Hux × Vesper · 2026 · [loci.garden](https://loci.garden)*
