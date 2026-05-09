# Loci · GitHub Organisation Strategy
*Written: 2026-05-09 · Vesper × Kata*

---

## Principle

Loci is a monorepo with independently versioned components. GitHub should reflect that reality without ceremony. The strategy is: **one main, feature branches per integration, component release tags, milestone-grouped work, PR template with built-in acceptance criteria.**

No tooling overhead (no changesets, no release-please, no lerna). Just Git conventions + three GitHub features (Labels, Milestones, Releases).

---

## Branch Naming

| Prefix | Use | Example |
|--------|-----|---------|
| `feat/{ID}-{slug}` | Sovereignty stack integrations (from JUMP-IN briefs) | `feat/1A-ollama`, `feat/1B-goose-mcp` |
| `fix/{slug}` | Bug fixes | `fix/search-index-corruption` |
| `palace/{slug}` | Palace methodology changes (templates, processes) | `palace/garden-health-v2` |
| `site/{slug}` | Landing-only changes | `site/roadmap-status-update` |
| `comms/{slug}` | Communications / announcements | `comms/1D-nym-announcement` |
| `docs/{slug}` | Docs site only | `docs/ollama-setup-guide` |

Rules:
- `main` is always releasable — protected branch, no direct pushes
- Feature branches are short-lived: open → review → merge → delete
- One branch per JUMP-IN feature ID (don't mix 1A and 1B)

---

## Release Tags — Per Component

Each component versions independently. Tags use `{component}/v{semver}`.

| Component | Tag pattern | Example | Semver trigger |
|-----------|------------|---------|----------------|
| Desktop app | `desktop/vX.Y.Z` | `desktop/v0.3.0` | Minor: new feature. Patch: fix. |
| Chrome extension | `extension/vX.Y.Z` | `extension/v1.3.0` | Minor: new feature. Patch: fix. |
| Palace templates | `palace/vX.Y.Z` | `palace/v1.1.0` | Minor: new process/template. |
| Core types pkg | `core/vX.Y.Z` | `core/v1.1.0` | Minor: new types (breaking = major). |
| Landing site | `site/YYYY-MM-DD` | `site/2026-05-09` | Date-keyed, not semver. |

**Root `package.json`** stays at `0.1.0` indefinitely — it's the monorepo scaffold, not a product version. Never bump it for features.

### Release workflow (per feature):

```bash
# 1. Create branch (already done when work starts)
git checkout -b feat/1A-ollama

# 2. Develop. Commit with component prefix:
git commit -m "feat(desktop): add Ollama Tauri commands (1A)"
git commit -m "feat(core): add OllamaConfig type (1A)"

# 3. Update CHANGELOG.md + version in component config
# tauri.conf.json: 0.2.0 → 0.3.0
# CHANGELOG.md: new entry at top

# 4. PR to main
# → Cipher review (security gate)
# → HuxGATE (Hux approval before merge)
# → merge

# 5. Tag the release on main
git tag desktop/v0.3.0
git push origin desktop/v0.3.0

# 6. GitHub Release
# Title: "loci desktop v0.3.0 — Ollama Local Inference"
# Body: copy CHANGELOG entry for this version
# Attach: .dmg and .exe from CI build (when CI exists)

# 7. Update ROADMAP-TRACKER.md: 🟡 in-progress → 🟢 shipped
```

> **Deploy is not here.** VPS deployment (palace-vps, Caddy, `git pull`) is Engine Room ops. See `_palace/engine-room/CLAUDE.md` or the Engine Room deploy runbook. GitHub conventions end at the tag + Release. What happens after merge on the infrastructure side lives in the Engine Room.

---

## GitHub Labels

Three label groups. Create these once; apply consistently.

**Component** (blue):
- `desktop`
- `extension`
- `palace`
- `core`
- `site`
- `docs`

**Tier / type** (purple):
- `tier-1` — Q3 2026 integrations
- `tier-2` — Q4 2026 integrations
- `feature`
- `fix`
- `security` — Cipher gate required
- `comms` — announcement / copy

**Status** (yellow/red):
- `cipher-gate` — security review needed before merge
- `huxgate-required` — Hux must review before merge
- `needs-ui` — backend done, UI pending

---

## GitHub Milestones

- **Tier 1 — Q3 2026** (1A, 1B, 1C, 1D)
- **Tier 2 — Q4 2026** (2A–2H)
- **Maintenance** (fixes, deps, security patches)

Assign every feature branch PR to the relevant milestone.

---

## PR Template

Create `.github/PULL_REQUEST_TEMPLATE.md`:

```markdown
## What

[JUMP-IN brief ID + one-line description, e.g. "1A — Ollama local inference backend"]

## Acceptance Criteria

From `pipeline/features/{ID}/JUMP-IN.md`:

- [ ] (copy criteria here)

## Components changed

- [ ] desktop
- [ ] extension
- [ ] palace/templates
- [ ] core/types
- [ ] landing/site

## Release checklist

- [ ] CHANGELOG.md updated with version entry
- [ ] Version bumped in component config (tauri.conf.json / package.json)
- [ ] ROADMAP-TRACKER.md status updated
- [ ] llms.txt updated if integration is public-facing
- [ ] Cipher gate cleared (security review)
- [ ] HuxGATE passed

## Notes

[Anything the reviewer needs to know. Tauri permissions, Cipher gate decisions, known limitations.]
```

---

## Commit Message Convention

```
{type}({component}): {short description} [{ID}]

Types: feat | fix | security | docs | palace | chore
Components: desktop | extension | core | site | palace

Examples:
feat(desktop): add Ollama Tauri commands (1A)
feat(core): add OllamaConfig type (1A)
fix(extension): repair search index deserialization
security(desktop): validate Ollama base_url against SSRF (1A)
palace(templates): add garden-health v2 process
```

---

## What We Don't Do

- No monorepo versioning (no `changesets`, no `lerna`, no `release-please`)
- No automatic changelog generation (CHANGELOG.md is hand-authored, voice matters)
- No version bumps to root `package.json`
- No `dependabot` auto-merge (security changes go through Cipher gate)
- No force-push to main
- No deploy instructions in this doc — deploy is Engine Room, not GitHub conventions

---

## Current State (2026-05-09)

| Component | Version | Last release |
|-----------|---------|-------------|
| desktop | v0.4.0 | 2026-05-09 (1B MCP backend) |
| extension | v1.2.0 | 2026-05-05 |
| palace | v1.0.0 | 2026-05-08 |
| site | 2026-05-09 | sovereignty stack launch |

Active branches:
- `feat/1A-ollama` — open PR → main when UI complete (settings panel + offline badge, Hux)
- `feat/1B-goose-mcp` — open PR → main when UI complete (MCP toggle + status, Hux) + Goose connection test

---

*pipeline/GITHUB-STRATEGY.md · Loci monorepo conventions*
