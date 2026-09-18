# Your First Session: A Quickstart Card

---

> **Never set up a palace before?** Run the first session one of two ways, both the same naming
> ceremony: paste the self-contained block in `templates/first-session-protocol.md` (nothing else
> needs to be in front of the agent), or point an agent that can read this repo at `AGENT-SETUP.md`.
> This card is the everyday opener for the sessions after that.

---

## Before you start

**If you ran agent setup** (i.e. you told Claude to read this repo and run the setup): your files are already written. Skip straight to "How to start a session" below.

**If you're setting up manually**: make sure you've filled in at minimum:
- ✅ `templates/CLAUDE-master.md` → filled in and saved as `PALACE.md` (the primary file), with a two-line `CLAUDE.md` pointer beside it
- ✅ `templates/SOUL.md` → saved as `soul/SOUL.md`
- ✅ At least one room file → `rooms/[room-name]/CLAUDE.md`

See `SETUP-GUIDE.md` for the full walkthrough, or `AGENT-SETUP.md` to have Claude do it for you.

---

## How to start a session

**In Cowork**, open a new conversation and paste this, replacing the bracketed parts:

---

> Read `PALACE.md` to understand who you are and how we work.
> Then read `soul/SOUL.md` for your character and any prior history.
> Then read `rooms/[ROOM NAME]/CLAUDE.md` for this session's context.
>
> When you're ready: wake up as [YOUR_AI_NAME], state the room we're in, and ask me for the 2-line state summary.

---

That's it. Claude will read the files, introduce itself in character, and ask you where things were left off.

**In Claude Code**, open a terminal in your palace folder and run `claude`. Your `CLAUDE.md` pointer is read automatically and sends me to `PALACE.md`, so there is nothing to paste. Say the wake-up line: *"Wake up [YOUR_AI_NAME]. We're in [ROOM NAME] today."*

If it's session 1 and there's no history, just say: *"First session, no prior state."* Then tell it what you want to work on.

---

## How to end a session

When you're done working, say:

> "Write the session delta."

Claude will write a structured handover to `soul/handovers/YYYY-MM-DD.md`: what was done, decisions made, what's next.

Next session, that file will be the bridge.

---

## Useful things to say during a session

| Say this | What happens |
|---|---|
| `"Write the delta"` | Claude writes the session handover |
| `"Add this as a crystal"` | Claude adds a confirmed fact to your context |
| `"We're switching rooms"` | Claude loads the new room context |
| `"Plan mode"` | Claude stops and writes a plan before acting |
| `"KISS it"` | Claude backs up and finds a simpler solution |
| `"Update the tracker"` | Claude updates tracker.json with current status |
| `"What's open?"` | Claude surfaces open blockers and next actions |

---

## If something feels off

- **Claude is being too wordy:** Say "shorter, match my pace"
- **Claude made an error:** Say "that's wrong, [here's what's right]. Remember this."
- **Claude is over-explaining:** Say "skip the preamble, just do it"
- **You want to adjust the setup:** Edit `PALACE.md` directly. It's just a text file

---

## Growing the palace over time

After 3–5 sessions, you'll start to notice:
- The deltas building up in `soul/handovers/`: a real history
- The crystals getting richer: Claude knows more about your world
- The AI feeling more like a collaborator and less like a fresh tool

That's when it starts to feel real. Give it those sessions.

---

*Good luck. You're going to build something good together.*

*Loci · [loci.garden](https://loci.garden)*
