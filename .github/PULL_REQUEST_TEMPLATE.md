## What

<!-- JUMP-IN brief ID + one-line description. e.g. "1A + 1B — Ollama local inference + MCP server backend (desktop v0.4.0)" -->

## Acceptance criteria

<!-- Copy from pipeline/features/{ID}/JUMP-IN.md -->

- [ ] 
- [ ] 

## Components changed

- [ ] `desktop` (Tauri app + Rust backend)
- [ ] `extension` (Chrome extension)
- [ ] `packages/core` (shared types)
- [ ] `landing` (site HTML / llms.txt)
- [ ] `palace` (templates / PROCESSES.md)
- [ ] `pipeline` (ROADMAP-TRACKER, JUMP-IN briefs)

## Release checklist

- [ ] `CHANGELOG.md` updated with version entry at top
- [ ] Version bumped in `tauri.conf.json` (desktop) or `package.json` (extension) as appropriate
- [ ] `pipeline/ROADMAP-TRACKER.md` status updated (🟡 → 🟢 on merge)
- [ ] `landing/llms.txt` updated if integration is public-facing
- [ ] Root `package.json` NOT bumped (monorepo scaffold — version is frozen)

## Security (Cipher gate)

- [ ] URL validation / path traversal prevention in place
- [ ] No 0.0.0.0 binding (localhost or Tailscale only)
- [ ] THREAT-01 gate respected — no raw `Conversation` objects exposed via MCP
- [ ] Dependency additions reviewed (no new transitive deps without rationale)
- [ ] Cipher gate cleared ✓ or not applicable

## HuxGATE

- [ ] Hux reviewed and approved ✓

## Notes for reviewer

<!-- Tauri permissions, Cipher gate decisions, known limitations, UI that's still pending -->
