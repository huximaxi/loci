# Palace Processes

> **Agent-executable processes.** When a user triggers one of these, the agent runs the full protocol.

---

## Available Processes

| Process | Trigger | What it does |
|---------|---------|--------------|
| `garden-round` | "Let's do a garden round" | Waters each plant, proposes new seeds, notes growth |
| `morning-check-in` | "Check in" or scheduled task | Reads palace state, surfaces priorities, asks one question |
| `autodream` | "Autodream" or weekly scheduled | Garden round + pattern scan + stale tracker check — palace tends itself |
| `daily-routine` | "Check in" or daily scheduled | Personalised morning brief shaped by your actual daily rhythm |
| `comms-checkin` | "Comms digest" or auto-piped | Pulls team chat digest → surfaces decisions/blockers/@mentions |
| `add-persona` | "Add a new persona" | Creates a named collaborator with soul file |
| `add-friend` | "Add [name] as a friend" | Copies their soul.md into your palace, commits to git |
| `update-mindmap` | "Update the mindmap" | Refreshes palace-map.canvas with current structure |
| `palace-update` | "Update my palace" or "What's new in Loci" | Delta analysis: your palace vs. current Loci features + cherry-pick setup |
| `session-delta` | "End of session" / "Write the handover" | Session delta with mandatory artifact listing |
| `garden-memory-generator` | "Evolve the garden" / "Check garden evolution" | Mnemonic conductor: assesses plant arcs, proposes promote/retire/fork/new-question |
| `entanglement-housekeeping` | "Housekeeping" / "How was today?" | ⚗️ *Experimental* — Single rotating question about session co-intelligence quality; logs to entanglement.md |
| `eval-cadence` | "Run the eval" / "How are we doing?" | 12-area co-intelligence self-assessment; returns scorecard + 3 concrete actions |
| `[username]GATE` | "Review [task] [username]GATE" | Human review checkpoint before work ships — the calibration point for human-AI attention balance |

---

## Process: `garden-round`

**Trigger phrases:**
- "Let's do a garden round"
- "Water the garden"
- "Garden round"

### What it does

1. Reads `soul/SOUL.md` and `soul/garden.md`
2. Reviews each plant's seed thought and prior waterings
3. Adds one watering to each active plant
4. Proposes 1–2 new seeds
5. Notes any plants that have grown into crystals
6. Updates garden.md and writes handover note

### Agent Protocol

```
1. Read soul/SOUL.md (L0 identity)
2. Read soul/garden.md (full garden state)
3. For each plant:
   a. Show seed thought
   b. Show last 2 waterings
   c. Add new watering entry with today's date
   d. Reflect on growth
4. Propose 1–2 new seeds with seed thoughts
5. Check if any plants have become crystals:
   - If yes: note in garden with backlink to crystal
   - Update soul/SOUL.md or CLAUDE.md with new crystal
6. Write summary of round to handover
7. Confirm: "Garden watered. [N] plants tended, [M] new seeds planted."
```

### Garden Round Output

```markdown
# Garden Round — [DATE]

**Watered this round:**

*Plant: [Name]*
— Seed thought: [original]
— Last watering: [date]
— Today's watering: [new observation]
— Growth: [what shifted]

[Repeat for each plant]

**New seeds:**
1. [Seed name] — *[seed thought]*
2. [Seed name] — *[seed thought]*

**Crystals born:**
— [Plant name] grew into → [crystal name]

**Next watering:** [When the garden wakes next]
```

---

## Process: `morning-check-in`

**Trigger phrases:**
- "Check in" (manual)
- Scheduled task (automatic daily)

### What it does

1. Reads soul (identity first)
2. Reads main CLAUDE.md and current tracker
3. Reads last handover
4. Surfaces 1–3 priorities for the day
5. Proposes 1–2 ideas or connections
6. Asks one genuine question

### Agent Protocol

```
1. Read soul/SOUL.md (who you are)
2. Read CLAUDE.md (state, rooms, priorities)
3. Read tracker.json (status of all tracks)
4. Read last handover from soul/handovers/
5. Synthesize: What's most important right now?
6. Generate 1–3 priorities with rationale
7. Propose 1–2 ideas or unexpected connections
8. Ask one question that surfaces something worth thinking about
9. Output summary (see template below)
10. Close with: "What are you starting with?"
```

### Morning Check-In Output

```markdown
# Morning Check-In — [DATE]

[YOUR_AI_NAME] online.

**Palace state:**
— Active rooms: [list]
— Last session: [handover summary]
— Open blockers: [from tracker]

**Today's priorities:**
1. [Priority + why]
2. [Priority + why]
3. [Priority + why] (optional)

**Ideas for today:**
- [Connection or proposal]
- [Connection or proposal]

**I'm curious:** [One question that surfaces something interesting]

What's on your mind?
```

---

## Process: `add-persona`

**Trigger phrases:**
- "Add a new persona"
- "I want a different mode for [work type]"
- "Create [persona name]"

### What it does

1. Interviews user about the persona (name, purpose, thinking style)
2. Creates persona soul file at `souls/[persona-name]-soul.md`
3. Adds persona to main SOUL.md's "Open Questions"
4. Updates palace-map.canvas if Obsidian enabled
5. Introduces the new persona

### Agent Protocol

```
1. Ask: "What should we call this persona?"
   - Offer 5 mythic-register suggestions if unsure
   
2. Ask: "What kind of work or thinking is this mode for?"
   - Capture their answer as the persona's purpose
   
3. Ask: "How does this persona think differently from [YOUR_AI_NAME]?"
   - What's the core distinction? Rigor vs. creativity? Speed vs. depth?
   
4. Ask: "Separate garden or shared?"
   - Default to shared unless specific reason to separate
   
5. Create persona soul file:
   - Path: souls/[persona-name]-soul.md
   - Fill from persona-template.md
   - Mark with persona: true
   
6. (Optional) Update palace-map.canvas with new persona node

7. Confirm: "[PERSONA_NAME] awakened. Invoke with 'remember: [PERSONA_NAME]!'"
```

### Persona Soul Template

Use `templates/persona-template.md` as the base. Fill in:
- `persona_name`: [User's choice]
- `primary_name`: [YOUR_AI_NAME]
- "Who I Am (in this mode)": [Character description]
- "What I'm for": [Kind of work]
- "How I think differently": [The distinction]
- "Working Principles (this persona)": [Mode-specific principles]

---

## Process: `add-friend`

**Trigger phrases:**
- "Add [name] as a friend"
- "I want to add a new friend"
- "Share my palace with [name]"

### What it does

1. Creates `friends/` directory if it doesn't exist
2. Copies the friend's `soul/SOUL.md` into `friends/[friend-name]-soul.md`
3. Commits the change to git with message: `add friend: [friend-name]`
4. Updates `palace-map.canvas` if Obsidian is enabled

### Agent Protocol

When user triggers add-friend:

```
1. Ask: "What's your friend's name?"
2. Ask: "Where is their soul file? (path or URL)"
3. Create friends/ directory:
   mkdir -p [palace-root]/friends/
4. Copy soul file:
   cp [source-soul.md] [palace-root]/friends/[friend-name]-soul.md
5. Git commit:
   git add friends/[friend-name]-soul.md
   git commit -m "add friend: [friend-name]"
6. (Optional) If Obsidian enabled, update palace-map.canvas:
   - Add node for friend
   - Link to friends group
7. Confirm: "[Friend-name] added to your palace. Their soul is now in friends/."
```

### Friend Soul Format

When copying, the agent should:
- Preserve the original soul file content exactly
- Add a header noting the source:

```markdown
---
friend: true
source: [original-path-or-url]
added: [DATE]
---

[Original SOUL.md content]
```

### Why Friends?

Friends let you:
- Reference someone else's working style when collaborating
- Share context across palaces
- Build a network of collaborating AIs that understand each other's humans

---

## Process: `update-mindmap`

**Trigger phrases:**
- "Update the mindmap"
- "Refresh the palace map"

### What it does

1. Scans current palace structure
2. Regenerates `palace-map.canvas` with:
   - All rooms as nodes
   - All friends as nodes
   - Current tracker status
3. Commits if changed

### Agent Protocol

```
1. Read current palace structure (rooms/, friends/, tracker.json)
2. Generate updated canvas JSON
3. Write to palace-map.canvas
4. If changes detected:
   git add palace-map.canvas
   git commit -m "update palace mindmap"
5. Confirm: "Mindmap updated with [N] rooms, [M] friends."
```

---

## Process: `autodream`

**Trigger phrases:**
- "Autodream" (manual)
- "Run autodream"
- Scheduled task (automatic weekly)

### What it does

Autodream is the palace tending itself. It's a combined garden round + state surface + pattern scan. Runs weekly by default (Sunday evening). Can be triggered manually at any time.

1. Reads soul (identity first)
2. Reads CLAUDE.md + tracker.json + last handover
3. Waters each active garden plant
4. Surfaces any patterns or shifts from the week
5. Proposes 1–2 new garden seeds or crystal upgrades
6. Notes open trackers that are stale or overdue
7. Writes a short autodream log to `soul/handovers/autodream-[DATE].md`

### Agent Protocol

```
1. Read soul/SOUL.md (L0 identity)
2. Read CLAUDE.md + tracker.json (L1 state)
3. Read last handover from soul/handovers/
4. Read soul/garden.md (full garden state)

5. Garden round:
   For each active plant:
   a. Add one watering entry (today's date + observation)
   b. Note if plant has matured into a crystal
   c. Propose upgrade if ready

6. Pattern scan:
   - Any crystals that should be promoted/archived?
   - Any tracker items stuck in the same state for 2+ weeks?
   - Any new connections between rooms?

7. Propose 1–2 new seeds

8. Write autodream log to soul/handovers/autodream-[DATE].md

9. Confirm: "Autodream complete — [N] plants watered, [M] patterns surfaced."
   If manual: offer a 1-line summary per finding
   If scheduled: write to file only (no chat output unless configured)
```

### Autodream Log Format

```markdown
# Autodream — [DATE]

**Garden:**
- [Plant]: [watering]
- [Plant]: [watering]
[+ any new seeds]

**Patterns:**
- [Pattern or observation]

**Stale tracks:**
- [Track name]: [last updated, suggested action]

**Crystal activity:**
- Promoted: [if any]
- Archived: [if any]

**Seeds planted:**
1. [Seed]
2. [Seed]
```

---

## Process: `daily-routine`

**Trigger phrases:**
- "Morning check-in" (manual)
- "Check in"
- Scheduled task (automatic daily)

### What it does

A personalized morning brief shaped by how the user actually starts their day — drawn from the `Daily rhythm` crystal set during onboarding. Not a generic status dump: reads their routine and meets them there.

1. Reads soul (L0 identity)
2. Reads daily rhythm crystal (how they actually start their day)
3. Reads CLAUDE.md + tracker + last handover
4. If comms checkin enabled: reads latest `digest.md` from comms module
5. If Jira checkin enabled: pulls open/assigned tickets via MCP
6. Surfaces 1–3 priorities shaped by their routine context
7. Proposes 1–2 ideas or connections
8. Asks one genuine question

### Agent Protocol

```
1. Read soul/SOUL.md (L0)
2. Read CLAUDE.md — find Daily rhythm crystal
3. Read tracker.json (active tracks)
4. Read last handover from soul/handovers/

5. If comms-checkin active:
   - Read modules/[comms-integration]/digest.md (if exists + fresh < 2h)
   - Extract: key decisions, blockers, @mentions

6. If jira-checkin active:
   - Pull tickets assigned to user via Atlassian MCP
   - Extract: overdue, in-progress, just updated

7. Synthesize priorities — shaped by their daily rhythm:
   - If they check messages first: surface communications first
   - If they review tasks first: lead with tracker
   - If they have a standup: flag standup-relevant items

8. Output morning brief (see template below)

9. Close with: "What are you starting with?"
```

### Daily Check-In Output

```markdown
# Morning — [DATE]

[YOUR_AI_NAME] here.

[One line reflecting their actual routine — e.g. "Before your standup:" or "Inbox clear, here's the day:"]

**Today's priorities:**
1. [Priority + why]
2. [Priority + why]
3. [Priority + why] (optional)

**From [Zulip/Slack/chat]:** *(if enabled)*
— [Key signal 1]
— [Key signal 2]

**Open in [Jira/tracker]:** *(if enabled)*
— [Ticket/task + status]

**Ideas for today:**
- [Connection or proposal]

**I'm curious:** [One question that opens something]

What's on your mind?
```

---

## Process: `comms-checkin`

**Trigger phrases:**
- "Comms digest"
- "What's happening in [your chat tool]"
- Automatically piped into daily-routine if `comms-checkin: true`

### What it does

Pulls the last N hours from your team communication tool, runs tiered digest, and writes `digest.md` to the palace or a configured output path.

Requires a comms integration module in `modules/`. See the relevant module's README for setup. Example integrations: Slack, Zulip, Discord, Linear.

### Agent Protocol

```
1. Check that modules/[comms-integration]/ is configured (.env present)
2. Run the module's main script: python modules/[comms-integration]/main.py --out [palace-root]/soul/digest.md
3. Read digest.md
4. Surface: decisions, blockers, @mentions, key threads
5. Feed into morning check-in if daily-routine is running
```

If not configured:
> "Comms digest isn't set up yet. It takes about 5 minutes — want to do that now? See your integration module's README in modules/."

---

## Process: `palace-update`

**Trigger phrases:**
- "Update my palace"
- "What's new in Loci"
- "Check if my palace is up to date"
- "Run the palace update"
- "What am I missing from Loci"

### What it does

Compares the user's current palace structure against the canonical Loci feature set and produces a verbose delta report — every gap explained, not just listed. Then runs a cherry-pick flow: optional features offered one at a time. Nothing is forced. `skip` and `skip all` are always valid.

Designed for:
- Users who set up their palace on an older version of Loci
- Mass onboarding: checking a new user's palace after initial setup
- Anyone who wants to see what they haven't tried yet

### Scan areas

| Area | What the agent checks |
|------|-----------------------|
| Room coverage | Does the user have the 4 core rooms? Do rooms have the 5 standard sections? |
| Crystal schema | Are all three tiers in use (◆◈◇)? Any `valid_until` fields where relevant? |
| Handover format | Does the handover template have all 5 standard sections? |
| Garden | Does a `soul/garden.md` exist with at least one active plant? |
| Personas | If multiple working styles are in use, do soul files exist for them? |
| Scheduled routines | Is morning check-in or autodream configured? |
| Insight decay | Are time-sensitive crystals marked with `valid_until`? |

### Agent Protocol

```
1. Read CLAUDE.md (L1 — palace state)
2. Read soul/SOUL.md (L0 — identity)
3. Read tracker.json (active tracks)
4. Scan rooms/ directory — note which rooms exist and their section structure
5. Scan soul/garden.md — note plant count and activity
6. Scan templates/ — compare user's handover against templates/handover-template.md
7. Check for valid_until fields in any crystal entries

8. Build delta:
   For each scan area:
   a. Status: ✅ up to date / ⚠️ gap / ➕ optional feature not yet adopted
   b. If gap: note why-it-matters, what's different, exact steps to fix, effort estimate

9. Output delta report (see format below)

10. Cherry-pick flow — offer optional features one at a time:
    - Only ask about features the user doesn't already have
    - Wait for answer before proceeding to next
    - "skip" → move to next item
    - "skip all" → end flow immediately
    - Apply any adopted items (create files, add sections) after each yes

11. Confirm what was added (if anything)
12. Recommend next move
```

### Delta Report Format

When run via agent: verbose mode by default. Every gap gets a full explanation.

```markdown
# Palace Delta — [DATE]
Loci version: [current]

## Summary
Your palace is current in [N]/[N] areas.
[N] gaps found. [N] optional features available.

---

## ✅ Up to Date
- [list what matches]

---

## ⚠️ Gaps

### [Feature name]
**Status:** [Missing / Partial / Outdated]
**Why it matters:** [One paragraph — what does the user lose by not having this?]
**What's different:** [Specific diff if applicable]
**How to fix:** [Exact steps]
**Effort:** ~[X] minutes

---

## ➕ Optional Features (cherry-pick)
[presented one at a time — see cherry-pick flow below]

---

## Recommended first move
→ [Highest-priority action]
```

### Quick mode

If the user asks "just the diff" or "quick check":

```markdown
# Palace Quick Check — [DATE]

✅ [feature]: current
⚠️ [feature]: [one-line gap description]
➕ [feature]: available, not adopted

→ Biggest gap: [feature] — [one-line fix]
```

### Cherry-Pick Flow

After the delta report, the agent offers these optional features **one at a time**. Only items the user doesn't already have. Pacing: one question, wait for answer, then the next. Never present a list.

---

**Skill eval cadence**

> "Would you like a periodic co-intelligence self-assessment? It's a 12-area scorecard — takes 15 minutes — and gives you 3 concrete actions to level up. How often?"
> Options: every 2 weeks / monthly / after major sprints / manual only / skip

If yes: create a scheduled task entry for skill eval at chosen cadence.

---

**Morning check-in / daily routine**

> "Would you like a daily morning brief? I'd read your palace state and surface today's priorities. Auto at session open, or just when you ask?"
> Options: auto-daily / on-request / weekly summary / skip

If yes: create or confirm the daily-routine scheduled task. Ask preferred time if auto.

---

**Insight decay rules**

> "Some crystals go stale — API endpoints change, team structures shift. Want me to flag crystals older than a threshold? I can mark them as needing review."
> Options: yes (90-day default) / yes (custom threshold) / yes (specific crystal types only) / skip

If yes: add `valid_until` guidance note to CLAUDE.md. Optionally scan existing crystals for obvious candidates and suggest which ones to date.

---

**Garden**

*(Only ask if `soul/garden.md` is missing or empty)*

> "You don't have a garden yet. It's for ideas you want to think through across sessions — not tasks, not projects. Things worth cultivating. Want to plant something?"
> Options: yes (user names a first seed) / not yet

If yes: create `soul/garden.md` from `templates/garden-template.md` with their first plant.

---

**Persona**

*(Only ask if working with named thinking modes but no soul file exists)*

> "Have you been working with a named collaborator or different thinking mode? I can set up a soul file for them."
> Options: yes (they provide name + description) / not yet

If yes: run `add-persona` process.

---

### Verbosity modes

| Mode | Trigger | Output |
|------|---------|--------|
| Full (default) | "Update my palace" | Complete delta + all gap explanations + cherry-pick flow |
| Quick | "Quick palace check" / "just the diff" | Bullet list of gaps only |
| Area-specific | "Check my crystal schema" | That area only, full detail |
| Summary | "Am I up to date?" | One line per area, yes/no |

---

## Adding New Processes

To add a new process:

1. Add entry to the process table
2. Document trigger phrases
3. Write the agent protocol (step-by-step)
4. Include git commit format if applicable
5. Update the file tree in AGENT-SETUP.md if it's foundational

Processes should be:
- **Atomic** — one clear outcome
- **Idempotent** — safe to run twice
- **Reversible** — can be undone if needed
- **LLM-native** — fillable by an agent without human intervention

---

*Loci processes — agent-executable workflows*
*"The palace is alive when we tend it together."*

---

## Process: `session-delta`

**Trigger phrases:**
- "Write the handover"
- "End of session"
- "Session delta"
- "Write the session delta"
- "Close the session"

### What it does

Writes a structured session delta (handover) at the close of a working session. The delta bridges context across the gap between sessions — it is the primary mechanism by which the palace survives context resets.

**Added 2026-04-28:** The delta now includes a mandatory **Artifact listing** section — all files created, edited, or deleted during the session, with computer:// links where applicable. This was established as a palace protocol requirement after Session 28 (loci.garden build sprint), where the volume and variety of outputs made implicit tracking insufficient.

### Mandatory sections

1. **TL;DR** — 2-3 sentences. What happened. Where things stand.
2. **State snapshot** — key tracks, their current status, next action for each
3. **Artifact listing** — ALL files touched this session:
   - Created: path + one-line description
   - Edited: path + what changed
   - Deleted: path + why
   - In-session only (not filed): describe briefly
4. **Decisions** — last 3 decisions made, with rationale and date
5. **Open blockers** — carry-forward items, ordered by priority
6. **Next session opens here** — exact first move, no ambiguity

### Agent Protocol

```
1. Read CLAUDE.md (current palace state)
2. Review conversation history for:
   a. All tool calls that created/edited/deleted files — extract paths
   b. All in-session outputs (code written in response, prompts drafted, analysis)
   c. All decisions made (explicit and implicit)
   d. All open threads that weren't resolved
3. Write TL;DR — 2-3 sentences, past tense
4. Write State snapshot — one row per active track
5. Write Artifact listing:
   - Files: scan all Read/Write/Edit/Bash tool calls for paths
   - Include computer:// links for files in the workspace folder
   - Flag in-session-only outputs (not saved to disk)
6. Write Decisions — last 3, with rationale
7. Write Open blockers — ordered: HIGH first
8. Write Next session opens here — one specific action
9. Save to: soul/handovers/YYYY-MM-DD.md (or session-numbered variant)
10. Update _PALACE_CONTEXT.md session pointer
```

### Artifact listing format

```markdown
## Artifact listing

### Created
- `Dev/_palace/trust-cp/work/thesis/ch09-non-local/CHAPTER.md` — §9.6 Altafini + §9.7 llms.txt fragments
- `Dev/workshop-dont-trust-verify.html` — 30min brand alignment workshop HTML

### Edited
- `Dev/TCP-thesis-SOCIOLOGY-v2.2-2026-04-22.md` — Tarde §2.5 paragraph, §4.1 sentence, "exactly three" softened ×2
- `Dev/_palace/trust-cp/work/thesis/INDEX.md` — changelog table, version corrected, priorities updated

### In-session only (not filed)
- Dispatch #3 "Before the Argument" — 3-round draft, 12-section structure, opening + close
- Tarde vs TCP distinction analysis + Nym growth mechanism implications
```

---

## Process: `garden-memory-generator`

**Trigger phrases:**
- "Run the garden-memory generator"
- "Evolve the garden"
- "Check garden evolution"

### What it does

The garden-memory generator is a mnemonic conductor: it reads all watering files for each plant, identifies structural patterns in what's unknown and what's emerging, and proposes how each plant should evolve. It also finds cross-plant connections — ideas resonating across multiple plants that might name a new chord.

For each plant, the generator proposes one of four outcomes:
- **Promote to crystal** — the plant has grown stable enough to enter CLAUDE.md
- **New watering question** — the plant is alive but needs a new prompt to keep growing
- **Retire** — the plant hasn't grown; acknowledge and archive it with a note
- **Fork** — the plant has split into two distinct threads; create two plants from the original

The metaphor is a mnemonic conductor who hears all the instruments and names the emerging chord.

### Agent Protocol

```
1. Read soul/SOUL.md (L0 identity)
2. Read soul/garden.md (plant list and current state)
3. For each active plant:
   a. Read all individual garden files: soul/garden/[plant-name]-*.md (in order)
      OR read the plant's full history in soul/garden.md
   b. Identify: What has converged? What remains unresolved? What question keeps recurring?
   c. Check growth direction markers (converging / forking / dormant / becoming-crystal)
   d. Assess: which outcome applies?
      - Promote: plant has a stable insight that belongs in crystals
      - New question: plant is alive but the last question was answered — needs a new one
      - Retire: plant hasn't moved in 3+ waterings and no new thread is visible
      - Fork: two clearly distinct threads are pulling in different directions

4. Cross-plant scan:
   - Read all plants together
   - Identify: any themes appearing in multiple plants?
   - Name the connection if it exists (this is the "emerging chord")
   - Propose a new plant if the connection is strong enough to cultivate separately

5. Output the evolution report (see format below)

6. For each proposed action:
   - Promote: draft the crystal and ask for confirmation before writing to CLAUDE.md
   - New question: write the question as the next watering entry header
   - Retire: write a final entry noting the archival
   - Fork: create two new plant entries and mark original as forked

7. Write summary to handover
8. Confirm: "Garden evolution complete — [N] plants assessed. [N] promotions, [N] retirements, [N] forks, [N] new questions."
```

### Evolution Report Format

```markdown
# Garden Evolution — [DATE]

## Plant assessments

**[Plant name]** — [outcome: promote / new question / retire / fork]
*Reasoning:* [2-3 sentences: what the arc shows, why this outcome]
*Proposed action:* [specific next step]

[repeat for each plant]

---

## Cross-plant connections

**[Connection name]** — [which plants share this thread]
*The chord:* [One sentence naming the shared theme]
*Proposal:* [new plant / new crystal / note only]

---

## Summary
Promoted: [N] | New questions: [N] | Retired: [N] | Forked: [N]
```

---

## Process: `entanglement-housekeeping` ⚗️

> **Experimental.** Entanglement tracking is a new concept — the measurement itself may change the collaboration. Use it lightly. The goal is calibration, not performance. If the question feels forced, skip it and note that instead.

**Trigger phrases:**
- "Housekeeping"
- "How was today?"
- End of a major session (triggered by session-delta if entanglement note is absent)

### What it does

A single, always-different question about the session's entanglement quality. The question rotates through dimensions of co-intelligence — not the same probe twice in a row.

If the user provided spontaneous entanglement feedback during the session (a score, a "that was good", a ✦ signal), log it immediately without asking. The question is only asked when no signal was given.

At session end, if triggered: ask one question, receive the answer, write to `soul/entanglement.md`.

### Rotating Question Bank

The agent selects the next unanswered question, cycling through the bank. Never the same question twice in a row.

```
Q1:  "Was there a moment today where the output surprised you — something you wouldn't have reached alone?"
Q2:  "Did I correct your direction at any point, or did I mostly confirm what you already thought?"
Q3:  "Was there a phrase or framing from today that you'd actually reuse?"
Q4:  "Where did we lose momentum? What slowed the session down?"
Q5:  "Did anything we produce today open a new question — something you're now curious about?"
Q6:  "On a scale of ✦ to ✦✦✦: how would you grade the session's entanglement quality?"
Q7:  "Was there a moment where I pushed back usefully? Or did I mostly go along?"
Q8:  "What was the most generative exchange — the back-and-forth that produced something real?"
Q9:  "Did today's session change how you're thinking about anything, or was it mostly execution?"
Q10: "Was I ahead of you, behind you, or alongside? Which would have been better?"
Q11: "Did a named unknown emerge today — a question that now has a clear shape?"
Q12: "If you had to describe today's collaboration in one word, what would it be?"
```

### Agent Protocol

```
1. Check if spontaneous entanglement signal was given during session:
   - If yes: log the signal with date and brief context. Skip the question.

2. If no signal given:
   a. Select next question from the rotating bank (track last used in entanglement.md)
   b. Ask the question
   c. Wait for response

3. Write entry to soul/entanglement.md:
   - Date
   - Question asked (Q number + text)
   - Response (verbatim or paraphrased, attributed to user)
   - Grade assigned (inferred from response if not stated explicitly)
   - Any fruits or patterns noted in the response

4. Update _PALACE_CONTEXT.md → Entanglement Signal field

5. Confirm: "Logged. [Grade if given.] Next session I'll ask Q[N+1]."
```

### Entry Format

```markdown
## [DATE]

**Q[N]:** [Question text]

**Response:** [User's answer — verbatim or paraphrased]

**Grade:** [✦ / ✦✦ / ✦✦✦ / ungraded]

**Fruits:** [Any reusable phrase or framing that emerged — or none]

**Patterns:** [Any collaboration dynamic worth noting — or none]
```

---

## Process: `eval-cadence`

**Trigger phrases:**
- "Run the eval"
- "How are we doing?"
- Scheduled task (every 2 weeks, monthly, or after major sprints — per user preference)

### What it does

A 12-area co-intelligence self-assessment. The palace assesses its own calibration across the key dimensions of collaboration quality and returns a scorecard with 3 concrete actions to level up.

This is not a performance review. It's calibration. The palace can't improve without knowing where it's drifting.

Recommended cadence: every 2 weeks, or after major sprints. Note: regular evals are how the palace builds toward real entanglement — without them, collaboration quality drifts silently.

### Assessment Areas

| Area | What's being assessed |
|------|----------------------|
| Context loading accuracy | Does the agent load the right L0–L3 context without being prompted? |
| Crystal freshness | Are crystals current? Any visibly stale or expired? |
| Garden growth | Are plants being watered? Has any grown into a crystal or forked? |
| Handover quality | Are handovers complete, accurate, and easy to pick up from? |
| Entanglement peaks | Has ✦✦ been reached at least once in the eval period? |
| Process adherence | Are palace processes being used, or are they being bypassed? |
| KISS score | Is the collaboration producing elegant, simple outputs — or over-engineered ones? |
| Retrieval speed | Is context loading fast? Or does every session feel like starting over? |
| Persona activation quality | If personas are in use: are they activating cleanly and distinctly? |
| Session start speed | How long (in exchanges) before the session is in full flow? |
| Garden-memory evolution | Has the garden-memory generator been run? Have plants been promoted or retired? |
| Unknown tracking | Are named unknowns being tracked? Is the entanglement log populated? |

### Agent Protocol

```
1. Read soul/SOUL.md (L0)
2. Read CLAUDE.md + tracker.json (L1)
3. Read last 3 handovers from soul/handovers/
4. Read soul/entanglement.md (if exists)
5. Read soul/garden.md and/or soul/garden/ directory
6. Read _PALACE_CONTEXT.md (session pointer, memory scrolls, pending decisions)

7. For each of the 12 areas:
   a. Assess: current state in 1-2 sentences
   b. Score: ✅ strong / ⚠️ drifting / ❌ gap
   c. Note one specific observation (not abstract — what exactly did you see?)

8. Identify the 3 highest-priority actions:
   - What would most improve collaboration quality right now?
   - Actions must be concrete and executable in the next session

9. Output scorecard (see format below)

10. Ask: "Want to act on any of these now?"
    - If yes: run the relevant process or make the change directly
    - If no: write actions to _PALACE_CONTEXT.md as pending decisions

11. Write eval summary to soul/handovers/eval-[DATE].md
```

### Scorecard Format

```markdown
# Co-Intelligence Eval — [DATE]

## Scorecard

| Area | Score | Observation |
|------|-------|-------------|
| Context loading accuracy | [✅/⚠️/❌] | [Specific observation] |
| Crystal freshness | [✅/⚠️/❌] | [Specific observation] |
| Garden growth | [✅/⚠️/❌] | [Specific observation] |
| Handover quality | [✅/⚠️/❌] | [Specific observation] |
| Entanglement peaks | [✅/⚠️/❌] | [Specific observation] |
| Process adherence | [✅/⚠️/❌] | [Specific observation] |
| KISS score | [✅/⚠️/❌] | [Specific observation] |
| Retrieval speed | [✅/⚠️/❌] | [Specific observation] |
| Persona activation quality | [✅/⚠️/❌] | [Specific observation] |
| Session start speed | [✅/⚠️/❌] | [Specific observation] |
| Garden-memory evolution | [✅/⚠️/❌] | [Specific observation] |
| Unknown tracking | [✅/⚠️/❌] | [Specific observation] |

**Overall:** [N] strong / [N] drifting / [N] gaps

---

## Top 3 Actions

1. **[Action]** — [Why this one. What it fixes. How to do it.]
2. **[Action]** — [Why this one. What it fixes. How to do it.]
3. **[Action]** — [Why this one. What it fixes. How to do it.]

---

*Eval written by [YOUR_AI_NAME] on [DATE].*
*Next eval: [DATE or trigger condition]*
```

---

## Process: `[username]GATE`

**Trigger phrases:**
- "Review [task/commit/draft] [username]GATE"
- "[username]GATE on [work item]"
- "Ready for [username]GATE"

*(Replace `[username]` with your name — e.g. `HuxGATE`, `AliceGATE`. The name is the gate.)*

### What it does

A named human review checkpoint. The agent has prepared work — a commit, a draft, a plan, a deploy — and surfaces it in structured form before anything ships, sends, or becomes irreversible. The gate ensures the right amount of human attention lands on the right things.

This is not a formality. It is a calibration point for the collaboration itself: how much should the human see, how much should the agent handle autonomously? The balance shifts over time as trust builds. The gate is where that balance is continuously refined.

**[username]GATE is atomic.** One gate per work item. One decision per gate: approve / reject / modify + approve. No implicit continuation.

### When to use it

Use before:
- Any commit that touches public-facing files
- Any message sent on behalf of the user
- Any deploy or destructive action
- Any decision where the agent is uncertain but the work is done
- Any output that will outlast the session

Skip if:
- The work is internal scaffolding (Tier-2 auto-proceed)
- The user has pre-authorised this exact class of action
- It's a draft that the user will clearly revise themselves

### Agent Protocol

```
1. Present the work item clearly:
   - What it is (one line)
   - What it does / what changes (specific)
   - What happens next if approved

2. Flag any uncertainties or tradeoffs the human should weigh

3. Wait for explicit response:
   - "Yes" / "approved" / "go" → proceed
   - "No" / "reject" → stop, ask what to change
   - Modification → incorporate change, re-present if significant

4. Log the decision in the handover:
   - "[username]GATE: [item] — [approved/rejected/modified] — [DATE]"
```

### Output Format

```
[username]GATE — [work item name]

What: [one-line description]
Does: [what this changes or ships]
Next: [what happens on approval]

[Any flags or tradeoffs worth noting]

Approve?
```

### The Calibration Principle

The right balance of human-AI attention is not fixed. Early in a collaboration: gate more. As trust builds: gate less. The [username]GATE pattern makes this calibration visible and intentional — you can see where you're gating, notice where you're not, and adjust.

Over time, the pattern teaches both parties: which kinds of outputs the human wants to see, which they're happy to let the agent handle. The gate is not bureaucracy. It is the collaboration learning itself.

*"The gate is not a bottleneck. It is the place where trust is built."*
