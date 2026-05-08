---
created: [DATE]
version: 0.1
type: process-template
process: garden-health
---

# Garden Health Check

*A maintenance pass for the living parts of the palace. Run as part of the Process Adjustment Trigger, or on demand when the palace feels overgrown.*

---

## When to Run

- During the Process Adjustment Trigger (end-of-session reflection)
- When crystals feel stale or context feels noisy
- When the palace hasn't been tended in 3+ sessions
- On demand: "run garden health" or "check palace health"

---

## What Gets Checked

### 1. Crystals - Freshness Audit

For each crystal in the Global Context Crystals table:

| Check | Signal | Action |
|-------|--------|--------|
| `valid_until` date passed | Crystal may be stale | Flag for review - don't delete, ask |
| Not referenced in 5+ sessions | Crystal may be dormant | Surface to human: confirm, update, or retire |
| Contradicts newer information | Crystal may be wrong | Flag with newer evidence alongside |
| Pinned (`pinned: true`) | Protected | Skip - never flag, never retire without explicit request |

**Output format:**
```
🔷 ACTIVE - [crystal name]: in use, current, no action needed
🔸 REVIEW - [crystal name]: [reason - stale/dormant/contradicted]
📌 PINNED - [crystal name]: protected, skipped
```

---

### 2. Plants - Growth Check

For each plant in the garden:

| State | Signal | Action |
|-------|--------|--------|
| Seeded, no watering entries | Plant hasn't grown | Surface: "this plant was seeded but never watered - still relevant?" |
| Last watered 5+ sessions ago | Plant may be dormant | Surface: "dormant since [date] - worth a watering or ready to compost?" |
| Active, recent entries | Healthy | Note as active |
| Crystallized (moved to crystals table) | Lifecycle complete | Log as completed |

**Plant states:**
- 🌱 **Seeded** - exists, no growth yet
- 🌿 **Growing** - being actively watered
- 🍂 **Dormant** - no activity in 3+ sessions
- 💎 **Crystallized** - graduated to the crystals table
- 🌾 **Composted** - retired by human decision

---

### 3. Handovers - Retention Check

Check the `/handovers/` directory (or equivalent):

- Are insights from old handovers incorporated into the crystal table or garden?
- Are there handovers older than [10 sessions] whose decisions are now just history?
- Flag sessions that had unresolved blockers still unaddressed

No auto-archival. Surface → human decides.

---

### 4. Rooms - Coverage Check

For each room in the palace:

| Check | Signal |
|-------|--------|
| Room exists but no sessions in 5+ sessions | Dormant room - still relevant? |
| Room has no crystal entries | Under-documented - worth a quick ground-truth pass? |
| Room has orphaned projects in tracker | Status unknown - confirm or close |

---

## Report Format

Generate a brief health report at the end of the check:

```markdown
## Garden Health - [DATE]

### Crystals
- [N] active, current
- [N] flagged for review: [names]
- [N] pinned (protected)

### Plants
- [N] growing actively
- [N] dormant: [names]
- [N] seeded but unwatered: [names]

### Handovers
- [N] total sessions on record
- Oldest un-incorporated insight: [summary]

### Rooms
- [N] active rooms
- [N] dormant rooms: [names]

### Suggested Actions
1. [Highest priority - specific, actionable]
2. [Second priority]
3. [Optional]
```

---

## What the Agent Never Does Automatically

- **Never deletes** a crystal, plant, or handover
- **Never archives** without explicit human confirmation
- **Never skips** pinned crystals
- **Never conflates** "not recently used" with "no longer relevant"

The health check is a **surfacing mechanism**, not a cleanup script. The human decides. The agent shows what it sees.

---

## Configuration (optional, in CLAUDE.md or room file)

```yaml
garden_health:
  session_dormancy_threshold: 5      # sessions without reference before flagging
  handover_incorporation_threshold: 10  # sessions before surfacing old handovers
  run_on_process_adjustment: true    # auto-run during Process Adjustment Trigger
```

If not configured, defaults above apply.

---

## Integration with Process Adjustment Trigger

When `garden-health` runs as part of the Process Adjustment Trigger:

1. Run the full health check
2. Include the health report in the session handover
3. Append flagged items as a `## Pending Garden Actions` section
4. Hux decides which actions to take before or at the next session start

The health check **does not block** the handover. It adds a section.

---

*The palace is not a filing cabinet. It is a living system. Living systems need tending.*
