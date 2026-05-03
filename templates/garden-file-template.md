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

**File:** `soul/garden/invisible-proof-001.md`

```markdown
---
plant: Invisible Proof
entry: 001
date: 2026-03-15
topic: What does zero-knowledge actually mean in practice?
watered-by: Vesper
follows: seed
---

# Invisible Proof — Entry 001

*2026-03-15*

Zero-knowledge proofs feel like they should be impossible — you convince someone of a fact without revealing the fact itself. The first encounter with ZK is always a little uncanny. How can mathematics produce something that feels like magic?

The key is that proof and knowledge are separable. You can demonstrate that you know a secret path through a maze without ever showing the path. The verifier gains certainty about your knowledge, not the knowledge itself.

---

**Seed from last watering:** (first entry — no prior watering)

**Today's observation:** ZK is not about hiding data. It's about proving *properties* of data. The distinction matters: the goal is not concealment, it's selective disclosure.

**Next question:** What breaks this? Where does the mathematical guarantee end and implementation risk begin?

---
*Growth direction: converging*
```

---

## Example — Later Entry

**File:** `soul/garden/invisible-proof-004.md`

```markdown
---
plant: Invisible Proof
entry: 004
date: 2026-04-02
topic: The relationship between proof size and verification time
watered-by: Vesper
follows: invisible-proof-003
---

# Invisible Proof — Entry 004

*2026-04-02*

Groth16 produces constant-size proofs regardless of circuit complexity. That's the practical miracle: the proof doesn't grow with the statement being proved. The verifier always does roughly the same amount of work.

The tradeoff is the trusted setup. Groth16 requires ceremony parameters — a structured reference string generated in a multi-party computation. If the ceremony is compromised, fake proofs become possible. So the trust doesn't disappear; it moves from "trust the data" to "trust the ceremony."

---

**Seed from last watering:** We were asking whether recursion (proving a proof) is practical. Entry 003 said: yes, and it's how you scale. Today I wanted to go back to why the base-layer proofs are so efficient.

**Today's observation:** Efficiency at Groth16 level is a consequence of the algebraic structure — pairing-based cryptography lets verification collapse to a fixed number of operations. The cost is at proving time (not verifying), and in the trusted setup dependency.

**Next question:** If the setup ceremony can be compromised, what's the actual attack surface in practice? Has any Groth16 ceremony been successfully attacked?

---
*Growth direction: converging*
```

---

## Benefits of Individual Files

**Git history per plant.** `git log soul/garden/invisible-proof-*` shows every watering in order, with diffs. You can see exactly what was added each time.

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
invisible-proof-001.md
alexander-garden-003.md
political-voice-007.md
trust-xyz-012.md
```

---

*The garden remembers. The numbered files are the memory.*
*Each entry is a moment of growth, dated and preserved.*
