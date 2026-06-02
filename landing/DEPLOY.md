# Deploying loci.garden

## Infrastructure

- **VPS**: 1984 Hosting, Iceland
- **Web server**: Caddy (automatic HTTPS, multi-tenant via `import sites/*.caddy`)
- **Domains**: loci.garden, docs.loci.garden
- **Repo**: https://github.com/rainbow-zoku/loci.garden
- **Served root**: the repo root (this directory). Caddy's `file_server` serves it straight off disk, so a content deploy is just updating that checkout. No build step.

## Deploy after PR merge

After merging a PR, update the VPS checkout to the new `main`:

```bash
# On the VPS, inside the site checkout:
git fetch origin
git reset --hard origin/main   # not `git pull` (see note below)
```

Caddy serves the files live, so no restart or reload is required for a content change. Operator specifics (host alias, checkout path, credentials) live in the private ops notes, not in this public repo.

### Why `reset --hard`, not `git pull`

The repo history was rewritten once (a `git filter-repo` pass to scrub a leaked file from history). A long-lived VPS checkout created before that rewrite has a forked history and will not fast-forward, so `git pull` would attempt a divergent merge. `fetch` + `reset --hard origin/main` realigns the checkout cleanly. Copy the served dir to a timestamped backup first if you want an instant rollback.

## Branch strategy

- `main`: production
- `feature/*`: feature branches, PR into `main`
- Never push directly to `main` (branch protection enforces this)

## Rollback

Before deploying, copy the served `landing/` directory to a timestamped backup. To roll back, restore that copy in place. No Caddy change needed.

## Contact

Questions: hux@nymtech.net
