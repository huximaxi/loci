# v0.6.0-rc.3 scope draft (methodology 1.6-candidate)

> Draft scope for the next-week public RC.
> Builds on rc.2's foundations (CLI, personas/, skills/). Each item below is sized
> to land in one focused session; the bundle is meant to be picked up cold.

---

## Themes for 1.6

1. **The CLI grows up**: beyond the read-shaped slice rc.2 ships.
2. **Skill tree shape**: turn `templates/skills/` from a flat shelf into a navigable tree.
3. **More persona templates**: broaden the dyad shelf beyond engine-room.
4. **"Loci helps you set up your Claude"**: the framing piece. Onboarding the first AI to the palace.

None of these are speculative; each one has a real predecessor (a piece in a
working palace, a request from a check-in, or a doctrine line in
PALACE-METHODOLOGY that wants a shipped piece).

---

## Items, each one shippable on its own

### A. CLI · the deferred half

The rc.2 CLI is read-only. rc.3 brings the cautious half of writing.

- **One-shot inference (`loci -z "<prompt>"`)**: call your configured Ollama
  backend with palace context. Stdlib + a small HTTP client (`ureq` over
  `reqwest` to keep deps tiny and avoid tokio). No sandbox required for
  localhost-only backends.
- **Backend choice in `loci init`**: Anthropic and OpenAI branches. API
  keys via env (`ANTHROPIC_API_KEY`, `OPENAI_API_KEY`) or per-user keyring
  (not flat-file config). Network-egress disclosure on `init` and on every
  `-z` invocation.
- **`loci sync`**: reindex / state.json writeback. The first write-side
  CLI command. Touches palace files; need a `--dry-run` default and an
  explicit `--apply` flag.
- **`loci feedback`**: full version of the diagnostic feedback flow. Gated
  on a public issue template on `huximaxi/loci` (file a starter template as
  part of this RC, then wire the CLI's `feedback` to open it pre-populated).

**Out of 1.6 (deferred further):** Chat TUI (`loci chat`), MCP server
(`loci serve`). Both are larger adversarial surfaces; want a Cipher
threat-model pass before they land.

---

### B. Skill tree shape

The `templates/skills/` shelf in rc.2 ships one skill (quarantine). The
1.6 work makes the shelf navigable:

- `templates/skills/INDEX.md`: a single-page index of every skill, grouped
  by owner posture (security, ops, design, growth) with one-line summaries.
- A second skill: **pre-public-push read** (Cipher's standing reflex turned
  procedure). The companion to `quarantine.md` on the outbound side.
- A third skill: **two-phase destructive op** (Praxis posture). The
  dry-run-then-execute discipline named in the rc.2 persona template,
  written as its own procedure.

Three skills makes a shelf; one is an example. The shape needs three to
prove it.

---

### C. More persona templates

Broaden beyond the engine-room dyad. Two more, each generic enough to
fork:

- **Kata** (data recon, bulk processing, exhaustive audits). The persona
  who descends into the trench. Pairs with no one; works alone, returns
  structured reports.
- **A growth voice** (the outward-facing copywriter; placeholder name
  pending, likely something other than "RUNE" to keep this template
  generic). Marketing, positioning, launch copy. Pairs with the designer.

Each follows the Cipher.md / Praxis.md pattern: working principles,
standing reflexes, what-I-don't-do, activation triggers.

---

### D. "Loci helps you set up your Claude"

The framing piece. Currently the README's setup story is "clone, copy
templates, point your AI at CLAUDE.md." That works once you know what a
palace is. For a fresh user it's still abstract.

The 1.6 piece is a small guided onboarding that runs at first-launch:

- **`loci init --guided`**: beyond the rc.2 init wizard. Walks a new user
  through palace creation (PALACE.md / CLAUDE.md scaffolding, one room
  picked, a first crystal seeded). The CLI doesn't talk to an AI; it
  prepares the ground for the user's AI to do its first useful read.
- **`AGENT-SETUP.md` refresh**: update the existing agent-setup walkthrough
  to land on `loci init --guided` instead of the markdown-only path. Both
  remain valid; the CLI-led path becomes the recommended one.

This is the door that says "the CLI is for users, not just for inspecting
your palace after the fact."

---

## What ships with rc.3

- 4 new CLI commands (`-z`, `sync`, `feedback`, `init --guided`)
- 2 new skills + a skills index
- 2 new persona templates
- 1 refreshed onboarding guide
- 1 new public issue template (for `loci feedback`)
- CHANGELOG entries + PALACE-METHODOLOGY 1.6-candidate notes

## What does NOT ship in rc.3

- `loci chat` (TUI): deferred to 1.7+, behind threat-model
- `loci serve` (MCP): deferred to 1.7+, behind threat-model
- New methodology doctrine (1.6 is implementation, not new ideas)

---

## Pre-flight before opening this PR

- Cipher bleed scan on the new templates (especially the third persona
  template, since "growth voice" sits closest to organisation-specific
  content)
- Cipher threat-model read on the new CLI write surfaces (`sync`,
  `init --guided`)
- Praxis pre-mortem on the API-key path in `init` (env vs keyring; the
  asymmetric risk lives here)
- Bleed-guard pre-push hard gate on the branch
- README sync + version anchor bump

---

## Picked up cold

If you are picking this up next week with no other context: the work
above is independent of any palace-specific work. Each item builds on what
rc.2 shipped (CLI binary, templates/personas/, templates/skills/). The
order doesn't matter; pick any item, ship it, repeat. The PR is the
container, not the schedule.
