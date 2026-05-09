#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════
# Loci · Roadmap Watcher
# Checks git worktrees, feature branch activity, and tracker
# staleness. Surfaces a daily briefing or updates roadmap.html.
#
# Usage:
#   ./roadmap-watcher.sh           # print checkin summary
#   ./roadmap-watcher.sh --html    # regenerate roadmap.html status badges
#   ./roadmap-watcher.sh --quiet   # no output, only update tracker
#
# Cron (daily 08:00):
#   0 8 * * * cd ~/Dev/loci/pipeline && ./watcher/roadmap-watcher.sh >> watcher/watcher.log 2>&1
# ═══════════════════════════════════════════════════════════

set -euo pipefail

REPO_DIR="$HOME/Dev/loci"
TRACKER="$HOME/Dev/loci/pipeline/ROADMAP-TRACKER.md"
ROADMAP_HTML="$HOME/Dev/loci/pipeline/roadmap/roadmap.html"
LOG_FILE="$HOME/Dev/loci/pipeline/watcher/watcher.log"
MODE="${1:-}"
STALE_DAYS=14
TODAY=$(date +%Y-%m-%d)

# ── Colours ─────────────────────────────────────────────────
RED='\033[0;31m'; YLW='\033[0;33m'; GRN='\033[0;32m'
BLU='\033[0;34m'; DIM='\033[2m'; NC='\033[0m'

# ── Header ──────────────────────────────────────────────────
if [[ "$MODE" != "--quiet" ]]; then
  echo ""
  echo "╔══════════════════════════════════════════════╗"
  echo "║  🌿 Loci · Roadmap Watcher · $TODAY  ║"
  echo "╚══════════════════════════════════════════════╝"
  echo ""
fi

# ── 1. Git: recent activity on feature branches ─────────────
FEATURE_IDS=("1A" "1B" "1C" "1D" "2A" "2B" "2C" "2D" "2E" "2F" "2G" "2H")
FEATURE_BRANCHES=(
  "feat/1A-ollama"
  "feat/1B-goose-mcp"
  "feat/1C-nostr-identity"
  "comms/1D-nym-announcement"
  "feat/2A-continue"
  "feat/2B-nostr-zaps"
  "feat/2C-at-protocol"
  "feat/2D-nym-sync"
  "feat/2E-ipfs"
  "feat/2F-anythingllm"
  "feat/2G-kagi"
  "feat/2H-tailscale"
)

ACTIVE_FEATURES=()
STALE_FEATURES=()
UNSTARTED_FEATURES=()

if [[ "$MODE" != "--quiet" ]]; then
  echo "── Feature Branch Activity ─────────────────────────────"
fi

cd "$REPO_DIR" 2>/dev/null || { echo "⚠  Loci repo not found at $REPO_DIR"; exit 1; }

for i in "${!FEATURE_IDS[@]}"; do
  ID="${FEATURE_IDS[$i]}"
  BRANCH="${FEATURE_BRANCHES[$i]}"

  # Check if branch exists (local or remote)
  if git show-ref --verify --quiet "refs/heads/$BRANCH" 2>/dev/null || \
     git show-ref --verify --quiet "refs/remotes/origin/$BRANCH" 2>/dev/null; then

    LAST_COMMIT=$(git log "$BRANCH" --format="%ar" -1 2>/dev/null || echo "unknown")
    LAST_MSG=$(git log "$BRANCH" --format="%s" -1 2>/dev/null | cut -c1-60)
    LAST_DATE=$(git log "$BRANCH" --format="%cd" --date=format:"%Y-%m-%d" -1 2>/dev/null || echo "")

    # Check staleness
    if [[ -n "$LAST_DATE" ]]; then
      DAYS_SINCE=$(( ( $(date +%s) - $(date -d "$LAST_DATE" +%s 2>/dev/null || date -j -f "%Y-%m-%d" "$LAST_DATE" +%s 2>/dev/null || echo 0) ) / 86400 ))
      if (( DAYS_SINCE > STALE_DAYS )); then
        STALE_FEATURES+=("$ID")
        [[ "$MODE" != "--quiet" ]] && printf "  ${YLW}⏸ %-5s${NC} %-40s ${DIM}%s (stale: %d days)${NC}\n" "$ID" "$LAST_MSG" "$LAST_COMMIT" "$DAYS_SINCE"
      else
        ACTIVE_FEATURES+=("$ID")
        [[ "$MODE" != "--quiet" ]] && printf "  ${GRN}● %-5s${NC} %-40s ${DIM}%s${NC}\n" "$ID" "$LAST_MSG" "$LAST_COMMIT"
      fi
    fi

    # Update tracker Last Activity column
    sed -i.bak "s/| $ID | .* | $LAST_DATE |/| $ID | $(echo "$LAST_MSG" | cut -c1-30) | $LAST_DATE |/" "$TRACKER" 2>/dev/null || true

  else
    UNSTARTED_FEATURES+=("$ID")
    [[ "$MODE" != "--quiet" ]] && printf "  ${DIM}○ %-5s  branch not yet created${NC}\n" "$ID"
  fi
done

# ── 2. Tracker staleness check ───────────────────────────────
if [[ "$MODE" != "--quiet" ]]; then
  echo ""
  echo "── Tracker Summary ────────────────────────────────────"
  echo "  Active:    ${#ACTIVE_FEATURES[@]} features"
  echo "  Stale:     ${#STALE_FEATURES[@]} features (>${STALE_DAYS}d no activity)"
  echo "  Unstarted: ${#UNSTARTED_FEATURES[@]} features"
fi

# ── 3. Check for open Cipher security gates ─────────────────
GATES=$(grep -c "🔓" "$TRACKER" 2>/dev/null || echo "0")
OPEN_GATES=$(grep "| THREAT-" "$TRACKER" 2>/dev/null | grep -v "cleared" | wc -l | tr -d ' ')

if [[ "$MODE" != "--quiet" ]] && (( OPEN_GATES > 0 )); then
  echo ""
  echo "── Cipher Security Gates ──────────────────────────────"
  grep "| THREAT-" "$TRACKER" 2>/dev/null | grep -v "cleared" | while read -r line; do
    echo "  ${RED}🔒${NC} $line"
  done
fi

# ── 4. Append to changelog in tracker ───────────────────────
SUMMARY="$TODAY  Watcher: ${#ACTIVE_FEATURES[@]} active, ${#STALE_FEATURES[@]} stale, ${#UNSTARTED_FEATURES[@]} not started"
# Only append if something changed
if ! grep -q "$TODAY" "$TRACKER" 2>/dev/null; then
  # Use a temp approach to insert into the changelog section
  python3 -c "
import re, sys
content = open('$TRACKER').read()
marker = '\`\`\`'
# Insert new changelog entry after the opening backtick block marker
parts = content.split(marker, 2)
if len(parts) >= 3:
    new_entry = '$SUMMARY'
    parts[1] = parts[1].rstrip() + '\n' + new_entry + '\n'
    open('$TRACKER', 'w').write(marker.join(parts))
" 2>/dev/null || true
fi

# ── 5. HTML mode: update status badges in roadmap.html ──────
if [[ "$MODE" == "--html" ]] && [[ -f "$ROADMAP_HTML" ]]; then
  echo ""
  echo "── Updating roadmap.html ───────────────────────────────"

  for i in "${!FEATURE_IDS[@]}"; do
    ID="${FEATURE_IDS[$i]}"
    BRANCH="${FEATURE_BRANCHES[$i]}"

    if git show-ref --verify --quiet "refs/heads/$BRANCH" 2>/dev/null; then
      # Branch exists = in-progress
      sed -i.bak "s/data-status=\"not-started\" data-id=\"$ID\"/data-status=\"in-progress\" data-id=\"$ID\"/" "$ROADMAP_HTML" 2>/dev/null || true
    fi
  done

  rm -f "${ROADMAP_HTML}.bak"
  echo "  ✓ roadmap.html status badges updated"
fi

# ── 6. Footer ────────────────────────────────────────────────
if [[ "$MODE" != "--quiet" ]]; then
  echo ""
  echo "── Next Actions ───────────────────────────────────────"

  if (( ${#UNSTARTED_FEATURES[@]} > 0 )); then
    NEXT="${UNSTARTED_FEATURES[0]}"
    echo "  Start:  Create branch feat/$NEXT-* and open pipeline/features/$NEXT-*/JUMP-IN.md"
  fi

  if (( ${#STALE_FEATURES[@]} > 0 )); then
    echo "  Review: ${STALE_FEATURES[*]} — stale, needs triage"
  fi

  echo ""
  echo "  Run './watcher/roadmap-watcher.sh --html' to sync roadmap page"
  echo "  Log: pipeline/watcher/watcher.log"
  echo ""
fi

# Write log entry
echo "$SUMMARY" >> "$LOG_FILE" 2>/dev/null || true
