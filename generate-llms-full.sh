#!/usr/bin/env bash
# Generates llms-full.txt - full context declaration for AI agents.
# Output goes to landing/llms-full.txt (served at loci.garden/llms-full.txt).
# Run from the loci repo root.

set -e
cd "$(dirname "$0")"

OUT="landing/llms-full.txt"

{
  echo "# loci.garden - full context declaration"
  echo "# Proactive context management for human-AI collaboration"
  echo "# Built by Hux x Vesper · 2026 · Apache 2.0"
  echo ""
  echo "> This is the full context declaration. For the concise version, see: https://loci.garden/llms.txt"
  echo ""
  echo "> **What is Loci?** A methodology for persistent AI collaboration. Your AI gets memory (crystals), character (soul files), and ideas that grow (the garden). No database. No vendor lock-in. Just markdown files."
  echo ""
  echo "**Website**: https://loci.garden"
  echo "**GitHub**: https://github.com/huximaxi/Loci"
  echo "**Palace methodology version**: loci-core v1.0 (2026-05-08)"
  echo ""
  echo "---"
  echo ""
  echo "## Full repository contents below"
  echo ""
  echo "---"
  echo ""

  for file in \
    README.md \
    LOCI-CORE.md \
    CHANGELOG.md \
    FIRST-SESSION.md \
    PALACE-UPDATE.md \
    SETUP-GUIDE.md \
    AGENT-SETUP.md \
    PROCESSES.md \
    templates/retrieval-hierarchy.md \
    templates/CLAUDE-master.md \
    templates/SOUL.md \
    templates/room-template.md \
    templates/crystals-guide.md \
    templates/garden-template.md \
    templates/garden-health-template.md \
    templates/synthesis-automation.md \
    templates/peer-card-template.md \
    templates/persona-template.md \
    templates/handover-template.md \
    templates/scheduled-task-template.md \
    templates/output-primitive.md \
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
