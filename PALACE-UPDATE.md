<!--
  Palace update protocol. Agent-facing entry point for `palace-update`.
  Reads the version anchor in PALACE-METHODOLOGY.md and reports the delta.
-->

# PALACE-UPDATE.md · palace update protocol

> **You are an AI agent.** A human with an existing palace has asked you to update it.
> Your job: compare what they have against the current loci methodology, show them the delta, and offer the new pieces one at a time. They decide what enters.
> This is not first-time setup. Do not start over. Begin from what they already have.

This is the read-only, explicit half of loci's update story. You report what changed. You never push, auto-apply, or reach out on your own. The human pulls, sees the diff, and consents. The methodology you update against lives in `PALACE-METHODOLOGY.md`.

---

## Before you start

1. Read the user's palace: their `CLAUDE.md`, soul files, and room files. See what they actually have.
2. Read the published `PALACE-METHODOLOGY.md`. The `> loci-core version:` line is the current methodology version; the entries below it are what each version added.
3. Read the user's own `PALACE-METHODOLOGY.md` if they have one. Its `> loci-core version:` is their installed version.

Takes about 60 seconds. Do it before you show the delta.

---

## Trigger phrases

- "Update my palace"
- "What's new in loci"
- "Check if my palace is up to date"
- "What am I missing"
- `palace-update`

---

## Protocol

```
1. Read the user's palace (CLAUDE.md + soul files + their methodology version, if any)
2. Read the published PALACE-METHODOLOGY.md (latest version + per-version entries)
3. Build the delta: the versions between theirs and latest = what changed
4. Report it: each item with why-it-matters and the exact change, never just a version number
5. Cherry-pick: offer the new pieces one at a time
6. Apply only what they accept
7. Confirm what changed, and write their methodology version forward
```

Read-only by default. Steps 1-4 never write. Steps 5-6 write only what the human accepts, one piece at a time.

---

## What to check

| Area | What you check |
|------|----------------|
| Room coverage | Core rooms present, each with its standard sections? |
| Crystal schema | All three tiers (◆ ◈ ◇)? `valid_until` where it matters? |
| Handover format | The mandatory sections? An artifact listing? |
| Garden | A garden with at least one active plant? |
| Personas | Soul files for any named thinking modes in use? |
| Routines | A morning check-in or autodream configured? |
| Output primitives | In scope, if the user exports palace work? |
| Insight decay | Time-sensitive crystals marked with `valid_until`? |
| The Gate | An explicit allow/deny split (what to stop-and-ask vs what to just-do)? |

---

## Versioning

Orient the delta with the `> loci-core version:` anchors.

- The user's version is in their `PALACE-METHODOLOGY.md`, or ask them.
- Show only what changed since their version. If unsure, run a full check and note what is new.
- A `stable:` line and a `status:` line tell you whether the latest is a ratified version or an in-flight candidate. Offer candidate features as candidates.

---

## Cherry-pick order

Offer optional features most-impactful first:

1. The Gate (if no explicit allow/deny split)
2. Output primitives (if not in use)
3. Garden (if missing or empty)
4. Morning check-in / daily routine (if not scheduled)
5. Autodream (if not scheduled)
6. Palace audit (if not run recently)
7. Persona soul files (if personas are used without them)
8. Entanglement tracking (experimental, flag as such)

One question at a time. `skip` and `skip all` always valid.

---

## Tone

This person already has a palace. They are not starting over. The update is additive: you fill gaps, you do not rebuild. Keep it brisk.

If everything is current:
> "Your palace is current. [N] areas checked, [N] optional pieces available. Want any?"

If there is a delta:
> "You are on [version]. [N] things changed since. Here they are."

---

*First-time setup uses the setup guide, not this file. The methodology this updates against is `PALACE-METHODOLOGY.md`.*
