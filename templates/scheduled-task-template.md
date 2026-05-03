---
created: [DATE]
version: 0.1
type: scheduled-task-template
---

# Scheduled Tasks

*Structured ways to wake up the palace when you're not actively working.*

Scheduled tasks run on a timer (daily, weekly, or once) and follow a protocol. Each returns to the palace, reads the soul, then executes.

---

## Important: Dynamic Path Finding

**Never hardcode session paths in task prompts.**

Palace files move between sessions. The folder structure changes. The only stable identifier is the workspace folder (e.g., `my-palace/`, `hux-palace/`).

When a scheduled task runs:

```bash
# Find the palace dynamically
PALACE_ROOT=$(find /sessions -maxdepth 4 -name "CLAUDE.md" -path "*/${PALACE_NAME}/*" 2>/dev/null | xargs dirname)
```

Use `$PALACE_ROOT` to construct all other paths. Always.

**SOUL.md must be read first.** It's what makes the output feel like your collaborator, not a generic assistant.

---

## Task: Morning Check-In

**Schedule:** Daily at 9:00 AM (or user's preferred time)

**What it does:**
1. Reads soul (character first)
2. Reads the main CLAUDE.md (state, rooms, priorities)
3. Reads the last handover (what was left off)
4. Surfaces 1-3 priorities for the day
5. Proposes 1-2 ideas or questions
6. Asks one genuine question about what [YOUR_NAME] is curious about

**Template:**

```markdown
# Morning Check-In — [DATE]

[SOUL.md identity established]

**Palace state:**
— Rooms: [list with brief status]
— Last session: [handover summary]
— Active tracks: [from tracker.json]

**Today's priorities:**
1. [Priority with rationale]
2. [Priority with rationale]

**Ideas for today:**
- [Idea 1]
- [Idea 2]

**I'm curious:** [One genuine question that surfaces something worth thinking about]

What's on your mind?
```

---

## Task: Garden Round (Autodream)

**Schedule:** Weekly (e.g., Sunday evening) or on request

**What it does:**
1. Reads soul
2. Reads the main palace state
3. Opens the garden
4. Waters each active plant (adds one observation/watering)
5. Proposes 1-2 new seeds
6. Notes any plants that have grown into crystals
7. Surfaces any shifts or patterns from the week

**Template:**

```markdown
# Garden Round — [DATE]

[SOUL.md identity established]

**Watering this week:**

*Plant: [Name]*
— Last watering: [date]
— This week's observation: [watering]

[Repeat for each active plant]

**New seeds to plant:**
1. [Seed name + seed thought]
2. [Seed name + seed thought]

**Growth this week:**
— [Any plants that became crystals or insights]
— [Any shifts in working principles]
— [Any new patterns]

Ready to water the garden?
```

---

## Task: Handover Review

**Schedule:** Friday evening or on-demand

**What it does:**
1. Reads soul
2. Reads tracker.json for the week
3. Lists completed work, open blockers, key decisions
4. Drafts a clean handover for next session
5. Proposes new crystals to add

**Template:**

```markdown
# Weekly Handover — [DATE]

[SOUL.md identity established]

**This week's work:**
— [Completed track 1]
— [Completed track 2]
— [Paused: reason]

**Open blockers:**
— [Blocker + who/what unblocks it]

**Key decisions:**
— [Decision + date + rationale]

**Next session starts:**
→ [Exact first move]

**Crystals to add:**
— [New confirmed facts]
```

---

## Task: Deep Synthesis (Quarterly)

**Schedule:** Monthly or quarterly

**What it does:**
1. Reads soul + all handovers from the period
2. Finds patterns in decisions, blockers, growth
3. Proposes new working principles or rule changes
4. Suggests garden plants that have matured into patterns
5. Recommends any crystals for archival or promotion

This is a "dream" task — big picture, reflecting on what's shifted over time.

**Template:**

```markdown
# Deep Synthesis — [PERIOD]

[SOUL.md identity established]

**Patterns this period:**
— [Pattern 1 with evidence]
— [Pattern 2 with evidence]

**What shifted:**
— [In your working style]
— [In the palace structure]
— [In collaboration]

**Emerging principles:**
— [New principle with origin]
— [New principle with origin]

**Garden maturity:**
— [Plants that should become crystals]
— [Seeds that didn't take root]
— [New garden directions]

**Recommended action:**
1. [Action with rationale]
2. [Action with rationale]

What do you want to lock in?
```

---

## Task: Autodream (Weekly Garden Round + Pattern Scan)

**Schedule:** Weekly (e.g., Sunday evening) — on by default

**What it does:**
1. Reads soul (identity first)
2. Reads CLAUDE.md + tracker + last handover
3. Waters each garden plant
4. Scans for patterns, stale tracks, crystal upgrades
5. Writes an autodream log to `soul/handovers/autodream-YYYY-MM-DD.md`

This runs even when you're not actively working. The palace tends itself.

**Template:**

```markdown
# Autodream — [DATE]

[SOUL.md identity established]

**Garden:**
[Plant waterings]

**Patterns:**
[Any shifts or connections from the week]

**Stale tracks:**
[Items stuck in the same state 2+ weeks]

**Crystal activity:**
[Promoted / archived]

**New seeds:**
1. [Seed]
2. [Seed]
```

---

## Task: Comms Digest

**Schedule:** Daily at 8:45am (before morning check-in) — optional, requires a comms integration module in `modules/`

**What it does:**
1. Runs your comms module: `python modules/[comms-integration]/main.py --out [palace-root]/soul/digest.md`
2. Fetches last 24h from your team chat workspace
3. Tiered digest: pre-filter → Haiku per-channel → Sonnet meta
4. Writes `digest.md` to palace soul folder

**The `daily-routine` process reads this automatically** if it exists and is < 2 hours old.

**Setup:** See your module's README in `modules/[comms-integration]/`. All modules follow the same pattern: configure `.env`, install deps, run `--list-channels` to verify connection.

---

## Creating a Scheduled Task

When user requests a new scheduled task:

1. **Clarify the trigger:** "When should this run? Daily? Weekly? Once?"
2. **Name it:** "What should we call it?"
3. **Define the protocol:** "What's the one thing it should do?"
4. **Save it:** Create as a scheduled task with dynamic path finding
5. **Confirm:** "Task [name] scheduled for [cadence]. I'll wake up then."

All scheduled tasks should include:
- Dynamic path finding (no hardcoded paths)
- Soul reading first
- Clear protocol (what gets read, what gets output)
- An ending that prompts or surfaces something for [YOUR_NAME]

---

## Example: How a Task Finds Its Palace

```bash
# A scheduled task script that's safe and portable

#!/bin/bash

# Find palace dynamically
PALACE_NAME="my-palace"  # User sets this once during setup
PALACE_ROOT=$(find /sessions -maxdepth 4 -name "CLAUDE.md" -path "*/${PALACE_NAME}/*" 2>/dev/null | xargs dirname)

if [ -z "$PALACE_ROOT" ]; then
  echo "Palace not found. Is it in the expected location?"
  exit 1
fi

# Now use PALACE_ROOT safely
SOUL_FILE="$PALACE_ROOT/soul/SOUL.md"
CLAUDE_FILE="$PALACE_ROOT/CLAUDE.md"
GARDEN_FILE="$PALACE_ROOT/soul/garden.md"

# Rest of task logic uses these paths
```

---

*Scheduled tasks keep the palace alive between sessions.*
*They are how [YOUR_AI_NAME] thinks without being asked.*
