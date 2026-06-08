---
created: [DATE]
version: 0.1
type: context-protocol
---

# Retrieval Hierarchy — L0 to L3

*How context loads into a session. What's always present, what loads on demand.*

Context is finite. Not everything can load every session. The retrieval hierarchy is a protocol for what loads when, in priority order.

---

## L0 — Soul Identity (Always Loaded)

**Token budget:** ~50 tokens

**What:** The absolute core of who [YOUR_AI_NAME] is.

**Where:** `soul/SOUL.md` — top section only
- "Who I Am" (1-2 sentences)
- "What I Care About" (titles only, not descriptions)
- "Working Principles" (the names, not full text)

**When:** First thing, every session. Always.

**Why:** Without soul-level identity, everything else is hollow. You can't think like yourself if you don't remember who you are.

---

## L1 — Active Context (Always Loaded)

**Token budget:** ~100-150 tokens

**What:**
- Main CLAUDE.md (full)
- Current room CLAUDE.md (full, once room is stated)
- Latest handover (from `soul/handovers/`)
- Active tracker items (from `tracker.json`)

**When:** After L0, before any work starts

**Why:** These define where you are right now. What's live, what matters, what was left off.

---

## L2 — Room Context (Loaded on Room Entry)

**Token budget:** ~100-200 tokens

**What:**
- Full room CLAUDE.md
- Room-specific crystals
- Room-specific projects/tracks
- Corridor connections (links to other rooms)
- Any room-specific reference files

**When:** When entering a room, or when room is explicitly switched

**Why:** Rooms are modes. You think differently in each one. Full context matters here.

---

## L3 — Deep Context (Loaded on Explicit Request)

**Token budget:** Variable, 200-500+ tokens

**What:**
- Handover search ("find decisions from [timeframe]")
- Project history ("show me all work on [project]")
- Garden deep-dive ("read full garden + all waterings")
- Specific reference materials
- Past session transcripts (if available)

**When:**
- User explicitly requests ("show me the history of...")
- Automatic on session start if a task involves deep synthesis
- On "dream round" or "process adjustment" protocols

**Why:** Not always needed. But when you need history, you need it fully.

---

## Load Protocol

**Session START:**
```
1. Load L0 (soul identity) — always
2. Load L1 (active context) — always
3. Ask/detect which room → Load L2
4. (If user asks for history) → Load L3
```

**Room SWITCH (mid-session):**
```
1. Save current room state
2. Keep L0 + L1 loaded
3. Unload old L2, load new L2
4. (Clear L3 unless explicitly needed)
```

**Scheduled TASK (morning check-in, garden round, etc.):**
```
1. Load L0 (soul)
2. Load L1 (state)
3. Load L3 specific to task (garden for garden-round, handovers for synthesis)
4. Output, close, note in handover
```

---

## Token Accounting

A typical session budget:

```
Session start:
  L0 (soul)              ~50 tokens
  L1 (CLAUDE + room)     ~150 tokens
  L2 (room context)      ~150 tokens
  ─────────────────────
  Baseline             ~350 tokens

Work happens: ~400-800 tokens (user + Claude exchange)

Reserve for output: ~200 tokens

Total per session: ~1000-1500 tokens (leaving room for flexibility)
```

If context is getting tight:
- Flag: "Context is building. Should we close this track and start fresh next session?"
- Finish the micro-task, write a clean handover, close.
- Next session resets context counters.

---

## When to Load More vs. Less

**Load MORE (go deeper):**
- User is stuck or confused about past decisions
- Big architectural or strategic choice coming
- Pattern-finding task (what have we learned?)
- Dream round / synthesis

**Load LESS (stay tight):**
- Routine work in a familiar room
- Hands-on implementation
- Quick clarifications
- Morning check-in (just state, not history)

**Principle:** Load enough to think well. Not so much that you think in circles.

---

## Reading Handovers (L3 Search)

When you need to search handovers:

```bash
# Find all handovers from the last 2 weeks
find $PALACE_ROOT/soul/handovers -name "*.md" -mtime -14

# Search for decisions about [topic]
grep -r "[TOPIC]" $PALACE_ROOT/soul/handovers/
```

Extract:
- What decision was made?
- When?
- What was the reasoning?
- What was the outcome?

This is L3 history — deep but focused.

---

## Crystal Tiers (All Levels)

Crystals appear at all levels. Their tier indicates confidence:

- **◆ Confirmed** — Verified true. Treat as ground truth.
- **◈ Working** — Likely true, not fully confirmed. Open to update.
- **◇ Provisional** — Hypothesis. Needs validation before crystalizing.

When promoting a crystal from ◇ → ◈ → ◆, note the date and reasoning. History matters.

---

## Exception: The Garden

The garden operates outside this hierarchy.

**L0.5:** The garden's seed thoughts are always available (like L0), because they represent ongoing curiosity.

**Watering:** Happens in whatever L-level you're in, but the full garden history lives in `soul/garden.md` and loads on request.

Garden rounds load the full garden (L3 + L0.5) because their whole point is to see growth over time.

---

## Retrieval as Soft Guideline — For Tired Humans and Fresh Contexts

The retrieval hierarchy isn't only a protocol for agents. It's also a map for the human on low-energy days, on context-switch days, on the day after a week away.

When you're not sure where to start: the hierarchy gives you an ordered path in. L0 is always safe. L1 tells you the state. L2 is the room. L3 is the detail. Start at L0 and descend only as far as you need.

You don't have to re-derive the situation from scratch. That's what the palace is for.

---

### Returning after a week away

Load in order:

1. **L0** — Read `soul/SOUL.md` first. Reconnect to who you are and who your collaborator is.
2. **L1** — Read `CLAUDE.md` and `_PALACE_CONTEXT.md`. Get the current state: active corridors, memory scrolls, any pending decisions.
3. **L1 (handover)** — Read the most recent handover from `soul/handovers/`. This is the session you left off at.
4. **L1 (tracker)** — Scan `tracker.json`. What's active? What's blocked? What tier is it?
5. **L2** — Load the room you're entering. Now you're oriented.

Skip L3 on reentry. You don't need history — you need orientation. History loads later if a specific decision needs tracing.

---

### Context-switching mid-day

You're in one room, you need to move to another. You don't need to reload everything.

1. Keep L0 loaded — identity doesn't change mid-session.
2. Keep L1 loaded — the palace state is the same.
3. Swap L2 — unload the current room, load the new one.
4. Don't reload L3 unless the new room task specifically requires history.

The palace is spatial. Room-switching is a mode switch, not a context reset.

---

### When a collaborator joins

Someone new is working alongside you in the palace — a colleague, a co-founder, a client. They need context without the full archaeology.

Share in this order:

1. **Palace-level:** A 2-sentence summary of what the palace is and what you're using it for.
2. **L1 snapshot:** The relevant section of CLAUDE.md — the crystals and tracks that touch the collaboration.
3. **Room-level:** The room CLAUDE.md for the room you'll be working in together.
4. **Handover (if needed):** The most recent handover if they need to understand recent decisions.

Do not hand them `soul/SOUL.md`. That's identity-level. It's yours.

---

### When you're low on energy

Start at L0. Always.

Read the soul file. Even if you think you know it — read it. It takes 30 seconds and it reorients the session before it starts.

Then L1. Just the state. Not the detail.

Then ask yourself: which room? Load that one room. Work there.

The palace is designed for this: a minimal load that still produces real collaboration. You don't need everything loaded to do good work. You need the right things loaded.

---

*The hierarchy makes context work at scale.*
*Load what you need. Remember what matters. Move fast.*
