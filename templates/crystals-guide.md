---
created: [DATE]
version: 0.1
type: guide
---

# Crystal Tiers — Guide

*How facts live in the palace. What changes. What doesn't. How to tell the difference.*

Crystals are the palace's memory system — established truths about you, your work, your world. But not all truths are equally stable. The three-tier system captures that difference explicitly, so the palace knows what to trust, what to revisit, and what to treat as a working hypothesis.

---

## The Three Tiers

### ◆ Confirmed (solid diamond)

**What it is:** Foundational facts. Core to who you are, what you do, how you work. Rarely changes. If it does change, it's significant — and worth noting explicitly.

**Examples:**
- Your name, your role, your organisation
- The tech stack you build on
- Your privacy model or core values
- Who your AI collaborator is

**Rule:** Treat ◆ crystals as ground truth. No need to re-verify every session. If you discover a ◆ crystal is wrong, update it and note why it changed.

**Format:**
```
◆ [Crystal name]: [value]
```

**Example entries:**
```
◆ Owner: Hux — Head of UX/Product, Nym Technologies SA
◆ Stack: Next.js 14 / Strapi v4 / Vercel
◆ Privacy model: mixnet + zk-nym credentials — metadata-resistant
◆ Core value: Simplicity first. No over-engineering.
```

---

### ◈ Contextual (diamond with dot)

**What it is:** True now, but may shift. Contextual facts tied to the current phase, sprint, contract, or situation. Should be reviewed periodically. Use `valid_until` when the expiry is predictable.

**Examples:**
- Current sprint goal
- Active vendor contract
- This quarter's roadmap priority
- A team member's current role (if in flux)
- A working approach you're trying for a while

**Rule:** When a ◈ crystal reaches its `valid_until` date, flag it for review. Don't auto-delete — review and either confirm, update, or retire.

**Format:**
```
◈ [Crystal name]: [value]
valid_until: [YYYY-MM-DD or event]
```

**Example entries:**
```
◈ Current sprint: Censorship circumvention features (Airporting toggle + Client Config Access)
valid_until: 2026-05-30

◈ Vendor: Stripe for payment processing — active contract
valid_until: 2026-12-31

◈ Working approach: Daily 9am check-in while this feature is in flight
valid_until: sprint-end
```

---

### ◇ Exploratory (hollow diamond)

**What it is:** Hypotheses, early findings, working assumptions. Not yet confirmed. Should be promoted to ◈ or ◆ once validated, or retired with a note if they don't hold.

**Examples:**
- A working theory about user behaviour
- An architectural decision you're testing
- A framing you're trying out
- Something you believe but haven't proven

**Rule:** Exploratory crystals are not disposable — they're the palace's research in progress. Name them clearly. When you learn more, promote or retire them explicitly.

**Format:**
```
◇ [Crystal name]: [hypothesis]
valid_until: [optional — when you expect to know more]
```

**Example entries:**
```
◇ User behaviour: Privacy-conscious normies don't toggle advanced settings — they trust defaults
valid_until: 2026-Q2-research

◇ Architecture: Moving to edge functions for the auth flow will reduce latency enough to matter
valid_until: after-spike

◇ Framing: "Speed mode" lands better than "2-hop mode" for casual users
```

---

## Adding Crystals

During a session, add crystals as you learn them. You don't have to wait for a formal review.

**Quick format** (inline in CLAUDE.md or room files):
```
◆ [name]: [value]
◈ [name]: [value] — valid_until: [date or event]
◇ [name]: [value]
```

**Agent rule:** When a crystal is established in conversation, write it to the appropriate file immediately — don't leave it in chat. Crystals live in `CLAUDE.md` (palace-wide) or in the relevant room's `CLAUDE.md` (room-specific).

---

## Promoting Between Tiers

Promotion is explicit. Don't silently upgrade a crystal — note what changed.

```
◇ Framing: "Speed mode" lands better than "2-hop mode"
→ Promoted to ◈ 2026-04-15: confirmed by 3 user interviews. Still A/B testing.
→ Promoted to ◆ 2026-05-01: A/B concluded — "Speed mode" wins across all segments.
```

**Promotion criteria:**
- ◇ → ◈: hypothesis confirmed once, or corroborated by evidence
- ◈ → ◆: stable across multiple sessions/contexts, no longer contextually bound

Demotion also happens. If a ◆ crystal turns out to be false or phase-dependent, move it down and add a note. The history matters.

---

## `valid_until` Usage

`valid_until` is an optional field on ◈ and ◇ crystals. Use it when:
- The fact is tied to a specific date (contract end, sprint close, deadline)
- The fact is tied to an event (after the feature ships, after the research round)
- You know roughly when you'll learn more

**Format options:**
```
valid_until: 2026-06-30          # specific date
valid_until: sprint-end          # event-based
valid_until: 2026-Q3             # quarter
valid_until: after-user-research # milestone
```

**Morning check-in surfacing:** The `morning-check-in` process scans for crystals whose `valid_until` is within 7 days and flags them for review. This is the palace's built-in insight decay mechanism — facts don't go stale silently.

---

## Crystal Placement

| Where | What goes there |
|-------|----------------|
| `CLAUDE.md` (root) | Palace-wide facts — identity, stack, values, global working style |
| `rooms/[room]/CLAUDE.md` | Room-specific facts — project state, room context, relevant crystals for that mode |
| `soul/SOUL.md` | Identity-level facts — who you are, what you care about, the soul's core |

Crystals can be duplicated across files if they're relevant in multiple contexts. The palace is spatial — the same truth can live in multiple rooms.

---

## Example — Full Crystal Block

```markdown
## Crystals

◆ Owner: [Name] — [Role], [Organisation]
◆ Stack: [Tech stack]
◆ Privacy stance: [One line on how they think about privacy]

◈ Current focus: [Sprint or project goal]
valid_until: [DATE or EVENT]

◈ Working approach: [How they're currently working — could change]
valid_until: [DATE or EVENT]

◇ Hypothesis: [Something believed but not yet confirmed]
◇ Experiment: [Something being tested]
```

---

*Three tiers. One principle: know what you know, know what you don't, and never let the two blur.*
