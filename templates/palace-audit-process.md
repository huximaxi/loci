# Palace Audit — Structural Autodream Process

> Agent-executable process. Trigger: "palace audit" / "structural autodream" / "check our architecture" / "is our file structure healthy"

---

## What this does

Scans the full palace setup — CLAUDE.md files, soul files, skills, handover chain — for staleness, duplication, broken references, coverage gaps, and architectural drift. Scores health across 5 dimensions and outputs a report.

Run after major sprints, when architecture feels heavy, or when CLAUDE.md files haven't been reviewed in more than 3 weeks.

---

## 5 Audit Dimensions

| # | Dimension | What it checks |
|---|---|---|
| 1 | **Staleness** | CLAUDE.md files not updated in > 3 sessions; handovers with open blockers already resolved |
| 2 | **Duplication** | Same information in both global layer and project CLAUDE.md; persona soul files duplicated |
| 3 | **Broken refs** | File paths cited in CLAUDE.md / handovers that no longer exist |
| 4 | **Coverage gaps** | Active projects with no handover; rooms with no CLAUDE.md; skills listed but missing |
| 5 | **Architectural drift** | Behavioral constants in project CLAUDE.md (should be global layer); state in global CLAUDE.md (should be project layer) |

Score: 0–5 per dimension. Total: /25.

---

## Protocol

```
TRIGGER
   │
   ▼
Step 1: SCAN (shell — no LLM reads yet)
   ├── List all CLAUDE.md files
   ├── List all soul files
   ├── List all skill files + check against any skills index
   ├── List handovers by date
   └── Check file refs exist (bash test -f on each path cited)
   │
   ▼
Step 2: DIFF DETECTION (read only flagged files)
   ├── Read CLAUDE.md files flagged as stale
   ├── Compare skills index vs actual skill files
   ├── Compare pending blockers in project CLAUDE.md vs latest handover
   └── Verify global layer workspace paths are current
   │
   ▼
Step 3: SCORE + REPORT
   ├── 5-dimension scorecard (0–5 each, /25 total)
   ├── Findings: file | issue | severity (🔴 blocking / 🟡 advisory / 🟢 healthy)
   ├── Specific actions: "update X", "delete Y", "add Z to index"
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

Reports go to `[palace]/audits/YYYY-MM-DD.md`. They accumulate — the audit history is itself an architectural signal.

```markdown
---
type: palace-audit
date: YYYY-MM-DD
score: XX/25
verdict: HEALTHY | NEEDS_ATTENTION | DRIFT_DETECTED
---

# Palace Audit — YYYY-MM-DD

## Scorecard
| Dimension | Score | Notes |
|---|---|---|
| Staleness | X/5 | |
| Duplication | X/5 | |
| Broken refs | X/5 | |
| Coverage gaps | X/5 | |
| Architectural drift | X/5 | |
| **Total** | **XX/25** | |

## Findings
| File | Issue | Severity |
|---|---|---|
| path/to/file | description | 🔴/🟡/🟢 |

## Actions
1. [specific action] → [file path]

## Verdict
[HEALTHY / NEEDS_ATTENTION / DRIFT_DETECTED] — [one paragraph]
```

---

*Process by Loci · v1 · [loci.garden](https://loci.garden)*
