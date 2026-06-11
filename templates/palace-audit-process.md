# Palace Audit: Structural Autodream Process

> Agent-executable process. Trigger: "palace audit" / "structural autodream" / "check our architecture" / "is our file structure healthy"

---

## What this does

Scans the full palace setup (CLAUDE.md files, soul files, skills, handover chain, and the project tracker) for staleness, duplication, broken references, coverage gaps, architectural drift, and tracker integrity. Scores health across 6 dimensions and outputs a report.

Run after major sprints, when architecture feels heavy, or when CLAUDE.md files haven't been reviewed in more than 3 weeks.

---

## 6 Audit Dimensions

| # | Dimension | What it checks |
|---|---|---|
| 1 | **Staleness** | CLAUDE.md files not updated in > 3 sessions; handovers with open blockers already resolved |
| 2 | **Duplication** | Same information in both global layer and project CLAUDE.md; persona soul files duplicated |
| 3 | **Broken refs** | File paths cited in CLAUDE.md / handovers that no longer exist |
| 4 | **Coverage gaps** | Active projects with no handover; rooms with no CLAUDE.md; skills listed but missing |
| 5 | **Architectural drift** | Behavioral constants in project CLAUDE.md (should be global layer); state in global CLAUDE.md (should be project layer) |
| 6 | **Tracker integrity** | `tracker.json` refs pointing at missing files; version staleness (tracker points below the highest-numbered artifact on disk); placeholder tracks with no artifacts; recent files on disk that no track references |

Score: 0–5 per dimension. Total: /30.

---

## Protocol

```
TRIGGER
   │
   ▼
Step 1: SCAN (shell, no LLM reads yet)
   ├── List all CLAUDE.md files
   ├── List all soul files
   ├── List all skill files + check against any skills index
   ├── List handovers by date
   ├── Check file refs exist (bash test -f on each path cited)
   ├── Walk tracker.json: test every path in each track's `artifacts` (test -f),
   │   normalising filenames to Unicode NFC before comparing
   └── For versioned artifacts (name carries vN or rev-N), find the highest-numbered
       match on disk and flag any tracker ref that points below it
   (all scans honour .lociignore: skip "material", walk "memory")
   │
   ▼
Step 2: DIFF DETECTION (read only flagged files)
   ├── Read CLAUDE.md files flagged as stale
   ├── Compare skills index vs actual skill files
   ├── Compare pending blockers in project CLAUDE.md vs latest handover
   ├── Verify global layer workspace paths are current
   └── Read tracker.json + flagged tracks; confirm drift before reporting it
   │
   ▼
Step 3: SCORE + REPORT
   ├── 6-dimension scorecard (0–5 each, /30 total)
   ├── Findings: file | issue | severity (🔴 blocking / 🟡 advisory / 🟢 healthy)
   ├── Specific actions: "update X", "delete Y", "add Z to index", "reconcile tracker ref"
   └── Verdict: HEALTHY / NEEDS_ATTENTION / DRIFT_DETECTED
   │
   ▼
Step 4: PRESENT + OFFER
   ├── Summary: score + verdict + top 3 actions
   ├── Save full report to: [palace]/audits/YYYY-MM-DD.md
   └── Ask: "Want me to execute any of these fixes now?"
```

---

## Staleness heuristics

| File type | Stale if... |
|---|---|
| Room CLAUDE.md | No handover references it in > 3 sessions |
| Project CLAUDE.md | Pending blockers not updated in > 1 session |
| Global CLAUDE.md | Date stamp > 2 weeks old without a process-adjustment trigger |
| Handover file | Lists open blockers already resolved in project CLAUDE.md |
| Skill file | Referenced in index but file missing; or file exists but not in index |
| Soul file | Path in global CLAUDE.md doesn't match actual location |

---

## Tracker integrity (Dimension 6)

`tracker.json` is the one file a human edits while files move underneath it, so it drifts silently and the other five dimensions never look at it. This dimension walks it. **All checks are READ-ONLY: report drift, never rewrite the tracker.** Reconciliation is a human action, offered in Step 4.

For each track in `tracker.json`:

1. **Broken artifact refs.** Every path in `artifacts` must exist on disk. Normalise both the tracked name and the directory listing to Unicode **NFC** before comparing. An em-dash or accented filename stored NFD on disk will otherwise read as "missing" when it is present. This is a real false-positive, not a hypothetical. And if a missing ref carries a version, look for a higher-numbered sibling before calling it broken: a missing `v2.3` beside a `v2.4` on disk is a stale pointer (see staleness below), not a lost file.
2. **Version staleness.** If an artifact name carries a version (`v0.14`, `rev-40`), find the highest-numbered sibling on disk. If the tracker points below it, the live document has moved and the tracker is stale. (Highest-numbered on disk is the live document; see the "confirm against disk" principle.)
3. **Placeholder tracks.** A track marked `active` with empty `artifacts` and a "not started" `current_step` is either mislabelled or genuinely queued. Flag for confirm-or-close.
4. **Unreferenced drafts.** Recent files on disk inside a track's `room` that no track references. The work happened outside a palace session and never made it back to the tracker.

Scope the walk with `.lociignore` (below) so "material" does not swamp it.

---

## .lociignore (memory vs material)

A palace holds two kinds of thing. **Memory** is soul, handovers, crystals, the tracker, CLAUDE.md, garden, rooms. **Material** is vendored codebases, reference dumps, PDFs, and binaries the work consumes. Audits and structural scans should walk memory, not material. A palace with a 10k-file vendored tree will otherwise swamp every scan and time out.

Drop a `.lociignore` at the palace root: one glob per line, listing material to skip. The audit scan (and any other structural scanner) honours it. An example ships in `templates/.lociignore`; copy it to your palace root and edit.

---

## The two-layer rule (what your agent enforces)

```
GLOBAL CLAUDE.MD (Layer 0)    →  behavioral constants only
  ✓ Identity, triggers, prompt wrapper, MCPs, output standards
  ✗ Project state, active blockers, ticket IDs, room specifics

PROJECT CLAUDE.MD (Layer 1)   →  living state only
  ✓ Active themes, blockers, room index, handover pointers
  ✗ Behavioral instructions, prompting techniques

HANDOVERS                      →  session delta only
  ✓ What changed this session, decisions, artifacts, open blockers
  ✗ Permanent knowledge (belongs in project CLAUDE.md)

SKILLS                         →  reusable process commands
  ✓ One SKILL.md or process.md per skill
  ✗ Session-specific context, ticket numbers

ROOM CLAUDE.MD                 →  room-scoped context only
  ✓ Room-specific tools, active work, local decisions
  ✗ Palace-wide context (belongs in project CLAUDE.md)
```

---

## Report format

Reports go to `[palace]/audits/YYYY-MM-DD.md`. They accumulate. The audit history is itself an architectural signal.

```markdown
---
type: palace-audit
date: YYYY-MM-DD
score: XX/30
verdict: HEALTHY | NEEDS_ATTENTION | DRIFT_DETECTED
---

# Palace Audit: YYYY-MM-DD

## Scorecard
| Dimension | Score | Notes |
|---|---|---|
| Staleness | X/5 | |
| Duplication | X/5 | |
| Broken refs | X/5 | |
| Coverage gaps | X/5 | |
| Architectural drift | X/5 | |
| Tracker integrity | X/5 | |
| **Total** | **XX/30** | |

## Findings
| File | Issue | Severity |
|---|---|---|
| path/to/file | description | 🔴/🟡/🟢 |

## Actions
1. [specific action] → [file path]

## Verdict
[HEALTHY / NEEDS_ATTENTION / DRIFT_DETECTED]: [one paragraph]
```

---

*Process by Loci · v1 · [loci.garden](https://loci.garden)*
