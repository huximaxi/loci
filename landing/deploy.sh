#!/bin/bash
# loci.garden — deploy script
# Run from loci-garden/ directory.
# Usage: ./deploy.sh "optional commit message"

set -e

MSG="${1:-update — $(date '+%Y-%m-%d')}"

echo "→ Staging changes..."
git add index.html index-old.html start.html about.html download.html comparison.html style.css style-wizard.css skin-toggle.js fold.js llms.txt llms-full.txt robots.txt deploy.sh
git add assets/ seed/

echo "→ Committing: $MSG"
git commit -m "$MSG" || echo "(nothing new to commit)"

echo "→ Syncing to VPS..."
rsync -avz \
  --exclude='.git' --exclude='.DS_Store' \
  --exclude='dispatches' --exclude='Design System*' \
  -e "ssh -p 2222 -i ~/.ssh/id_ed25519_huximaxi" \
  ./ hux@195.246.230.118:/home/hux/loci-garden/

echo "→ Fixing permissions..."
ssh -p 2222 -i ~/.ssh/id_ed25519_huximaxi hux@195.246.230.118 \
  'find /home/hux/loci-garden -type f -print0 | xargs -0 chmod 644 && find /home/hux/loci-garden -type d -print0 | xargs -0 chmod 755'

echo "✓ loci.garden deployed: $MSG"
