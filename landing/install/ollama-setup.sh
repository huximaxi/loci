#!/usr/bin/env bash
# ollama-setup.sh — Loci local inference setup via Ollama
#
# What this does:
#   1. Checks Ollama is installed (https://ollama.ai)
#   2. Pulls nomic-embed-text  — local semantic indexing (Loci feature 1A)
#   3. Pulls a recommended chat model (qwen2.5:7b by default)
#   4. Prints the config snippet for ~/.loci/config.json
#
# Override model: CHAT_MODEL=llama3.2 bash ollama-setup.sh
# Skip chat model: SKIP_CHAT=1 bash ollama-setup.sh
#
# Source: https://github.com/huximaxi/loci
# License: Apache 2.0

set -euo pipefail

if [ -t 1 ]; then
  C_RESET=$'\033[0m'; C_BOLD=$'\033[1m'; C_DIM=$'\033[2m'
  C_AMBER=$'\033[38;5;208m'; C_GREEN=$'\033[32m'; C_RED=$'\033[31m'
else
  C_RESET=""; C_BOLD=""; C_DIM=""; C_AMBER=""; C_GREEN=""; C_RED=""
fi

say()  { printf "%s\n" "$*"; }
ok()   { printf "%s✓%s %s\n" "$C_GREEN" "$C_RESET" "$*"; }
warn() { printf "%s⚠%s %s\n" "$C_AMBER" "$C_RESET" "$*"; }
die()  { printf "%s✗%s %s\n" "$C_RED" "$C_RESET" "$*"; exit 1; }

EMBED_MODEL="nomic-embed-text"
CHAT_MODEL="${CHAT_MODEL:-qwen2.5:7b}"
SKIP_CHAT="${SKIP_CHAT:-0}"

say ""
say "${C_BOLD}loci ollama setup${C_RESET} ${C_DIM}— local inference${C_RESET}"
say "${C_DIM}https://loci.garden${C_RESET}"
say ""

# ── Check Ollama ─────────────────────────────────────────
if ! command -v ollama &>/dev/null; then
  die "Ollama not found. Install it first: https://ollama.ai/download"
fi
ok "Ollama found: $(ollama --version 2>/dev/null | head -1)"

# ── Pull embedding model ─────────────────────────────────
say ""
say "Pulling ${C_AMBER}${EMBED_MODEL}${C_RESET} (semantic indexing)..."
ollama pull "$EMBED_MODEL"
ok "$EMBED_MODEL ready"

# ── Pull chat model ──────────────────────────────────────
if [ "$SKIP_CHAT" = "0" ]; then
  say ""
  say "Pulling ${C_AMBER}${CHAT_MODEL}${C_RESET} (chat inference)..."
  ollama pull "$CHAT_MODEL"
  ok "$CHAT_MODEL ready"
fi

# ── Print config snippet ─────────────────────────────────
say ""
say "${C_BOLD}Add this to ~/.loci/config.json:${C_RESET}"
say ""
say "  ${C_DIM}{${C_RESET}"
say "    ${C_DIM}\"llm\": {${C_RESET}"
say "      ${C_DIM}\"provider\": \"local\",${C_RESET}"
say "      ${C_DIM}\"endpoint\": \"http://localhost:11434\",${C_RESET}"
say "      ${C_DIM}\"model\": \"${CHAT_MODEL}\"${C_RESET}"
say "    ${C_DIM}},${C_RESET}"
say "    ${C_DIM}\"index\": {${C_RESET}"
say "      ${C_DIM}\"embed_model\": \"${EMBED_MODEL}\",${C_RESET}"
say "      ${C_DIM}\"semantic\": true${C_RESET}"
say "    ${C_DIM}}${C_RESET}"
say "  ${C_DIM}}${C_RESET}"
say ""
ok "Ollama setup complete. Start loci: https://loci.garden/download.html"
