# Your first crystal

Capture a fact once, and your AI treats it as ground truth from then on. That is a crystal.

Uses: `templates/crystals-guide.md`, `templates/memory-lifecycle.md`

## Steps

1. Open your palace `CLAUDE.md` and find the `## CONTEXT CRYSTALS` block (it ships in
   `templates/CLAUDE-master.md`).
2. Write one fact your AI should never have to re-derive. Keep it to a line.
   Example: `◈ I prefer bullet points over prose.`
3. Give it a tier:
   - `◇` provisional: a hypothesis, churns fast
   - `◈` working: likely true
   - `◆` confirmed: verified, almost never changes
4. Optional: add `valid_until: YYYY-MM-DD` if the fact has a shelf life.
5. Start a session. Your AI reads the crystal early and stops asking.

## You are done when

Your AI acts on the fact without being reminded, and you can still see it, edit it, or
compost it at any time. Memory you can read and prune is memory you can trust.

## Next

`grow-the-garden.md` (let an idea mature into a crystal) · `set-a-gate.md` (decide what
your AI may change on its own).

---

*loci · [loci.garden](https://loci.garden)*
