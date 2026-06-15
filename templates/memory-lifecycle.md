---
created: [DATE]
version: 0.1
type: guide
---

# The Memory Lifecycle

*A palace fills, decays, and stays readable. Four principles for memory that keeps its signal, not just its volume.*

Most note systems only grow. They fill until search stops working and the oldest, truest facts are buried under the newest noise. A palace treats memory as a living store with a full lifecycle: things enter when they earn it, fade when they stop mattering, and stay inspectable the whole way through. These four principles state that lifecycle. The mechanics live in `crystals-guide.md` (the tiers) and `garden-health-template.md` (the surfacing pass); this page is the doctrine they serve.

---

## 1. Keep what surprised you

Not everything said in a session deserves a crystal. The cheap rule is "write it all down." The better rule is "write down what's new": the thing your prior context would not already have predicted.

Before you crystallise something, ask: would the palace, as it already stands, have known this? If the answer is yes, you are about to store a restatement, not a fact. Fold it into the crystal that already covers it instead. If the answer is no, the session genuinely surprised you, and that is exactly what earns a slot.

This keeps memory legible. A store where only the surprising earns a place stays small enough to read. A store where everything earns a place becomes a second inbox.

---

## 2. Let the rest decay

Forgetting is maintenance, not loss. A crystal that has stopped being true, or stopped being used, is not a failure of the system. Retiring it is the system working.

Three first-class ways things fade:

- **Expiry.** `valid_until` on ◈ and ◇ crystals. When the date passes, the fact is flagged for review, never silently dropped.
- **Demotion.** A ◆ that turns out to be phase-dependent moves down to ◈, with a note on why. Understanding evolving is worth recording.
- **Composting.** Retirement at any tier. Strike it through, add the reason and the date. The history tells the story of how you learned.

---

## 3. Each tier on its own clock

The three tiers do not age at the same rate, so do not review them at the same rate.

- **◇ Exploratory** turns over fast. Hypotheses are meant to resolve, promoted or retired, within a short window. A ◇ that has sat untouched for a long time is the normal case to revisit.
- **◈ Contextual** turns over more slowly, on the rhythm of the sprint, contract, or phase it is bound to.
- **◆ Confirmed** almost never moves. A ◆ going stale is rare enough that when it happens, it is a real signal, not routine churn.

Match the review cadence to the tier. One clock for all of memory either nags you about doctrine or lets hypotheses calcify. Pick your own intervals; the principle is only that they differ.

---

## 4. Keep all of it inspectable, and prunable by you

Memory you can read and prune is memory you can trust. This is the privacy axis, and it is the whole point of keeping memory in plain files you own.

- **Provenance.** Any crystal can show why it exists, when it landed, and which tier it holds. Nothing in the store is anonymous to you.
- **Gated, reversible deletion.** The palace composts rather than deletes: a move with a note, not a hard delete. It is reversible, and it never happens without you asking for it.
- **No silent edits.** The agent surfaces candidates (stale, redundant, overdue) and you make the call. A machine can flag structure; it must not adjudicate meaning.

A memory store you cannot read, or cannot prune, is not one you control. The plain-text palace is the bet that you should be able to do both, at any time, by hand.

---

*A palace you cannot read is just a bigger inbox. Keep it small, keep it yours.*
