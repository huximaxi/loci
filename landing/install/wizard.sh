#!/usr/bin/env bash
# wizard.sh — Loci MCP installer for Claude Desktop
#
# What this does:
#   1. Detects your OS (macOS + Linux supported; Windows prints manual steps)
#   2. Locates your Claude Desktop config
#   3. Backs it up with a timestamp
#   4. Merges a `loci` entry into `mcpServers` (idempotent)
#   5. Tells you what to do next
#
# Open beta — for testing. The `loci` MCP server is provided by the Loci
# Desktop app (https://github.com/huximaxi/loci/releases). The config this
# script writes is ready for when the app is running; nothing is required
# right now if you just want to inspect the change.
#
# Re-run safely: idempotent merge, never duplicates the `loci` entry.
# Dry run: set DRY_RUN=1 to print intended actions without writing.
#
# Source: https://github.com/huximaxi/loci

set -euo pipefail

# ── Colors (best-effort, no-op if not a TTY) ────────────────
if [ -t 1 ]; then
  C_RESET=$'\033[0m'; C_DIM=$'\033[2m'; C_BOLD=$'\033[1m'
  C_AMBER=$'\033[38;5;208m'; C_GREEN=$'\033[32m'; C_RED=$'\033[31m'
else
  C_RESET=""; C_DIM=""; C_BOLD=""; C_AMBER=""; C_GREEN=""; C_RED=""
fi

say()  { printf "%s\n" "$*"; }
ok()   { printf "%s✓%s %s\n" "$C_GREEN" "$C_RESET" "$*"; }
warn() { printf "%s⚠%s %s\n" "$C_AMBER" "$C_RESET" "$*"; }
die()  { printf "%s✗%s %s\n" "$C_RED" "$C_RESET" "$*"; exit 1; }

DRY_RUN="${DRY_RUN:-0}"

say ""
say "${C_BOLD}loci wizard installer${C_RESET} ${C_DIM}— open beta${C_RESET}"
say "${C_DIM}https://github.com/huximaxi/loci${C_RESET}"
say ""

# ── Detect OS + config path ─────────────────────────────────
OS="$(uname -s)"
case "$OS" in
  Darwin)
    CFG_DIR="$HOME/Library/Application Support/Claude"
    ;;
  Linux)
    # Claude Desktop on Linux: official builds use XDG_CONFIG_HOME
    CFG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/Claude"
    ;;
  MINGW*|MSYS*|CYGWIN*)
    warn "Windows detected. Open Claude Desktop manually and merge this snippet:"
    cat <<'JSON'
{
  "mcpServers": {
    "loci": {
      "command": "loci-mcp",
      "args": ["--config", "~/.loci/config.json"]
    }
  }
}
JSON
    say ""
    say "Config file lives at:  %APPDATA%\\Claude\\claude_desktop_config.json"
    exit 0
    ;;
  *)
    die "Unsupported OS: $OS (only macOS + Linux are wired up today)"
    ;;
esac

CFG_FILE="$CFG_DIR/claude_desktop_config.json"
ok "OS detected: $OS"
ok "Config target: $CFG_FILE"

# ── Python3 sanity ──────────────────────────────────────────
if ! command -v python3 >/dev/null 2>&1; then
  die "python3 is required for the config merge (no jq dependency by design)."
fi

# ── Backup ──────────────────────────────────────────────────
if [ "$DRY_RUN" = "1" ]; then
  warn "DRY_RUN=1 — printing intended actions, not writing anything"
fi

if [ -f "$CFG_FILE" ]; then
  TS="$(date +%s)"
  BACKUP="$CFG_FILE.bak.$TS"
  if [ "$DRY_RUN" != "1" ]; then
    cp "$CFG_FILE" "$BACKUP"
  fi
  ok "Backed up existing config → $BACKUP"
else
  if [ "$DRY_RUN" != "1" ]; then
    mkdir -p "$CFG_DIR"
    echo '{}' > "$CFG_FILE"
  fi
  ok "No existing config found — creating fresh one at $CFG_FILE"
fi

# ── Merge loci entry (Python stdlib, no jq) ─────────────────
LOCI_ENTRY='{"command":"loci-mcp","args":["--config","~/.loci/config.json"]}'

if [ "$DRY_RUN" = "1" ]; then
  say "${C_DIM}--- DRY_RUN preview: would merge into mcpServers.loci ---${C_RESET}"
  say "$LOCI_ENTRY"
  say "${C_DIM}--- /preview ---${C_RESET}"
else
  python3 - "$CFG_FILE" "$LOCI_ENTRY" <<'PY'
import json, sys, pathlib
cfg_path = pathlib.Path(sys.argv[1])
entry = json.loads(sys.argv[2])
try:
    data = json.loads(cfg_path.read_text() or '{}')
except json.JSONDecodeError:
    print("Existing config is not valid JSON. Backup preserved; aborting.", file=sys.stderr)
    sys.exit(1)
if not isinstance(data, dict):
    print("Existing config is not a JSON object. Backup preserved; aborting.", file=sys.stderr)
    sys.exit(1)
mcp = data.setdefault("mcpServers", {})
mcp["loci"] = entry  # idempotent: re-running overwrites cleanly
cfg_path.write_text(json.dumps(data, indent=2) + "\n")
print("merged")
PY
  ok "Merged loci entry into mcpServers"
fi

# ── Done ────────────────────────────────────────────────────
say ""
say "${C_BOLD}Next steps:${C_RESET}"
say "  1. Restart Claude Desktop."
say "  2. Open a new chat — loci tools appear in the MCP toolbox."
say "  3. ${C_DIM}If you don't see them, the Loci Desktop app needs to be running.${C_RESET}"
say "     ${C_DIM}Grab the beta build from: https://github.com/huximaxi/loci/releases${C_RESET}"
say ""
say "${C_DIM}Report issues at https://github.com/huximaxi/loci/issues${C_RESET}"
say "${C_DIM}Re-run anytime — this script is idempotent.${C_RESET}"
say ""
