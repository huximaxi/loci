---
created: [DATE]
version: 0.1
type: guide
---

# Individual Garden Files — Guide and Template

*Instead of (or alongside) a single `garden.md`, each plant gets its own numbered session files.*

A garden grows across sessions. A single `garden.md` captures the whole garden — but over time it becomes a dense record where individual plants get harder to trace. Individual garden files give each plant its own archaeology: a clean git history, no merge conflicts, and a timeline you can actually read.

---

## File Format

Garden files live at:
```
soul/garden/[plant-name]-001.md
soul/garden/[plant-name]-002.md
soul/garden/[plant-name]-003.md
...
```

Each file = one watering entry. One session, one file. The number is the entry count for that plant.

---

## The Template

Create a new file for each watering, using the format below.

**File path:** `soul/garden/[plant-name]-[NNN].md`

```markdown
---
plant: [Plant name]
entry: [NNN]
date: [YYYY-MM-DD]
topic: [One-line topic of this watering]
watered-by: [YOUR_AI_NAME]
follows: [plant-name]-[previous NNN, or "seed" if this is the first entry]
---

# [Plant name] — Entry [NNN]

*[DATE]*

[Free-form watering content. One to three paragraphs. What was explored, what shifted, what question emerged. Keep it grounded — this is not a summary of everything, it's the specific thing that grew today.]*

---

**Seed from last watering:** [Quote or paraphrase the core thread you're continuing from]

**Today's observation:** [What's new. The actual growth.]

**Next question:** [The question this watering opens. This is what the next watering picks up.]

---
*Growth direction: [One word or phrase — converging / forking / dormant / becoming-crystal]*
```

---

## Example — First Entry (Seed)

**File:** `soul/garden/your-plant-001.md`

```markdown
---
plant: Your Plant
entry: 001
date: 2026-03-15
topic: The core question this plant is exploring
watered-by: [YOUR_AI_NAME]
follows: seed
---

# Your Plant — Entry 001

*2026-03-15*

This is the first entry. Write the question that seeded the plant and the first real thought about it. The opening entry should capture what made this worth tracking: the itch, the tension, the thing you keep coming back to.

A plant starts as a question, not an answer. The early entries are exploratory. You don't need to resolve anything yet — you need to state the problem clearly enough that future-you can pick it up.

---

**Seed from last watering:** (first entry — no prior watering)

**Today's observation:** State the one thing this session clarified. A plant grows by accumulating observations, not conclusions.

**Next question:** What would you need to learn next to move this forward?

---
*Growth direction: converging*
```

---

## Example — Later Entry

**File:** `soul/garden/your-plant-004.md`

```markdown
---
plant: Your Plant
entry: 004
date: 2026-04-02
topic: A sharper sub-question that emerged as the plant grew
watered-by: [YOUR_AI_NAME]
follows: your-plant-003
---

# Your Plant — Entry 004

*2026-04-02*

By the fourth entry, the plant has shape. This entry builds on what came before — note how it references the prior watering rather than starting fresh. The thinking has moved from "what is this?" to "what specifically follows from it?"

A later entry often surfaces a tradeoff: the thing you understood in entry 001 turns out to have a cost, a limit, or a dependency you didn't see at first. Naming that tradeoff is growth.

---

**Seed from last watering:** Entry 003 left off with a partial answer. Today I wanted to push on the part that still felt unresolved.

**Today's observation:** State the refinement. The plant deepens when each entry adds a constraint, a counter-example, or a connection the last one missed.

**Next question:** What's the next edge to test? Where might this break?

---
*Growth direction: converging*
```

---

## Benefits of Individual Files

**Git history per plant.** `git log soul/garden/your-plant-*` shows every watering in order, with diffs. You can see exactly what was added each time.

**No merge conflicts.** Multiple plants can be watered in the same session without editing the same file. Each watering creates a new file.

**Cleaner archaeology.** Want to understand how a plant evolved? Read the files in order. The numbered sequence is the story.

**Easier promotion.** When a plant becomes a crystal, the full record is already there. You can trace the exact watering where the insight crystallised.

---

## Alongside `garden.md`

Individual garden files can coexist with `garden.md`. The two play different roles:

| `soul/garden.md` | `soul/garden/[plant-name]-NNN.md` |
|------------------|-----------------------------------|
| Overview: all plants, current state | Deep record: full watering history per plant |
| Quick reference for morning check-ins | Archaeological record for garden rounds |
| What's active, what's dormant | How each plant grew over time |

The agent keeps both in sync: when a watering is added as an individual file, the plant's entry in `garden.md` is updated with the latest summary.

---

## Naming Convention

```
[plant-name]-[NNN].md

plant-name: lowercase, hyphenated (same as plant's name in garden.md)
NNN: zero-padded three-digit number (001, 002, ..., 099, 100)
```

**Examples:**
```
your-plant-001.md
another-plant-003.md
third-plant-007.md
fourth-plant-012.md
```

---

*The garden remembers. The numbered files are the memory.*
*Each entry is a moment of growth, dated and preserved.*
