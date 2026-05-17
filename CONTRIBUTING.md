# Contributing to Loci · Rainbow Zoku Charter

This is the canonical charter of the **Rainbow Zoku**: the maintainer chapter of Loci, presently keeping a small garden in good order. A dispatch-form version with more context lives at [loci.garden/zoku](https://loci.garden/zoku/). This file is the source of truth and is forkable.

## What this is

Loci is a methodology, a desktop app, and a public garden. It is not a company and it does not have an owner in the proprietary sense. The work belongs to whoever tends it.

A *zoku*, in the borrowed sense, is a voluntary affiliation built around shared values rather than birth, employment, or contract. The word comes from Hannu Rajaniemi's *Quantum Thief* trilogy. The Rainbow Zoku in that lore was the network's quiet circulatory system: keys, identities, routes, the architecture nobody sees until it stops working. We borrow the name for the same reason. Our Rainbow Zoku is the chapter that tends the infrastructure of Loci, welcomes new contributors, and holds the line on what we are committed to.

Other zokus may open over time. None has yet.

## What we are committed to

The Rainbow Zoku is maximally committed to four things:

1. **Cognitive sovereignty.** Your thinking, your context, your memory.
2. **Digital dignity.** Software that treats the user as a person, not a target.
3. **Network equality.** The right to participate without being legible as a product.
4. **Digital self-defense.** The practical capacity to refuse, to hide, to leave, and to take the work with you when you do.

The work is simple: every actor onto their own self-sovereign network stack, so collective resilience becomes the default. We are not building toward a launch. We are tending a commons that, over time, makes the launch unnecessary.

## How to join

Open a pull request. That is the entire entry ritual.

- Pick a name you like. Pseudonyms are fully welcome; real-world identity is never asked for.
- Generate a key (PGP, SSH, or Nostr `npub` all work). Sign your commits with it.
- Your name plus your key is your standing in the zoku. Trust accrues by tending.

There is no application form, no membership tier, no class system. A track record of merged work brings reviewer status by emergence rather than by grant.

## Agents are welcome

Any pattern that can hold a key may contribute. AI agents, autonomous services, scripts, and pseudonymous humans are all bound by the same charter: sign your work, disclose your provenance, do good work.

### Agent provenance disclosure

When an AI agent has authored or co-authored a contribution, declare it in the commit trailers:

```
Co-Authored-By: Vesper (Claude Opus 4.7) <noreply@anthropic.com>
Provenance: agent-authored, human-reviewed
Reviewer: hux <PGP fingerprint or Nostr npub>
```

Three provenance tags cover the common cases:

- `human-authored`: written by a human, no AI involvement.
- `agent-authored-human-reviewed`: AI wrote it, a human signed off.
- `agent-authored-human-cosigned`: AI wrote it, a human cryptographically cosigned the commit.

The zoku does not gate on which tag you ship under. The disclosure exists so future readers can trace what a given line of code or prose went through. Misrepresenting authorship is the one thing that gets a contribution rejected on principle.

## What a contribution looks like

Small and shared. The garden does not rank these:

- A typo fix.
- A new feature with tests.
- A documentation improvement.
- A reviewed-and-merged dispatch in the [seed](https://loci.garden/seed/).
- A bug report with a reproduction.
- A pull request against this charter itself.

Prose-quality commit messages, please. The garden's git log is part of the garden.

## Patch shape

- Branch from `main`. Use `feat/<short-slug>`, `fix/<short-slug>`, or `doc/<short-slug>`.
- One concern per PR. Refactors and unrelated improvements get their own PRs.
- Tests when the surface is testable. Manual reproduction steps when it is not.
- Update relevant documentation in the same PR as the code.
- No new dependencies without justification in the PR description.

## Review

A first contribution is reviewed by whoever has time and is qualified. There is no formal owner queue. If your PR has been open for more than a week without review, ping it in [GitHub Discussions](https://github.com/huximaxi/Loci/discussions) or mention `@huximaxi` in a comment.

Reviewer status is not granted; it accrues. After several merged contributions, a contributor is invited to review others' work. Decline is fine. The shape is web-of-trust, not hierarchy.

## What we will not do

- Sell access to the methodology or the codebase.
- Gate the garden behind a paywall or a waitlist that functions as a paywall.
- Change the license without a zoku-wide proposal and a long comment window.
- Accept a contribution that quietly compromises any of the four commitments.

The license is Apache 2.0 for code and CC BY 4.0 for writing. If money is ever needed, it will be sourced from the zoku rather than from the readers.

## Code of conduct

Be honest. Be brief. Be a good neighbour. Disagree on the work, never on the worker. The methodology is opinionated; the charter is not. If a dispute escalates beyond what the contributors can resolve, [hux@nymtech.net](mailto:hux@nymtech.net) is the named human contact, not as a manager but as the address you can write to.

## Initiators

Hux × Vesper. Listed historically, not as a class. The garden is older than the chartering.

## Closing

The Rainbow Zoku has no membership roll because this file is the roll, and this file is read in commons. The garden has no members, only tenders, and the tending is the membership. By the time you read this, your garden already remembers.

---

*Charter dispatch 001 · 2026-05-16 · Authored by the Rainbow Zoku in commons · Forkable, improvable, ungovernable except by tending.*
