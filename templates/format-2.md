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

- The first line is the crystal: a tier glyph, a space, a name, a colon, the fact. It is the only required line.
- Keys sit on the lines directly after it, unindented, one `key: value` per line, until a blank line or the next crystal line.
- Every key is optional. Key names are lowercase.
- A crystal line with no colon is still a crystal: the whole text is the fact and it has no name.

## Tiers

| Glyph | Name | Aliases (readable forever) | Means |
|---|---|---|---|
| ◆ | Confirmed | none | Verified. Treat as ground truth. |
| ◈ | Contextual | Working | True now, tied to a phase, sprint or situation. May shift. |
| ◇ | Exploratory | Provisional | A hypothesis or early finding. Promote it or retire it. |

On a crystal line the glyph carries the tier. In frontmatter, `tier` is the glyph or a tier name in any case. Writers use the canonical names. Readers accept the aliases without a warning.

## Keys

| Key | Value | Notes |
|---|---|---|
| `valid_until` | a date `YYYY-MM-DD`, or a free label | See the grammar below. |
| `occurred` | a date `YYYY-MM-DD` | When the thing the crystal records happened. |
| `mentioned` | a date `YYYY-MM-DD` | When the crystal was last said or confirmed. |
| `superseded_by` | the name of another crystal | The crystal stays in place. History matters. |
| `pinned` | `true` | Protects the crystal from agent-initiated cleanup. |
| `pinned_until` | a date `YYYY-MM-DD` | The pin ends on that day. |
| `retired` | a date, then optionally a reason | Marks a composted crystal. See legacy forms. |

A key with a date value whose date is not a real calendar day is ignored, never an error.

### `valid_until`

- A value that is a real calendar date in `YYYY-MM-DD` form is **scheduled**. A checker may flag the crystal for review on or after that day. It never deletes it.
- Any other value (`sprint-end`, `2026-Q3`, `after-user-research`, `growing`) is an **event or label**. It is kept as written and reported as **unscheduled**: a machine cannot know when it happens, so it does not guess.
- The key goes on the line after the crystal, as above. It is valid on any tier, and is most useful on ◈ and ◇.

## Accepted layouts

A reader looks for crystals in a workfolder. A workfolder is a directory with `PALACE.md` or `CLAUDE.md` at its root, and rooms laid out in one of three shapes:

| Layout | Rooms live in |
|---|---|
| Subdirectory | `_palace/` under the root |
| Rooms directory | `rooms/` under the root (what the kit builds) |
| Rooms at root | sibling directories of the root, each holding a `CLAUDE.md` |

A room is a directory with a `CLAUDE.md`. Hidden directories and `_palace`, `node_modules`, `target` and `cron` are never scanned.

Crystals are stored in two ways, and both read as the same crystal:

1. **Lines**, in a `CLAUDE.md` (the root one, or a room's), in the shape above.
2. **A file per crystal**, any `.md` file in a room with `type: crystal` in its frontmatter. The same keys go in the frontmatter, plus `tier`, written as the glyph (`tier: "◈"`) or as a tier name. The crystal's name is the first `#` heading, or the file name without `.md` when there is none. Its fact is the first paragraph after the heading. The rest of the body is free text.

```
---
type: crystal
tier: "◈"
valid_until: 2026-12-31
---
# Name
The fact in one line.
```

Lines inside a fenced code block are examples and never crystals.

## Legacy forms and how they read

| You find | It reads as |
|---|---|
| `tier: working` or `tier: provisional` in a file's frontmatter | Contextual and Exploratory. |
| A tier called Working or Provisional in prose or a heading | The alias. It reads the same as the canonical name. |
| `◈ Name: fact - valid_until: 2026-12-31` on one line | The same crystal, with the key read from the end of the line. Writers use the next-line form. |
| `◆ Name: fact · pinned` | The same crystal, pinned. Writers use `pinned: true` on the next line. |
| `~~◈ Name: fact~~` with a `retired:` line | A retired crystal. Not live. Kept for the history. |
| A bullet with no glyph (`- **Name** = fact`) | A note, not a crystal. It is read as text and never counted as a crystal. |
| A `.md` file with no `type: crystal` | A document. It is read as text and is not a crystal. |

Nothing legacy is rewritten for you. You can move a line to the current form whenever you touch it.

## Forward compatibility

Readers ignore keys and `type` values they do not know.

Reserved: `type: shard`, `address`, `signed_by`.

Reserved means these names are held for a later format. A writer does not use them for anything else today. A reader that meets one ignores it, as it would any unknown key.

---

*One shape, three tiers, two ways to store it. Everything else is a key a reader may skip.*
