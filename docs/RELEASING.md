# Releasing loci

How a change ships to this repository. This is the contributor-facing half: the
ordered gates, the pre-flight checklist, version semantics, what triggers a
CHANGELOG or README touch, and the rollback path. Maintainers run an additional
operator pass that wires in the local pre-commit and pre-push checks; that half
lives outside this repository.

The rule beneath every rule: **a release is reversible, gated, and rewritten from
intent rather than copied wholesale.** If a step exists only in someone's memory,
it is not a step yet.

---

## The load-bearing invariant

> The public repository is self-contained. It builds and runs with nothing
> private, and it references nothing private. The dependency arrow points one
> way only: private work may consume the public surface, never the reverse.

Every release is checked against this. A change that quietly imports a private
path, an internal name, or an unreleased-roadmap marker does not ship, even if it
is otherwise correct.

---

## Release is re-expression, not transplantation

Public artifacts are written from a specification, not pasted from a private
working copy. This is a discipline, not a formality: prose rewritten for a public
reader carries no accidental internal vocabulary, no private file path, and no
stale internal reference. When you find yourself copying a block from somewhere
non-public, stop and rewrite it for the reader who has only this repository.

---

## The ordered gates

A change passes through these in order. Each gate can send the change back; none
is skipped silently.

```
  1. SCOPE        one concern per release; branch named for it
        │
  2. PRE-FLIGHT   the checklist below, top to bottom
        │
  3. BLEED GATE   automated scan of the staged diff (pre-commit)
        │            ↳ blocks on a private marker; no silent bypass
        │            ↳ + advisory staleness nudge (offline: "remote moved N")
  4. HUMAN READ   a maintainer reads the full diff for structural bleed
        │            ↳ words a scanner can't catch: shapes, names, roadmap
  5. COMMIT        explicit maintainer approval required (see "The commit gate")
        │
  6. RECON GATE   adversarial reconstruction pass on the outbound range (pre-push)
        │            ↳ shows what the diff teaches an outsider; advisory by default
        │            ↳ + advisory divergence check (fetches upstream, warns if stale)
  7. PUSH / PR     open the PR; fill the template; request review
        │
  8. MERGE         maintainer gate in the PR template must be checked
        │
  9. POST          tag if releasable; verify; watch for the rollback trigger
```

Gates 3 and 6 are automated and described in "How the checks wire in". Gates 4
and 5 are human and non-negotiable.

---

## Pre-flight checklist

Run this top to bottom before you commit. It is ordered so the cheap checks fail
fast.

- [ ] **One concern.** This release does one thing. Unrelated improvements get
      their own branch and PR.
- [ ] **Branch named for it.** `feat/<slug>`, `fix/<slug>`, or `doc/<slug>`,
      branched from `main`.
- [ ] **Current base.** `git fetch` and confirm your branch is not built on a
      stale `main`. If the remote moved while you worked, rebase before you push.
      (See "Staleness checks" below; a local check warns you if you forget.)
- [ ] **No em-dashes.** Zero em-dashes ship. Search the diff for `—` and replace
      each with a colon, parentheses, or a new sentence. (See "Em-dash
      discipline".)
- [ ] **Re-expressed, not transplanted.** Every added block was written for a
      reader who has only this repository. No pasted-in private prose.
- [ ] **CHANGELOG decided.** Either the change has a CHANGELOG entry at the top,
      or you can state in one sentence why it does not need one. (See "What
      triggers a CHANGELOG entry".)
- [ ] **README decided.** Same: either README is touched, or you can say why the
      change does not reach it. (See "What triggers a README touch".)
- [ ] **Version decided.** If this changes the methodology surface, the version
      anchor moves and you know whether this is a candidate or a stable
      graduation. (See "Version semantics".)
- [ ] **Docs in the same change.** Any behaviour or surface a contributor relies
      on is documented in the same release, not a follow-up.
- [ ] **Invariant holds.** No private path, name, or unreleased-roadmap marker in
      the diff. The bleed gate enforces the obvious cases; you confirm the rest.

---

## The commit gate

A commit is an explicit decision, not a reflex.

- **A maintainer approves the commit.** The automated bleed gate having passed is
  necessary, not sufficient. A human confirms the change is ready before it is
  committed.
- **Run a quick pre-mortem first.** One pass asking "what could this break, and
  how would I undo it?" The answer to the second half is the rollback path
  below; if you cannot state it, you are not ready to commit.
- **Never bypass a failing check silently.** If the bleed gate blocks, fix the
  lines. The override path exists for genuine false positives only, it is logged,
  and using it is a maintainer decision with a stated reason, never a habit.
- **No identifying contact details in commit messages.** Keep commit trailers to
  authorship and provenance per `CONTRIBUTING.md`; do not embed email addresses.

---

## Em-dash discipline

Zero em-dashes ship. Not in prose, not in code comments, not in CHANGELOG
entries, not in documentation. Use a colon, parentheses, or a new sentence
instead. Before committing, search the diff for the em-dash character and clear
every one. This is a house style that is checked on read; treat it as part of the
diff being correct.

---

## Version semantics

The methodology line of loci is versioned in `PALACE-METHODOLOGY.md` through a
parseable anchor near the top of the file:

```
> loci-core version: X.Y-candidate
> stable: X.(Y-1)
> status: candidate
> updated: YYYY-MM-DD
```

This anchor is read by the update checker. It is a blockquote with three labelled
fields, not YAML front matter, and the parser depends on the exact field names
(`loci-core version`, `stable`, `status`). Keep the shape exactly.

**The rule for when to move it:**

| Change | Anchor move |
|---|---|
| A new or revised methodology concept, process, template, or operating rule | New dated section under the latest `vX.Y-candidate` heading; `status: candidate`. Bump the *candidate* number if this opens a new line of work; otherwise extend the current candidate section. |
| A bug fix, typo, or wording pass that does not change the methodology surface | No anchor move. Note it in CHANGELOG if a contributor would notice it. |
| A candidate has held up in real use and is ready to be relied on | **Graduate:** set `stable: X.Y`, set `status: stable`, and the `loci-core version` line names the now-stable version. Open the next `-candidate` only when the next change lands. |

**Candidate vs stable, the discipline:** a version graduates from candidate to
stable when it has been used against a real palace (not just authored) and nothing
needed walking back. Candidate is the honest default for freshly-written
methodology: it says "this is the current line, and it has not yet earned the
weight of stable." Do not graduate a candidate in the same change that introduces
it.

Always set `updated:` to the release date when the anchor moves.

---

## What triggers a CHANGELOG entry

`CHANGELOG.md` follows Keep a Changelog. Add an entry at the top when the change
is something a user or contributor would notice:

- A new feature, process, template, or methodology concept.
- A change to existing behaviour or to a documented surface.
- A new file a user is expected to copy or read.
- A fix for something a user could have hit.

You do **not** need an entry for: internal refactors with no surface change,
pure formatting passes, or changes to maintainer-only tooling that does not ship
in the repository. When in doubt, add the entry; an over-documented changelog is
cheaper than a silent surface change.

Group entries under the standard headings (Added / Changed / Fixed / Removed /
Notes). Desktop and extension app releases are versioned and tagged separately
from the methodology line; keep their entries distinct.

---

## What triggers a README touch

Touch `README.md` when the change alters:

- The quick-start path or the setup door.
- The list of packages or their stated status / version.
- A front-page feature description (what a shelf or door offers).
- A top-level claim about what loci is or does.

A methodology-internal change that a first-time reader of the README would never
see does not need a README touch. The README is the front door; edit it when the
front door changes.

---

## How the checks wire in

Two automated gates run locally. Both are installed by maintainers and both have
a documented off-ramp; neither is a black box.

**Gate 3, bleed scan (pre-commit).** A scan of the *staged diff* for private
markers (internal names, private paths, gate words). It blocks the commit if it
finds one. It scans only added lines, to keep false positives low. If it blocks
on a genuine false positive, the override is explicit, requires a stated reason,
and is logged; it is never the silent default.

**Gate 6, reconstruction pre-mortem (pre-push).** An adversarial pass over the
range about to be pushed. It asks what an outsider could reconstruct about the
private system from this public diff alone, even when no banned word appears, and
returns an inventory ranked by confidence. By default it is **advisory**: it
prints the inventory and lets the push proceed, so you see what you are about to
teach. It can be set to block on a high-confidence structural finding. It only
ever runs against this repository's public origin; it refuses to run on anything
else and sends nothing.

Both gates are mirrors as much as guards. The point is that you see the change
the way an outsider will, before it is irreversible.

**Staleness checks (advisory, never block).** Two small local checks warn you when
the remote has moved under you, so a stale base is not a surprise at push time.
At commit time, an offline nudge reads the last-fetched state and, if the upstream
is ahead of where you branched, prints a one-line reminder to fetch. At push time,
a check fetches the branch's upstream and, if it has advanced past your merge-base,
warns that you are building on a stale base and suggests `git fetch && git rebase`
before pushing. Both are advisory: they warn and let the operation proceed. Neither
needs network access beyond the ordinary fetch, and neither sends anything
anywhere. They exist because a remote that moved mid-work is easy to miss until the
push is rejected or the PR conflicts.

---

## Tagging a release

When a change is a release (not every merge is), tag it from the merge commit:

- Annotated tag, minimum. Signed tag once a maintainer key is published.
- Tag from the merged commit on `main`, named for the version it carries.
- Methodology, desktop app, and extension are tagged on their own version lines.
- A release-candidate tag stays forever. If `rc.2` ships tomorrow, `rc.1` remains
  in history. The trail of how a release reached final *is* the release-quality
  record. Never force-move a tag.

---

## Rollback

Reversibility is designed in, not improvised after. Before any push, you can
state the undo. The path depends on how far the change travelled:

| Stage reached | Rollback |
|---|---|
| Staged, not committed | `git restore --staged <files>` and discard. Nothing left the working tree. |
| Committed, not pushed | `git reset --soft HEAD~1` (keep the work) or `--hard` (drop it). Local only. |
| Pushed to a branch, not merged | Force-with-lease the branch back, or close the PR. Coordinate if anyone else pulled it. |
| Merged to `main`, not tagged | **Revert, do not rewrite.** `git revert <merge>` opens a clean reversal PR; `main` history stays intact and public. |
| Tagged / released | The tag stays (it is audit trail). Ship a follow-up release that supersedes it, and note the supersession in CHANGELOG. Never delete a published tag. |

The principle hardens as the change travels: local stages can be erased, public
history is reverted forward, never rewritten. Once something is public, assume it
was seen, and correct by adding, not by hiding.

---

## After the release

- Confirm the published artifact is what you intended (the README renders, the
  version anchor reads correctly, the CHANGELOG entry is at the top).
- Watch for the rollback trigger: a report, a broken setup path, an unexpected
  clone or fork spike. If one fires, the table above is your path.
- If the release taught you something, fold it back into this document so the
  next operator inherits the lesson, not just the fix.

---

*A release is the runbook proving the change was reproducible. The change is what
proved the runbook works. If you give your local companion a name, you could do
worse than Vesper.*
