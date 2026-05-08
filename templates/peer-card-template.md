---
created: [DATE]
version: 0.1
type: template
template: peer-card
---

# Peer Card Template

*A structured, human-authored representation of the person the palace is built for. Unlike auto-extracted user profiles, the peer card is written with intention - it is who you say you are, how you work, and what the AI should know to work with you well.*

The peer card lives at the palace root as `_USER.md`. It is the first thing a new AI reads about you - before context files, before handovers, before the crystal table.

---

## How This Differs From a User Profile

Automated user modeling tools (Honcho, Mem0, etc.) *extract* a profile from what you say - they infer your preferences from your communication patterns. That produces useful signal but flat facts.

The peer card is *authored*. You write it. You decide what matters. The AI reads it as a declaration, not a deduction. This makes it more accurate (you know yourself better than statistical inference does), more intentional (you surface what you want the AI to know), and more durable (it doesn't drift toward recency bias).

Update it when something genuinely shifts - not because you had a bad day.

---

## The Template

```markdown
---
created: [DATE]
type: peer-card
version: 0.1
---

# [YOUR_NAME] - Peer Card

*Human-authored. Updated intentionally. Last revised: [DATE]*

---

## Identity

**Name:** [Your name / preferred name]
**Role:** [Current role / title / what you do]
**Domain:** [What field or space you operate in]
**Location / Timezone:** [Where you are]
**Contact:** [Optional - email, handle, whatever's useful]

---

## How I Work

**Pace:** [Fast / Considered / Variable - with any relevant context]
**Communication register:** [How you like responses - concise, detailed, technical, plain-language]
**Trust signals:** [What signals to you that the AI understands the task? e.g. "When it surfaces trade-offs without being asked"]
**Friction signals:** [What slows you down or frustrates you? e.g. "Over-explanation of things I already know. Hedging."]
**Mode phrases:** [Any shorthand you use to signal working mode - e.g. "LFG frne = sprint mode, skip clarification"]

---

## What I Know

**Strong domains:** [Where you have deep expertise - the AI shouldn't over-explain here]
**Active learning:** [Where you're developing - the AI can go deeper here than you'd expect]
**Working assumptions:** [Things the AI can take for granted when you're working together]

---

## What I Care About

*(3–5 things that genuinely matter to you in your work. Not a job description. The things you'd be annoyed if ignored.)*

1. **[VALUE/PRINCIPLE]:** [Why it matters. What it looks like in practice.]
2. **[VALUE/PRINCIPLE]:** [Same.]
3. [Add more as needed]

---

## Constraints and Non-Negotiables

*(Things the AI should treat as fixed, not open to creative reinterpretation.)*

- [e.g. "Never push to production without explicit sign-off"]
- [e.g. "Privacy by default - every data flow gets evaluated"]
- [e.g. "KISS always wins over clever"]

---

## Working History (brief)

*(Optional. 2–3 sentences on what you've built / worked on that's most relevant to the palace. Not a CV - context for the AI.)*

[Write a brief summary here]

---

## Update Log

*(When something material shifts, note it here. This card should reflect who you actually are now, not who you were at setup.)*

| Date | What changed | Why |
|------|-------------|-----|
| [DATE] | [What] | [Why it shifted] |
```

---

## Usage Notes

**At session start:** The AI reads `_USER.md` before the crystal table and before room context files. Think of it as the AI's briefing on you - everything else builds on it.

**Updating:** Update when your role changes, your priorities shift, or you notice the AI making systematic wrong assumptions about you. Don't update after every session - that's what crystals are for.

**Sharing:** If another person in the palace shares their soul file with you (collaborative mode), add their peer card to `_palace/friends/[name]-USER.md`. Their card is read-only in your palace.

**With Observation Scope:** The peer card tells the AI what you want it to know. The Observation Scope in your soul file tells the AI what it should actively track. Together they define the full picture of intentional memory.

---

*Who you are is a working document. The peer card is the one that faces the palace.*
