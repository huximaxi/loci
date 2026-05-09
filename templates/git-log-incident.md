# Output Primitive — Git Log Incident
*Atomic format for session arcs, investigations, and collaborative sequences.*
*Introduced: v0.9 · 2026-05-08 · Hux × Vesper · "pure gold."*

---

## What this is

A structured output format for documenting self-contained arcs of work — investigations, deployments, research threads, onboarding sequences — in the style of an annotated git log. Each discrete action or finding is a "commit." The log is honest: the 400 before the 200 is part of the story.

Copyable as markdown, renderable as HTML, diffable across time.

---

## When to use

Any self-contained arc that has:
- A clear start (discovery/trigger)
- Multiple collaborators or agents
- At least one plot twist, blocker, or unexpected finding
- A resolution

Good triggers: bug investigations, cross-team ticket arcs, deployment incidents, research threads that took turns, onboarding sequences.

---

## Format spec

```
# [TITLE] · Git Log Incident
*git log --all --oneline --[adjective] · [date]*

---

## [SHORT_HASH] · [Author] · [HH:MM TZ]  `[TAG]`
### [verb(scope): commit message — dry, precise, one line]

[Body — 3-6 lines. What actually happened. First person or third person,
consistent. Dry wit welcome. No bullet points — prose or short declarative lines.
The humor emerges from stating facts plainly, not from jokes.]

---
[repeat per commit]

---

*[N] commits · [stat] · [stat] · [stat] · [closing note]*
```

---

## Tag vocabulary

| Tag | Use when |
|---|---|
| `[DISCOVERY]` | Something found that changes the frame |
| `[FATAL DISCOVERY]` | Something found that demands immediate action |
| `[TICKET]` | A Jira/Linear/GitHub issue created or updated |
| `[PLOT TWIST]` | A collaborator challenges the premise |
| `[FALSE NEGATIVE]` | Investigation returns 0 when 1 exists |
| `[SALVATION]` | Someone outside the thread provides the answer |
| `[FORENSICS]` | Root cause identified |
| `[HTTP 400]` / `[HTTP 500]` | A tool failure worth naming |
| `[DELIVERED]` | Final action successfully executed |
| `[HUXGATE]` | Human approval moment in the arc |
| `[NOISE]` | Something that looked like signal |
| `[SIGNAL]` | Something that looked like noise |

The vocabulary is not closed. Add new tags as arcs demand them.

---

## Authorship convention

- **[Agent name]** — synthesis, coordination, API calls, file operations
- **[Cipher / security persona]** — adversarial analysis, security findings
- **[Design persona]** — design observations
- **[Recon persona]** — data extraction, brute-force passes
- **[Name] ✨** — external collaborator who dropped the key insight
- **[Name]** — team member who contributed a turn
- **[Human]** — decisions, approvals, the gate moments

---

## Output options

**HTML widget:** render via your agent's visualization tool using the monospace card layout — dark secondary background, commit blocks separated by 0.5px borders, hash in accent color (#7F77DD or equivalent), tags as small colored badges.

**Markdown file:** write using the format spec above verbatim. Copyable, diffable, archivable.

**Both:** widget first (immediate feedback), then save the MD for the archive.

---

## The atomic principle

Each commit = one discrete action or finding.
No commit covers more than ~5 minutes of real time.
The log is honest: if something failed first, it gets its own commit.

*"The humor emerges from stating facts plainly."*
