---
type: skill
name: insight-consolidation
owner_posture: orchestration
status: stable
---

# Insight Consolidation

*Catch the durable learning before it fades. An insight re-derived next month
costs the same as the first time; an insight banked tonight costs one small
file.*

---

## What it does

Converts the non-obvious things a session figured out (a fix, a pattern, a
preference, a decision and its *why*) into small single-fact notes plus
one-line index pointers, so future sessions recall them instead of
re-deriving them. It is the write-side of a palace's memory: deliberately
narrow, surprise-gated, and prunable.

## When to apply

- When something non-obvious got figured out and would matter again weeks
  from now.
- When a decision was made whose *why* is not recoverable from the artifact
  itself.
- As step 3 of `session-close.md`, on everything the reflection pass judged
  durable.

Not for: things the repo already records (code structure, git history), or
things only true for today's task. The gate question is in the procedure.

## The procedure

1. **Ask what's durable.** Would this matter in a session three weeks from
   now? If it's only true for today's task, let it go. Surprise is the
   strongest signal: if nothing about it surprised you, it probably didn't
   need writing down.
2. **Write one fact per note.** A short file: a title, a one-line hook, the
   fact, and (for decisions and feedback) a "why" and a "how to apply". One
   idea, one file. A note holding three ideas will be recalled for none of
   them.
3. **Fold the log, keep the map.** Don't hoard raw transcript. Keep the
   *role* and *coverage* of what you learned; compress the rest. The index
   line is the recall surface, not the body.
4. **Link, don't duplicate.** Before adding, check for an existing note that
   already covers it and update that one instead. Point related notes at
   each other; a link to a note that doesn't exist yet marks something worth
   writing, not an error.
5. **Don't over-derive.** One coincidence is not a law. State it once, mark
   it provisional, and let a second occurrence promote it.

## Verification

- Each new note answers one question, and its index line alone is enough to
  decide whether to open it.
- No new note duplicates an existing one (you checked before writing).
- Wrong notes found along the way were deleted or corrected, not kept "just
  in case".

## Kill condition

If notes pile up but are never recalled, the index has become too noisy to
search and the discipline is feeding the problem. Consolidate the
consolidator: merge, prune, and shorten the index before adding anything
more. If the palace has no recall mechanism at all yet, build that first;
write-only memory is a diary, not memory.

---

## See also

- `templates/skills/session-close.md`: the discipline that invokes this one
  at every close
- `templates/memory-lifecycle.md`: the four lifecycle principles this
  write-side feeds (surprise-gated write, multi-rate review, tiered decay)
- `PALACE-METHODOLOGY.md`: the memory lifecycle doctrine
