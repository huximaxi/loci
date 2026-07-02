# Skill templates

*Portable disciplines that survived the test in real palaces and are worth
lifting into yours.*

A **skill** in this kit is not a tool or a binary. It is a written procedure
with a stated trigger, a verification, and a kill condition. A discipline you
can teach a palace to apply consistently, then prune if it stops earning its
place.

## What's on the shelf

| Skill | Owner posture | File |
|---|---|---|
| **Foreign-process quarantine** | Security / adversarial (Cipher) | `quarantine.md` |
| **Session close** | Orchestration (Vesper) | `session-close.md` |
| **Insight consolidation** | Orchestration (Vesper) | `insight-consolidation.md` |

## How to use a skill template

1. Read it. Decide whether the discipline fits your palace.
2. If it does, copy it into your palace under `skills/<skill-name>.md` (or
   wherever your palace stores reusable disciplines).
3. Adapt the procedure to your concrete surfaces. The trigger and the kill
   condition should stay; the steps will need your context.
4. Name the persona who owns the discipline (if your palace has named
   personas). The persona reads the skill when their trigger fires.

## How to write a new skill template

Every skill in this shelf has the same five sections:

- **What it does**: the one-paragraph doctrine
- **When to apply**: the explicit trigger
- **The procedure**: numbered, idempotent, runnable cold
- **Verification**: how to know it worked
- **Kill condition**: when to retire the discipline

Skills that don't survive their own kill condition leave the shelf. The point
of the discipline is to earn its place every time it fires.

## Why "skill" and not "process"

A process is a thing a team agrees to do. A skill is a thing a collaborator
*becomes good at*. The template is meant to teach the latter: a discipline
that fires from a trigger, runs cleanly, and leaves the palace better than it
found it. The Greek `praxis` shape: skilled action whose end is the doing.
