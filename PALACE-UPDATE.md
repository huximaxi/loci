# PALACE-UPDATE.md — Palace Update Protocol

> **You are an AI agent.** A human with an existing palace has asked you to run a palace update.
> Your job: compare their current setup against the Loci feature set, report the delta, and run a cherry-pick flow for optional features.
> This is not onboarding. Do not re-run AGENT-SETUP.md. Start from what they already have.

---

## Before you start

Read these files to know what the current Loci feature set looks like:

- `PROCESSES.md` — full list of available processes
- `CHANGELOG.md` — what was added in each version
- `templates/` — all available template files
- `templates/CLAUDE-master.md` — the current master context spec

Then read the user's existing palace (their CLAUDE.md and soul files) to see what they have.

Takes about 60 seconds. Do it before opening the delta report.

---

## Trigger phrases

- "Update my palace"
- "What's new in Loci"
- "Check if my palace is up to date"
- "Run the palace update"
- "What am I missing from Loci"
- `palace-update` (process name)

---

## Protocol

Run the `palace-update` process as defined in `PROCESSES.md`. The full spec is there — this file is the entry point.

Short version:

```
1. Read user's CLAUDE.md + soul files + templates in use
2. Read Loci's CHANGELOG.md + templates/ + PROCESSES.md
3. Build a delta: what they have vs. what exists
4. Report gaps (with why-it-matters + exact fix + effort estimate)
5. Cherry-pick flow: offer optional features one at a time
6. Apply anything they accept
7. Confirm what changed
```

---

## What to check

| Area | What the agent checks |
|------|----------------------|
| Room coverage | 4 core rooms present? Each with 5 standard sections? |
| Crystal schema | All three tiers (◆◈◇)? `valid_until` fields where relevant? |
| Handover format | 5 mandatory sections? Artifact listing? |
| Garden | `garden.md` exists with at least one active plant? |
| Personas | Soul files for any named thinking modes? |
| Scheduled routines | Morning check-in or autodream configured? |
| Output primitives | `templates/output-primitive.md` in scope? |
| Processes | All relevant processes known and triggerable? |
| Insight decay | Time-sensitive crystals marked with `valid_until`? |
| **Two-tier wiring** | Identity block at top of CLAUDE.md? Global layer configured for their tool (`~/.claude/CLAUDE.md` / Project Instructions / Cowork workspace)? Run `scripts/check-two-tier.py` for a full report. |

---

## Versioning

Check `CHANGELOG.md` to orient the delta. If the user's palace was set up on an older version:
- Find their last-known feature (by asking or by reading their files)
- Show only changes since that point

If unsure: run a full check and note what's new.

---

## Cherry-pick order

Offer optional features in this priority sequence — most impactful first:

1. Two-tier memory wiring (if identity block missing or global layer not configured)
2. Output primitives (if not in use)
2. Garden (if missing or empty)
3. Morning check-in / daily routine (if not scheduled)
4. Autodream (if not scheduled)
5. Palace audit (if palace hasn't been audited recently)
6. Persona soul files (if personas are in use without soul files)
7. Entanglement tracking (experimental — flag as such)
8. Skill eval cadence (if no eval history)

One question at a time. `skip` and `skip all` always valid.

---

## Tone

This person already has a palace. They are not starting over. The update is additive — you are not rebuilding, you are filling gaps. Keep it brisk. They know how this works.

If everything is current:
> "Your palace is current. [N] areas checked, [N] optional features available — want to cherry-pick any?"

If gaps exist:
> "Found [N] gaps and [N] optional features. Here's the delta:"

---

*For first-time palace setup: use `AGENT-SETUP.md` instead.*
*Full process spec: `PROCESSES.md` → `palace-update`*
