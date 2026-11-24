---
created: 2026-11-23
version: 2.0
type: spec
---

# loci format 2

*What a crystal is on disk, so that every reader (a person, an agent, `loci`) reads the same thing.*

This page is the contract. `crystals-guide.md` teaches when to use each tier; this page says exactly what is written and how it is read. If the two ever disagree, this page wins and the guide is the bug.

---

## One crystal

```
◈ Name: the fact in one line
valid_until: 2026-12-31
occurred: 2026-11-01
mentioned: 2026-11-05
superseded_by: Other name
```

- The first line is the crystal: a tier glyph as the very first character of the line, a space, a name, a colon and a space, the fact. It is the only required line.
- The name is everything before the first `: ` (a colon followed by a space). A line with no `: ` is still a crystal: the whole text is the fact and it has no name. So `◆ Standup at 10:30 daily` is a nameless fact.
- Keys sit on the lines directly after it, unindented, one per line, in the form `key: value` with a lowercase key (letters and underscores). Every key is optional.
- The key block ends at the first line that is not a key line: a blank line, the next crystal, or any prose (a `→ Promoted to ◆ ...` note, say). Keys written after that line are not read.
- A value is the text after `: `, trimmed. Nothing after it on the line is a comment: write none.

## Tiers

| Glyph | Name | Aliases (readable until 3.0 at least) | Means |
|---|---|---|---|
| ◆ | Confirmed | none | Verified. Treat as ground truth. |
| ◈ | Contextual | Working | True now, tied to a phase, sprint or situation. May shift. |
| ◇ | Exploratory | Provisional | A hypothesis or early finding. Promote it or retire it. |

On a crystal line the glyph carries the tier. In frontmatter, `tier` is the glyph or a tier name in any case. A file crystal whose `tier` is missing or unreadable reads as Exploratory, the least trusted tier. Writers use the canonical names. Readers accept the aliases without a warning.

## Keys

| Key | Value | Notes |
|---|---|---|
| `valid_until` | a date `YYYY-MM-DD`, or a free label | See below. |
| `occurred` | a date | When the thing the crystal records happened. |
| `mentioned` | a date | When the crystal was last said or confirmed. |
| `superseded_by` | the name of another crystal | The crystal stays in place. History matters. |
| `pinned` | `true` | Protects the crystal from agent-initiated cleanup. |
| `pinned_until` | a date | Protects it up to and including that day. Enough on its own, with no `pinned: true`. Normal review resumes the next day. |
| `retired` | a date, then optionally a space and a reason | Marks a composted crystal. The date is the first word. |

A date is a real calendar day written `YYYY-MM-DD`. A value that is not a date (`2026-02-30` included) is not a date: for `valid_until` it reads as a label, for every other date key it is ignored. Never an error. `retired` is the exception: its date is the first word and the rest is a reason.

`superseded_by` matches a crystal name without regard to case, within the same file or room first, then the workfolder. A reader that cannot find the name, or finds several, still marks the crystal superseded.

### `valid_until`

- A real `YYYY-MM-DD` date is **scheduled**. A checker may flag the crystal for review on or after that day, and may warn earlier (the guide's morning check-in example warns seven days ahead). It never deletes it.
- Any other value (`sprint-end`, `2026-Q3`, `after-user-research`, `growing`) is an **event or label**. It is kept as written and reported as **unscheduled**: a machine cannot know when it happens, so it does not guess.
- It is valid on any tier and is most useful on ◈ and ◇.

## Accepted layouts

A workfolder is a directory with `WORKFOLDER.md`, `PALACE.md` or `CLAUDE.md` at its root and a scan root. The three markers are read the same way. A room is a directory holding a `CLAUDE.md`. The **scan root** is the directory the rooms live in. A reader picks the first that applies:

| Order | Scan root | When |
|---|---|---|
| 1 | `_palace/` under the root | the directory exists, even with no room in it |
| 2 | `rooms/` under the root | it exists and holds a room (what the kit builds) |
| 3 | the root itself | at least one subdirectory of the root holds a `CLAUDE.md` |

If more than one shape is present, the first wins and the rest are not read. Within the scan root, hidden directories and `node_modules`, `target` and `cron` are not scanned.

Crystals are stored in two ways, and both read as the same crystal:

1. **Lines**, in the root `WORKFOLDER.md`, `PALACE.md` or `CLAUDE.md`, or in a room's `CLAUDE.md`, in the shape above.
2. **A file per crystal**: any `.md` file with `type: crystal` in its frontmatter, under the scan root or under `garden/` at the workfolder root (the first-session ceremony writes there, and `garden/` is a sibling of `rooms/`, not inside it). It need not sit in a room. The same keys go in the frontmatter, plus `tier`, written as the glyph (`tier: "◈"`) or as a tier name. The crystal's name is the first `#` heading, or the file name without `.md` when there is none. Its fact is the first paragraph after the heading. The rest of the body is free text.

```
---
type: crystal
tier: "◈"
valid_until: 2026-12-31
---
# Name
The fact in one line.
```

Frontmatter keys and `type` values are matched without regard to case. Line keys must be lowercase to count as key lines. Fenced code blocks (opened with three backticks or three tildes, closed by the same) hold examples: nothing inside one is a crystal. A fence that is never closed runs to the end of the file.

## Legacy forms and how they read

| You find | It reads as |
|---|---|
| `tier: working` or `tier: provisional` in a file's frontmatter | Contextual and Exploratory. |
| A tier called Working or Provisional in prose or a heading | The alias. It reads the same as the canonical name. |
| `◈ Name: fact - valid_until: 2026-12-31` on one line | The same crystal, with the key read from the end of the line and removed before the name is split. Writers use the next-line form. |
| `◆ Name: fact · pinned` | The same crystal, pinned. Writers use `pinned: true` on the next line. |
| `~~◈ Name: fact~~` (the whole line struck through), usually with a `retired:` line after it | A retired crystal. Not live. Kept for the history. |
| A bullet with no glyph (`- **Name** = fact`) | A note, not a crystal. It is read as text and never counted as a crystal. |
| A `.md` file with no `type: crystal` | A document. It is read as text and is not a crystal. |

Nothing legacy is rewritten for you. You can move a line to the current form whenever you touch it.

## Forward compatibility

Readers ignore keys and `type` values they do not know.

Reserved: `type: shard`, `address`, `signed_by`.

Reserved means these names are held for a later format. A writer does not use them for anything else today. A reader that meets one ignores it, as it would any unknown key.

---

*One shape, three tiers, two ways to store it. Everything else is a key a reader may skip.*
