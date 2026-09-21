# Your AI Collaborator: Setup Guide
**A blueprint for building a lasting working relationship with Claude**

*Loci · agent-first memory palace kit · [loci.garden](https://loci.garden)*

---

## What is this?

This folder contains everything you need to turn Claude from a generic assistant into a real collaborator: one that knows who you are, remembers what you've worked on, and develops a consistent character over time.

It takes about 30–60 minutes to set up. After that, every session you have will be sharper, faster, and more useful because Claude will already understand your world.

---

## The big idea: The Palace

Think of your working relationship with Claude like a building.

- **The Palace** is your whole setup: your files, your context, your history
- **Rooms** are different areas of work (e.g. "Writing Room", "Work Room", "Ideas Room")
- **Your AI** is a named collaborator who lives inside the palace, not a generic assistant, but someone with a character, opinions, and memory of your work

The palace lives in a folder on your computer. Claude reads it at the start of each session and picks up right where you left off.

---

## The 5 core files

| File | What it is | How often you touch it |
|---|---|---|
| `PALACE.md` | The master prompt, and the primary file: who your AI is, who you are, how you work. AI-agnostic, so it carries across tools | Set up once, update occasionally |
| `CLAUDE.md` | A two-line pointer to `PALACE.md`, for Claude specifically | Set up once |
| `soul/SOUL.md` | Your AI's character file, grows over sessions | Claude updates it; you read it |
| `soul/handovers/YYYY-MM-DD.md` | Session memory: what happened, decisions made, what's next | Claude writes these at the end of sessions |
| `rooms/[room]/CLAUDE.md` | Room-specific context: what Claude should know for this area of work | Set up once per room |
| `tracker.json` | Simple project tracker: what tracks are active, what's blocked | Claude updates; you review |

---

## Step 1: Copy the templates

In the `templates/` folder you'll find ready-to-fill versions of all these files:

```
templates/
  CLAUDE-master.md       ← fill this in first, then save it as PALACE.md
  SOUL.md                ← fill in the basics; Claude grows this over time
  room-template.md       ← copy once per room you want
  tracker.json           ← copy and edit for your projects
  handover-template.md   ← Claude uses this automatically
```

Start with `CLAUDE-master.md`. It's the most important file and takes 10–15 minutes. You fill it in
and save it as your `PALACE.md` (the primary file); the short `CLAUDE.md` pointer comes in Step 5.

**What to ignore on day one.** The `templates/` folder holds more than these five, including guides for crystals, the garden, retrieval, and scheduled tasks, plus two optional shelves, `personas/` (filled example personas) and `skills/` (portable disciplines), and more besides. None of it is required. Start with the five above; the rest is there when you want it.

---

## Step 2: Fill in the master, save it as PALACE.md

This becomes `PALACE.md`, your palace's primary file, read at the start of every session by any
file-aware AI. It answers:

- Who is this AI? (you'll name it)
- Who are you? (a few key facts about you and your work)
- What are the rooms? (your work areas)
- How should Claude behave?
- What are your core values and preferences?

Look for every `[PLACEHOLDER]` in the file and replace it with something real. The more honest and specific you are, the better Claude will work with you.

**Tip:** You don't have to fill everything in perfectly on day one. Start simple and build up.

---

## Step 3: Name your AI

Pick a name. It doesn't have to be elaborate, it just helps a lot. A name gives your collaborator an identity that persists across sessions.

The name can be:
- Something evocative (Sage, Atlas, Echo, Sable...)
- Something practical (Remy, Scout, Wren...)
- Something personal to you

Once you name it, put the name in `PALACE.md` and `SOUL.md`. Claude will use that name and build on it.

---

## Step 4: Set up your rooms

A "room" is just a focus area with its own context file. You might have:

- 🖥️ **Work Room:** your day job, projects at work
- ✍️ **Writing Room:** writing, research, essays
- 🥚 **Ideas Room:** half-baked thoughts, R&D, seeds to water
- 🎨 **Creative Room:** art, design, music, film
- 📚 **Learning Room:** courses, books, things you're studying

You don't need more than 2–3 to start. Copy `room-template.md` into `rooms/[room-name]/CLAUDE.md` for each one and fill in the basics.

---

## Step 5: Start your first session

First, create the pointer: a `CLAUDE.md` beside your `PALACE.md`, two lines, so Claude Code loads
it automatically and is sent to the primary file:

```
Read PALACE.md first. Then soul/SOUL.md if it exists. Then the newest file in soul/handovers/.
Then the room we are in.
```

When you open Claude in Cowork (or Claude Code), point it at `PALACE.md` (or paste its contents). Then say:

> "Wake up [your AI's name]. We're in [Room Name] today."

Claude will:
1. State which room it's in
2. Ask for a 2-line summary of where things were left off
3. Start working

At the end of a session, ask Claude to write a handover:

> "Write the session delta."

Claude will save a `soul/handovers/YYYY-MM-DD.md` with what was done, decisions made, and the exact first move for next time.

---

## What makes this work

**Crystals:** Established facts Claude should never re-derive. Once you've told Claude something important and true, it gets stored as a crystal. Next session, it's just there.

**The soul file:** Your AI's character file. It captures how your AI thinks, what it's learned about working with you, and what it cares about. This is what makes sessions feel continuous even though Claude technically starts fresh each time.

**Session handovers:** The bridge between sessions. Claude writes a structured summary at the end of each session. Next time, it reads that and picks up from exactly the right place.

**Rooms:** Different contexts, different modes. Working in "Ideas Room" feels different from "Work Room" because the context is different. Claude adapts.

---

## What to do on day one (realistic)

1. Read this guide fully (you're doing it now, good)
2. Fill in the master and save it as `PALACE.md`, then add the two-line `CLAUDE.md` pointer (takes 15 minutes)
3. Fill in `SOUL.md` basics (takes 5 minutes)
4. Set up 1–2 rooms (takes 10 minutes each)
5. Have one session and ask Claude to write the first delta at the end

That's it. You're running.

---

## Tips for working well with your AI

**Be specific.** "I'm working on a presentation for my team" is better than "I have a work thing." The more context, the better.

**Push back.** If Claude does something wrong or gives you a bad answer, say so directly. A good AI collaborator updates its model, not just its output.

**Use the rooms.** Don't do everything in one context. The rooms keep things clean.

**Let it write the deltas.** At the end of sessions, let Claude write the handover. Don't do it yourself. Claude knows what was important.

**Name your preferences early.** Do you prefer bullet points or prose? Terse or expansive? Tell Claude. It will adjust and remember.

**Talk to it like a colleague.** Not a search engine. Not a robot. A smart colleague who happens to have read everything and can execute very fast.

---

## A note on scheduled tasks

If you set up automated runs (morning check-ins, autodreams, etc.), two things matter:

**Dynamic paths.** Session IDs change on every run, so hardcoded paths break immediately. Your scheduled task prompts should locate palace files relative to the palace root rather than embedding a specific session or machine path: for example `find "$PALACE_ROOT" -maxdepth 3 -name "PALACE.md"`, where the root is the one place you name once (the `Palace:` line in your identity block, or the `LOCI_PALACE` environment variable if you use the CLI).

**SOUL.md first.** Your AI's SOUL.md must be included in every scheduled task prompt: it is what makes the output feel like your collaborator rather than a generic assistant. Read it before any state files.

---

## Folder structure (what you'll end up with)

```
my-palace/
  PALACE.md                    ← master prompt, primary, AI-agnostic (start here)
  CLAUDE.md                    ← two-line pointer to PALACE.md (for Claude Code)
  tracker.json                 ← project tracking
  soul/
    SOUL.md                    ← your AI's character
    handovers/
      2026-03-20.md            ← session memories build up here
  rooms/
    work-room/
      CLAUDE.md                ← work room context
    writing-room/
      CLAUDE.md                ← writing room context
    ideas-room/
      CLAUDE.md                ← ideas room context
  _templates/                  ← the templates folder (keep for reference)
```

**If your agent set you up** (via `AGENT-SETUP.md`), you also have `soul/garden.md` (the garden, seeded from the interview), and possibly `souls/` (extra personas), `friends/` (soul files from friends), and `palace-map.canvas` (an Obsidian mindmap). Same palace, a few more shelves; nothing above changes.

---

## One last thing

The palace grows with you. It starts simple and gets richer over time. Don't try to build the perfect system before you start. Start working and let the system emerge from what you actually need.

The best version of your AI is the one that's been through a few dozen sessions with you and knows how you think.

Start there.

---

*Loci · agent-first memory palace kit · [loci.garden](https://loci.garden)*
*"A collaborator, not a tool."*
