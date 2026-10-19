## What

<!-- JUMP-IN brief ID + one-line description. e.g. "F3: scheduled-task locator finds PALACE.md" -->

## Acceptance criteria

<!-- Copy from pipeline/features/{ID}/JUMP-IN.md -->

- [ ] 
- [ ] 

## Components changed

- [ ] `loci-cli` (terminal CLI)
- [ ] `extension` (Chrome extension)
- [ ] `packages/core` (shared types)
- [ ] `landing` (site HTML / llms.txt)
- [ ] `palace` (templates / PROCESSES.md)
- [ ] `pipeline` (ROADMAP-TRACKER, JUMP-IN briefs)

## Release checklist

- [ ] `CHANGELOG.md` updated with version entry at top
- [ ] Version bumped in `loci-cli/Cargo.toml` (CLI) or `package.json` (extension) as appropriate
- [ ] `pipeline/ROADMAP-TRACKER.md` status updated (🟡 → 🟢 on merge)
- [ ] `landing/llms.txt` updated if integration is public-facing
- [ ] Root `package.json` NOT bumped (monorepo scaffold — version is frozen)

## Security (privacy gate)

- [ ] URL validation / path traversal prevention in place
- [ ] No 0.0.0.0 binding (localhost or Tailscale only)
- [ ] Dependency additions reviewed (no new transitive deps without rationale)
- [ ] Security review cleared ✓ or not applicable

## Maintainer gate

- [ ] Maintainer reviewed and approved ✓

## Notes for reviewer

<!-- Privacy gate decisions, known limitations, anything still pending -->
