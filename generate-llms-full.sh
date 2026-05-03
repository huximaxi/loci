#!/usr/bin/env bash
# Generates llms-full.txt — full context declaration for AI agents.
# Output is committed to the repo root and served at loci.garden/llms-full.txt.
# Run from the loci repo root.

set -e
cd "$(dirname "$0")"

OUT="llms-full.txt"

{
  echo "# loci — full context declaration"
  echo "# Generated $(date -u +%Y-%m-%d) from loci repo"
  echo "# Concise version: https://loci.garden/llms.txt"
  echo "# Repo: https://github.com/huximaxi/loci"
  echo ""
  echo "---"
  echo ""

  for file in \
    README.md \
    FIRST-SESSION.md \
    SETUP-GUIDE.md \
    AGENT-SETUP.md \
    PROCESSES.md \
    templates/retrieval-hierarchy.md \
    templates/CLAUDE-master.md \
    templates/SOUL.md \
    templates/room-template.md \
    templates/garden-template.md \
    templates/persona-template.md \
    templates/handover-template.md \
    templates/scheduled-task-template.md \
    examples/example-CLAUDE.md
  do
    if [ -f "$file" ]; then
      echo ""
      echo "---"
      echo "## FILE: $file"
      echo "---"
      echo ""
      cat "$file"
      echo ""
    else
      echo "# [MISSING: $file]" >&2
    fi
  done
} > "$OUT"

echo "Written: $OUT ($(wc -l < "$OUT") lines, $(wc -c < "$OUT" | tr -d ' ') bytes)"
