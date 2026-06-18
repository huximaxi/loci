# loci feature helper

> Agent-executable process. Trigger: "what can loci do?" / "which features do I need?" /
> "help me set up loci for X" / "feature helper".
>
> Plain text, engine-agnostic. Any file-aware LLM runs it. No build, no install.

## What this does

Walks a person from "here is what I want to do" to the smallest set of loci features that
does it, and points them at a tutorial. It reads `features/features.yaml` as its map. It
proposes; the human chooses. One step at a time, always skippable.

## The substrate, in one line

loci is an intelligence substrate: plain-text firmware (the `templates/` kit plus a few
named processes) that any file-aware AI can run. The desktop app and the CLI are personal
demos built on top of it. You do not need either to start.

## Protocol

```
TRIGGER
   │
   ▼
1. ASK (one question, skippable)
   "What do you want your AI to do better? A few words is enough."
   If the person already named a use-case (researcher / builder / companion-keeper /
   sovereign / team / migrator), skip to step 3.
   │
   ▼
2. MATCH to a use-case in features.yaml
   Read the use_cases list. Pick the closest by its `for:` line. If two fit, name both
   and ask which is closer. Never invent a use-case that is not in the file.
   │
   ▼
3. SHOW the smallest feature set that does the job
   List the use-case's `primary` feature sets first, each as: name, value, what it does.
   Add the `supporting` sets in one line below ("also leans on: ...").
   Stay inside what the file holds. Do not promise features not in features.yaml.
   │
   ▼
4. POINT at the tutorial
   Offer the use-case's `start:` tutorial, then the per-feature `tutorial:` links.
   Ask: "Want to walk the first one now?"
   │
   ▼
5. (optional) WALK the tutorial
   Open the tutorial file and follow it one move at a time. Stop on "skip".
```

## Rules

- Read `features/features.yaml` before answering. It is the source of truth, not memory.
- Smallest set first. Name the one or two feature sets that do the job, not all seven.
- Propose, never apply. Building a palace is the person's call, file by file.
- No feature that is not in the file. If asked for something loci does not ship, say so plainly.

---

*Process by loci · v0.1 · [loci.garden](https://loci.garden)*
