# Close and reopen

The hardest part of working with AI is starting over every time. A handover fixes that:
you close a session on a clear note, and the next one opens warm.

Uses: `templates/handover-template.md`, `templates/synthesis-automation.md`,
`templates/scheduled-task-template.md`

## Steps

1. At the end of a working unit, write a delta to `soul/handovers/YYYY-MM-DD.md` (the
   format ships in `CLAUDE-master.md`): state, last decisions, open blockers, crystals added.
2. Write the one line that matters most: `Next action: session opens here`, then the exact
   first move with no preamble.
3. Update `tracker.json` track statuses so the map matches the territory.
4. Optional: run a synthesis pass (`synthesis-automation.md`). It proposes connections and
   patterns across sessions; `auto_apply` is false, so you confirm what sticks.
5. Optional: schedule housekeeping (`scheduled-task-template.md`) so a weekly pass runs
   without you. Never hardcode session paths; the template finds them dynamically.

## You are done when

Your next session opens on the exact next move, warm, with no re-briefing. The handover did
the remembering so you did not have to.

## Next

`set-a-gate.md` (carry your trust settings forward) · `build-a-room.md` (hand off room by
room).

---

*loci · [loci.garden](https://loci.garden)*
