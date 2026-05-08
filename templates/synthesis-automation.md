---
created: [DATE]
version: 0.1
type: process-template
process: synthesis-automation
---

# Synthesis Automation

*Making the palace reflect on itself. The step between raw session work and crystallised understanding.*

Synthesis is what turns accumulated sessions into deepening knowledge. Without it, the palace grows in volume but not in depth - more handovers, more plants seeded, but fewer crystals formed and fewer connections made. This template covers both the manual synthesis pass and the optional scheduled automation.

---

## What Synthesis Is

A synthesis pass asks:
1. **What did we learn** in recent sessions that isn't yet in the crystal table?
2. **What patterns** are emerging across sessions that no single session captured?
3. **What should be promoted** - from ◇ to ◈, from ◈ to ◆, from plant to crystal?
4. **What connections** exist between things in different rooms that we haven't named?
5. **What should be composted** - stale plants, retired crystals, resolved questions?

A synthesis pass is **not**:
- A summary of what happened (that's the handover)
- A to-do list (that's the tracker)
- A cleanup script (that's the garden health check)

Synthesis produces **new understanding** - not a reorganisation of existing notes.

---

## Manual Synthesis Trigger

Invoke with:
- `"synthesise recent sessions"`
- `"run synthesis pass"`
- `"what have we learned lately that we haven't crystallised?"`
- `"palace synthesis"`

Or as part of the Process Adjustment Trigger.

### Synthesis Pass Protocol

```
1. Read the last [N] handovers (default: 5, configurable)
2. Read the current crystal table and garden
3. Run the five synthesis questions above
4. Produce a synthesis report (see format below)
5. Propose specific updates: crystal promotions, new crystals, plant waterings, retirements
6. Wait for human confirmation before writing anything
```

**Always propose, never auto-write.** The synthesis report is a proposal. The human confirms, modifies, or rejects each item. Then write.

---

## Synthesis Report Format

```markdown
## Synthesis Pass - [DATE]
Sessions reviewed: [N] (handovers [DATE] to [DATE])

### New Learnings Not Yet Crystallised
1. [Observation + proposed crystal or plant entry]
2. [Observation + proposed crystal or plant entry]

### Emerging Patterns
- [Pattern name]: [what it is, evidence from sessions]
  → Proposed: [new ◇ crystal | new plant | existing plant watering]

### Promotions Proposed
- ◇ [crystal name] → ◈: [reason, evidence]
- ◈ [crystal name] → ◆: [reason, confirming sessions]

### Cross-Room Connections Noticed
- [Room A] ↔ [Room B]: [what connects them that we haven't named]

### Candidates for Composting
- [Plant/crystal name]: [last active, reason it may be done]
  ⚠ Confirm before retiring

### Not Synthesised (requires more sessions)
- [Topic]: [what we're watching, expected resolution]
```

---

## Verbosity Modes

| Mode | When to use | Output |
|------|-------------|--------|
| `verbose` | Deep synthesis sessions, quarterly reviews | Full report, all sections, all proposals |
| `quick` | End of busy sprints, time-limited sessions | New learnings + promotions only |
| `pattern-only` | When you suspect something emerging but can't name it | Patterns section only |
| `connections` | Cross-room work, thesis sessions | Cross-room connections focus |

Invoke with: `"run synthesis pass (verbose)"` or `"quick synthesis"`

---

## Scheduled Synthesis (Optional)

For palaces that want periodic automatic synthesis passes without manual triggering:

### Setup

Add to your scheduled tasks (using `templates/scheduled-task-template.md`):

```yaml
task: palace-synthesis
schedule: every 5 sessions
# or: every 7 days
# or: every monday

verbosity: verbose
sessions_to_review: 5
output_to: [palace-root]/synthesis/[DATE]-synthesis.md
auto_apply: false       # always false - synthesis proposals require human review
notify: "Synthesis ready: [palace-root]/synthesis/[DATE]-synthesis.md"
```

**`auto_apply: false` is non-negotiable.** Synthesis automation produces proposals. A human reads them. A human confirms. Then the palace updates. Automating the proposal is fine. Automating the write is not.

### Local Setup (for Hux's palace)

Add to `Dev/CLAUDE.md` or `Dev/nym-stone/vesper/VESPER.md`:

```markdown
## Synthesis Schedule
- Run: every 5 sessions (or on demand)
- Verbosity: verbose
- Sessions to review: 5
- Output: `Dev/_palace/synthesis/`
- Gate: Hux reviews before any write
```

Then create the output directory:
```bash
mkdir -p ~/Dev/_palace/synthesis/
```

The agent will propose synthesis reports and save them to this directory. Hux reviews, modifies, approves. Then the crystal table and garden files update.

---

## Integration with Other Processes

| Process | Relationship |
|---------|-------------|
| `garden-health` | Health check surfaces what's dormant. Synthesis decides if it should grow or compost. |
| `process-adjustment` | Process adjustment is the trigger. Synthesis is one of its sub-passes. |
| `handover` | Handovers are the raw material. Synthesis is what you do with them. |
| `autodream` | Autodream is the scheduled version. Synthesis is the intentional version. |

---

## On Automation vs. Intention

The difference between synthesis automation and tools like Honcho or Mem0 is the **direction of the extraction**.

Those tools extract *who you are* from what you say - your preferences, your communication style. The extraction is automatic, the output is a user model.

Loci's synthesis extracts *what you understand* from what you've worked on - your knowledge architecture, your evolving crystals. The extraction is proposed, the output is a knowledge update. The human is in the loop not as a bottleneck but as the author.

Automation here means: *don't make me remember to do the synthesis pass*. Not: *do it without me*.

---

*The palace synthesises. The human crystallises. That's the order.*
