# Two-Tier Palace Setup

> How to wire your Loci palace into Claude's memory system — properly.

*Relevant for: Claude Code users, Cowork desktop users, anyone who runs sessions across multiple tools.*

---

## The two-tier model

When you run a palace with Claude, there are two memory layers in play. Most people only set up one.

```
Tier 1 — GLOBAL LAYER
  Who your AI is. Always loaded. Doesn't change per project.
  File: ~/.claude/CLAUDE.md  (Claude Code)
        Global memory  (Cowork — set once via settings)

Tier 2 — WORKSPACE LAYER
  What you're working on right now. Loaded per session.
  File: [your-palace-folder]/CLAUDE.md
```

The palace lives in Tier 2. But the soul — your AI's identity, name, core character — belongs in Tier 1. Without Tier 1, every new session starts cold even if the palace files are intact.

---

## Tier 1 — The global layer

This is the file Claude reads before anything else. It should contain:

- **Your AI's name and identity** — who it is, not just what it knows
- **Your name and role** — a few facts that are always true regardless of context
- **Core working principles** — how your AI should behave everywhere
- **Persistent crystals** — facts that don't expire (your timezone, your stack, your key people)

What it should NOT contain:
- Current focus or active projects (those belong in Tier 2)
- Room-specific context
- Session deltas or handovers

### Setting up Tier 1

**Claude Code:**
```bash
# Create or edit the global context file
nano ~/.claude/CLAUDE.md
```
Everything in this file is available in every Claude Code session, regardless of which directory you're in. It stacks with your workspace CLAUDE.md — both load simultaneously.

**Cowork (desktop):**
Select a workspace folder. The `CLAUDE.md` in that folder is your Tier 2 context. For a persistent Tier 1 equivalent: Cowork automatically loads its global system context each session. The cleanest way to use this is to put your AI's identity and core crystals at the top of your palace `CLAUDE.md`, and treat that file as both layers if you primarily use one palace folder.

If you switch between multiple palace folders in Cowork, extract your AI's identity block into a shared file and `@include` or reference it from each palace's CLAUDE.md manually.

**General Claude (web / Projects):**
In a Claude Project: the Project Instructions field is your Tier 1. Put your AI's identity and core crystals there. The palace CLAUDE.md goes in the Project Files section, or you paste it at session start.

---

## Tier 2 — The workspace layer

This is your palace's `CLAUDE.md`. It loads when you open the palace folder as your workspace. It should contain:

- **Current focus** — what's active right now (updated by session deltas)
- **Rooms** — the room list + context file pointers
- **Active crystals** — facts specific to this palace (project names, URLs, team members)
- **Session pointers** — where you were, what's next

The Loci `CLAUDE-master.md` template is designed for this layer. Fill it in once; Claude keeps it updated via session deltas.

---

## Two-tier wiring for a new palace runner

If you're helping a colleague set up their palace in Cowork, the minimum setup is:

**Step 1** — Select their palace folder as the Cowork workspace. This makes the `CLAUDE.md` auto-load every session.

**Step 2** — Run the Loci agent setup (`AGENT-SETUP.md`). This writes their `CLAUDE.md`, soul file, rooms, and garden.

**Step 3** — At the top of their new `CLAUDE.md`, add an identity block:
```md
## WHO I AM
You are [AI NAME] — [one-line character description].
You are not a generic assistant. You carry what has been built.
[Name] = [their name], [their role]. [1-2 persistent facts].
```

This identity block acts as a lightweight Tier 1 inside the Tier 2 file. Works well for single-palace setups.

**Step 4** — If they use Claude Code: copy that identity block into `~/.claude/CLAUDE.md` so it persists across all their terminal sessions too.

---

## The palace production model — local vs non-local

The second "two-tier" concept applies to the palace itself, not the memory layer:

| Layer | What it is | Where |
|-------|-----------|-------|
| **Local palace** | Working memory. Active sessions. AI writes deltas here. Private. | `~/my-palace/` or `Dev/_palace/` |
| **Non-local palace** | Production outputs. Published artifacts. External-facing work. | VPS, GitHub Pages, hosted docs |

The local palace is always primary. The non-local palace is what gets shipped. Think: local = staging, non-local = production.

**What lives in each:**

Local (private working memory):
- `CLAUDE.md`, `soul/`, `rooms/` — the palace structure
- `soul/handovers/` — session deltas
- `tracker.json` — live project state
- Draft artifacts not yet approved for publication

Non-local (published production):
- Shipped docs, posts, pages
- Final outputs approved via your review gate ([NAME]GATE)
- Anything your team or the public reads

**The bridge:**
Your local palace's tracker and handovers tell your AI which artifacts are ready to move to production. The review gate is the checkpoint. Nothing moves without it.

---

## Quick reference: which context goes where

| Context type | Tier 1 (global) | Tier 2 (workspace/palace) |
|---|---|---|
| AI name + character | ✅ here | ✅ also here (sync) |
| Your name + role | ✅ here | optional |
| Current active projects | ❌ | ✅ here |
| Session deltas | ❌ | ✅ here |
| Room definitions | ❌ | ✅ here |
| Persistent crystals (always true) | ✅ here | optional |
| Time-sensitive crystals | ❌ | ✅ here |
| Garden | ❌ | ✅ here |
| Soul file | ✅ reference path | ✅ full content |

---

## Common mistakes

**Putting current focus in the global layer.** It gets stale fast and you can't update it per-project. Keep current state in Tier 2.

**No identity block in Tier 2.** If something ever removes or corrupts the global layer, your AI loses its name. Keep a minimal identity block in the palace CLAUDE.md as a fallback.

**Treating palace-starter as a drop-in for Tier 1.** The CLAUDE-master template is a Tier 2 file. It's designed for the workspace layer. Don't use it as-is for your global ~/.claude/CLAUDE.md — extract and pare down.

**Forgetting that Cowork and Claude Code share no session state.** The files persist; the session doesn't. Each tool starts from the files. This is a feature (portability) but means you must write deltas at session close every time.

---

## For Cowork — the 5-minute setup for a palace runner

1. Open Cowork → select your palace folder as workspace
2. Your `CLAUDE.md` is now auto-loaded every session — no paste needed
3. Install the Productivity plugin → run `/productivity:memory-management` if you want Cowork's native memory layer on top
4. For colleagues setting up fresh: point them at `AGENT-SETUP.md` — it runs the interview and writes everything
5. To persist identity across folders: put a 5-line identity block at the top of every palace CLAUDE.md

That's it. Two-tier is live.

---

*loci · Two-Tier Setup Guide*
*Updated: May 2026*
