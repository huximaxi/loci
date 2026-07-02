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

### E. Desktop cockpit: maps rail + tools gate-ledger (shipped in this PR)

The headline candidate named in the starter-kit delta, landed first. The desktop
dashboard becomes a cockpit: one Operations view (the existing native
dashboard) plus one tab per **palace-map instrument** the app discovers in
the user's palace.

- **Discovery, not configuration**: any self-contained `*.html` at the
  palace surface (root, `_palace/`, `cockpit/`) carrying an embedded
  `<script id="payload"|"snapshot" type="application/json">` payload is an
  instrument. `dashboard.html` is excluded (a palace's own generated cockpit
  page; embedding it would nest cockpits). Tab badges are peeked from the
  payload (largest array count), best-effort, never an error.
- **Embedding via sandboxed `srcdoc`**: the instrument HTML is read through
  a path-validated command (relative, `.html`, canonical-under-root;
  traversal and symlink escapes rejected, unit-tested) and rendered in an
  iframe with `sandbox="allow-scripts"`: opaque origin, so instrument
  scripts run but the app's storage, `window.parent`, and the Tauri IPC
  globals are unreachable. Palace content can be written by crons and
  agents; the frame is a wall, not a door.
- **Tools gate-ledger**: if the palace's `palace-map.json` (or `map.json`)
  carries `tools.items`, the dashboard renders the shelf: each external tool
  with its quarantine verdict (`admitted` / `admitted-escorted` / `deferred` /
  `held-conditional` / `rejected`) as a state-colored card. The shelf lists;
  it never loads. Palaces without a ledger see no change (fail-soft).
- **Brand coherence, single stylesheet for now**: instruments are authored on
  the shared dark palette; the app appends one override stylesheet to each
  embedded document that remaps the conventional `:root` variables to the
  app's own palette, plus a short list of structural overrides for the known
  hardcoded-dark surfaces. Instruments that don't use the convention are
  unaffected. A proper theming contract (instruments reading a
  `prefers-...` signal or a palette payload) is a later RC.
- **One list, one owner**: when instrument tabs exist, the native view keeps
  only the KPI header and the automation instrument owns the job table; a
  palace with no instruments keeps the native table as fallback.
- **Motivation, SWE agents**: this shelf exists because vetting a
  third-party coding-agent harness (a SWE-agent-style tool that reads a
  repo and proposes or runs patches) before trusting it near real
  credentials or a real filesystem is exactly the kind of decision that
  otherwise gets buried in a session log and re-litigated the next time
  someone reaches for the same tool. A durable, visible verdict, once
  made, should never need re-deriving.

## What ships with rc.3

- Desktop cockpit: maps rail + tools gate-ledger (item E, in this PR)
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
