---
type: skill
name: rain
owner_posture: orchestration
status: draft (first run 2026-07-02)
---

# Rain

*A mass watering round for the idea garden: every persona at once, each
picking its own learning. Elastic, valuable, interruptible, which makes it
the natural spender for session-window headroom that would otherwise expire.*

---

## What it does

Summons the palace's full persona roster in parallel. Each persona reads its
own soul file, walks the garden's plants, and picks the ONE plant that speaks
to something it has actually learned or been circling lately, then waters it
with a dated, grounded entry (or seeds a new plant if something real is
germinating). All waterings come back as content; exactly one serial applier
lands them on disk. No agent writes to a bed directly.

The failure it prevents: gardens rot when watering depends on one voice
remembering to tend them. A rain round is the whole roster's attention
passing over the beds at once, and convergence (two personas independently
picking the same plant) is itself a signal a single tender can never produce.

## When to apply

- On explicit call ("rain", "watering round").
- When the runtime's token watcher reports session-window headroom: a fresh
  5-hour window just opened, or spare capacity is about to expire with a
  closing window. Rain is the designated spender because skipping a round
  costs nothing and running one always leaves durable artifacts.
- Never automatically. Weather is a suggestion; spending is a gate. A human
  nod fires the round.

One wiring note: the `--fire` hand-off sends the literal word `rain` as the
prompt to your agent runtime. Your palace's orchestrator instructions
(CLAUDE.md or equivalent) must define that trigger and route it to this
procedure; without it the word arrives bare, and the politest thing your
runtime can do with bare weather is ask about it.

## The procedure

1. **Scout.** Read the persona roster (name + soul file per persona) and the
   plant registry. Resolve today's date. Everyone gets the same garden map.
2. **Rain.** One agent per persona, in parallel. Summon, don't assign: address
   each by name, point them at their own soul, and let them choose their
   plant. Waterings are 100-200 words, concrete learning tied to the plant's
   mechanic, cross-references only where real. Tier promotions need one-line
   justification. Agents return content; they never write files.
3. **Fall.** One applier lands all waterings serially: append each entry
   under the plant's watering log, update its last-watered date, advance a
   tier glyph at most once per plant per round (a second persona proposing
   the same promotion stays recorded in-entry, not double-applied). Seeds
   whose slug collides with an existing plant land as waterings instead.
   Archive the round's raw waterings JSON beside the garden.
4. **Report.** By bed: what was watered, promoted, seeded, and where personas
   converged. One line per watering, in the personas' own words.

## Kill conditions

- No garden, or a roster of one: a rain round is overhead; water directly.
- Findings that name specifics (counts, locations, quotes) get verified
  against the source before the report repeats them; swarm prose can
  confabulate.
- If the applier cannot verify every entry on disk afterward, say so
  plainly; a green apply-log is not the same as watered plants.
