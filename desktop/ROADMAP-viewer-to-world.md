---
title: Loci Roadmap — Viewer to World
date: 2026-05-26
origin: Tauri dogfood session (Hux × Vesper), v0.6 Leptos dashboard first real use
author: Hux (direction) × Vesper (transcription)
tags: design-direction, tauri-bridge, palace-ui, chat-field, rpg, v0.6-phase-4
status: ◇ provisional trajectory, ratified design-only (build deferred)
inherits: DESIGN-IDEAS-rpg-of-cognitive-structures.md
---

# Loci Roadmap: Viewer to World

## The frame

What we dogfooded on 2026-05-26 is a **viewer**: it reads cron-states and handovers
and shows them honestly. The RPG design doc describes a **world**: "my own adventure
book writing itself as I write and play and read onwards."

The gap between viewer and world is exactly one thing: **the viewer has no input.**
You can look at the palace; you cannot act in it. The chat field is the hinge that
turns the map you read into the world you play.

Every RPG mechanic in `DESIGN-IDEAS-rpg-of-cognitive-structures.md` is write-shaped
(water a plant, name a pattern, summon a companion, complete a quest). The current
app is read-only. So the chat field is not just a feature — it is the first write
primitive, and it is load-bearing for the entire RPG trajectory.

## The chat field's three jobs = three phases (risk-ascending)

The chat field's first job is the fork that determines the whole architecture:

- **Query** (read-path): a search box with personality. RAG over palace files.
- **Companion** (narrative-path): the NPC mechanic. Persona voice + aperture.
- **Act** (write-path): the RPG command primitive. Mutates palace state.

These become three phases in rising order of risk. Each de-risks the next.

## Trajectory

### Phase 3.5 — Harden the viewer
Close the dogfood bugs before adding input. Make the read-only surface trustworthy.
- Wire the schema-manifest panel (`read_manifest` exists + tested, just not called by `dashboard.rs`)
- Fix `start_state_watcher` path (`_palace/cron` join, so live-refresh works; it currently watches `<root>/cron`)
- Graceful wrong-folder rejection (no WASM crash; backend already returns clean Err)
- Frontend error boundary (Leptos/WASM panics are currently invisible in the dev terminal)
- Kill the `_palace/_palace` decoy directory (validation trap; see PALACES.md cleanup)

### Phase 4a — Chat as QUERY (read-path, lowest risk, the hinge)
Input field arrives; its first job is to *answer*, not *act*.
- "Show me stale crons", "what did we decide about payments", "which plants need watering"
- Routes to **local Ollama, fail-closed** (privacy contract already in README), grounded in palace files via MCP/RAG
- No state mutation. Validates chat UX + inference wiring before any write risk.
- Ships as the daily-glance companion.
- **DESIGN CONSTRAINT (lock now, build later): the chat talks to an inference _trait_, not a vendor.** This converges the portability quest (CLAUDE.md → model-agnostic) with the chat-backend decision. Build pluggable from day one.

### Phase 4b — Chat as COMPANION (narrative-path)
Add character.
- Load persona soul files (Vesper first, then Cipher / Nyx / RUNE)
- "Ask Vesper", "summon Cipher" — the NPC mechanic, aperture-as-game-mechanic
- Still read-only on state; the new thing is voice + the "arriving in your own palace" feeling
- **RUNE pass** (narrative voice, RPG doc Q3/Q4) lands here

### Phase 4c — Chat as ACT (write-path, highest risk, last)
The RPG command primitive.
- "Water the invisible-proof plant", "name this pattern X", "mark quest done"
- Mutates palace state → **Cipher guardrails**: confirmation, dry-run preview, never-delete-without-approval
- TASKS.md becomes the quest log; plants get growth states; crystals enter inventory

### Phase 5 — The RPG map proper (the spatial world)
- Rooms as map locations; garden spatial viz (Tessellated Recognition §9.6.2)
- Crystals-as-collectibles inventory; connection-count-as-XP
- **Nyx pass** (art register; polytope-net vs graph vs roguelike map; RPG doc Q1/Q2)

## Cross-cutting (flagged, not decided)

- **Backend:** Ollama-first, pluggable-interface from day one. Converges with the portability quest.
- **Privacy:** every mode fail-closed local-first. No external calls without explicit opt-in. Nym/Loci core value prop.
- **Swarm:** Nyx (4b/5) + RUNE (4b) invoked per-phase, not up front.

## Why this sequencing

Risk-ascending, each step de-risks the next. Query (4a) proves the Ollama backend
and chat UX with zero write risk. Companion (4b) adds persona character on a proven
backend. Act (4c) adds state mutation only after the surface is trusted. By the time
the player can write to their own memory, every layer beneath has been dogfooded.

You can stop at any phase and have a coherent product:
3.5 = trustworthy viewer · 4a = useful query companion · 4b = character · 4c = actions · 5 = world.

---

*Crystallised 2026-05-26 from the first real dogfood. The viewer works; the world is sequenced.*
