# Persona templates

*A shelf of filled-out persona examples, ready to fork into your own palace.*

`templates/persona-template.md` is the empty skeleton. The files here are
**filled examples**: actual personas, scrubbed of any single palace's
specifics, that you can copy into your own palace and adapt.

## Why personas

Most work is fine with one collaborator: your main AI, by its name. Add a
persona when a recurring kind of work wants a different posture, not just a
different prompt. The persona is *the same collaborator awakened differently*,
not a fork.

## The engine-room dyad

This shelf ships with a paired example: **Cipher** (security / adversarial)
and **Praxis** (sysadmin / reversibility / runbooks). They are designed to
work together. Aristotle's distinction is the seam:

- **Cipher** does *theoria*: contemplating the perimeter. *What would have to
  be true for this to be false?*
- **Praxis** does *praxis*: skilled action whose end is the doing itself.
  *The fix without a rollback path is not fine.*

Cipher finds what breaks. Praxis keeps things running and turns the findings
into shipped procedure. The tension is generative.

You don't need both. You don't need either. They are an example of a working
pair you can lift directly into your palace and adapt.

## How to use a template

1. Copy the file into your palace under `souls/<persona-name>.md` (or wherever
   your palace stores persona souls).
2. Adapt the working principles to your domain.
3. Add the persona row to your master prompt's roster (see
   `templates/CLAUDE-master.md` § Persona Roster).
4. Invoke by name when the work calls for that posture. Never auto-invoke;
   confirm first.

## How to write a new one

Start from `../persona-template.md` (the empty skeleton), or fork one of these
filled examples and rewrite. The shape is small: name, who-I-am, working
principles, what-I-don't-do. The point of the soul file is to make a persona
*remember*, not to construct it from a paragraph at session-open.

## Naming

Greek, mythological, conceptual. One word. Gender-ambiguous. Memorable. A
name the persona answers to, not a job title.

## What's on the shelf

| Persona | Domain | File |
|---|---|---|
| **Cipher** | Security, threat modelling, adversarial analysis | `Cipher.md` |
| **Praxis** | Sysadmin, deploys, runbooks, reversibility | `Praxis.md` |
