# Deployment — loci.garden

## Infrastructure

- **VPS**: 1984 Hosting, Iceland
- **Web server**: Caddy (automatic HTTPS)
- **Domain**: loci.garden
- **Repo**: https://github.com/huximaxi/loci
- **Site root**: `landing/` subdirectory (NOT repo root)

## Directory Structure

```
/var/www/loci.garden/    # VPS site root
├── index.html           # Main landing page
├── start.html           # Getting started guide
├── about.html           # About page
├── llms.txt             # AI agent context declaration
├── llms-full.txt        # Full context declaration
├── robots.txt           # Search engine directives
├── sitemap.xml          # Site map for SEO
├── assets/              # Images, icons
├── seed/                # Dispatches
└── ...
```

## Deploy after PR merge

After merging a PR that touches `landing/**`:

```bash
# SSH into VPS
ssh loci-vps

# Navigate to site root
cd /var/www/loci.garden

# Pull latest (landing/ maps to site root)
git pull origin main

# If using sparse checkout:
# git sparse-checkout set landing
# git pull origin main

# Caddy auto-reloads on file changes, but to force:
sudo systemctl reload caddy
```

## GitHub Actions (future)

When CI is set up, the deploy rule will be:
- Changes to `landing/**` → trigger site deploy to VPS
- Changes to `extension/**` → trigger Chrome Web Store build

## Branch strategy

- `main` — production
- `feature/*` — feature branches, PR to main
- Never push directly to main

## Rollback

Keep `index-old.html` as backup. To rollback:

```bash
mv index.html index-broken.html
mv index-old.html index.html
```

## Contact

Questions: hux@nymtech.net
