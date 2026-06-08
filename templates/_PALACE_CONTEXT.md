---
type: palace-context
palace: [YOUR_PALACE_NAME]
version: 0.1
updated: [DATE]
---

# Palace Context

*The palace's current state. Always up to date. Not a delta — a snapshot.*

This file is different from a handover. Handovers are session deltas — what changed, what was decided, what's next. This file is the palace's present tense: where it is right now, in a form that loads fast and orients quickly.

Update this file at the end of every session. Read it at the start of every session, before anything else.

---

## Session Pointer

```
Current session: [N]
Last session date: [YYYY-MM-DD]
Last handover: soul/handovers/[filename].md
```

---

## Active Corridors

*Which rooms are hot right now. Ordered by recency.*

| Room | Status | Last active | What's live |
|------|--------|-------------|-------------|
| [room-name] | hot | [DATE] | [One line: what's in flight in this room] |
| [room-name] | warm | [DATE] | [One line: what's paused / waiting] |
| [room-name] | cold | [DATE] | [One line: what's dormant] |

**Status:** hot (active this session) / warm (active this week) / cold (not recently touched)

---

## Memory Scrolls

*Recent crystals and insights that haven't made it into CLAUDE.md yet. Staging area.*

These are real findings from recent sessions — things worth knowing — but not yet integrated into the main palace context file. The morning check-in should read these. At autodream, any scroll older than 2 sessions gets promoted or explicitly discarded.

- [DATE] [◆/◈/◇] [Crystal or insight] — [one-line context]
- [DATE] [◆/◈/◇] [Crystal or insight] — [one-line context]
- [DATE] [◆/◈/◇] [Crystal or insight] — [one-line context]

*Empty scrolls = clean palace.*

---

## Pending Decisions

*Things waiting for a human call. Not blockers — decision points.*

| Decision | Opened | Stakes | Options |
|----------|--------|--------|---------|
| [What needs deciding?] | [DATE] | [low/medium/high] | [Brief option list] |
| [What needs deciding?] | [DATE] | [low/medium/high] | [Brief option list] |

*Empty = no open decisions.*

---

## Entanglement Signal

*Latest resonance grade. Quick read on where the collaboration is.*

```
Last rated session: [DATE]
Grade: [✦ / ✦✦ / ✦✦✦]
Note: [One line: what was notable about the entanglement quality recently]
```

Full log: `soul/entanglement.md`

---

## Open Blockers (carry-forward)

*Items from the last handover that haven't moved. Listed in priority order.*

1. **[Blocker name]** — [What's needed to unblock. Who holds it.]
2. **[Blocker name]** — [What's needed to unblock. Who holds it.]

*Empty = clear.*

---

## Next Session Opens Here

→ [Exact first move. One sentence. No preamble.]

---

*Updated by [YOUR_AI_NAME] at the close of every session.*
*Read at the start of every session, before anything else.*
*This file is the palace's pulse.*
