#!/usr/bin/env python3
"""
check-two-tier.py — Loci two-tier memory wiring checker
Run from your palace root: python3 scripts/check-two-tier.py
Or point at a palace: python3 scripts/check-two-tier.py ~/my-palace
"""

import sys
import os
from pathlib import Path

PASS = "✅"
WARN = "⚠️ "
FAIL = "❌"

def check(label, result, detail=""):
    icon = PASS if result is True else (WARN if result == "warn" else FAIL)
    print(f"  {icon}  {label}")
    if detail:
        print(f"       {detail}")
    return result is True

def main():
    palace = Path(sys.argv[1]).expanduser() if len(sys.argv) > 1 else Path.cwd()
    home = Path.home()

    print(f"\n🏛  Loci two-tier wiring check")
    print(f"   Palace: {palace}\n")

    passed = 0
    total = 0

    # 1. CLAUDE.md exists
    total += 1
    claude_md = palace / "CLAUDE.md"
    if check("CLAUDE.md present", claude_md.exists(),
             f"Expected at: {claude_md}" if not claude_md.exists() else ""):
        passed += 1

    # 2. Identity block at top of CLAUDE.md
    total += 1
    if claude_md.exists():
        content = claude_md.read_text(errors="replace")
        first_500 = content[:500].upper()
        has_identity = "IDENTITY" in first_500 or "WHO I AM" in first_500 or "YOU ARE" in first_500
        if check("Identity block at top of CLAUDE.md", has_identity,
                 "Add '## IDENTITY — load this first' block (see TWO-TIER-SETUP.md)" if not has_identity else ""):
            passed += 1
    else:
        check("Identity block at top of CLAUDE.md", False, "CLAUDE.md missing — can't check")

    # 3. Global layer — Claude Code (~/.claude/CLAUDE.md)
    total += 1
    global_claude = home / ".claude" / "CLAUDE.md"
    if global_claude.exists():
        if check("Global layer present (~/.claude/CLAUDE.md)", True,
                 "Claude Code Tier 1 identity found"):
            passed += 1
    else:
        # Not a hard fail — Cowork users don't need this
        result = check("Global layer (~/.claude/CLAUDE.md)", "warn",
                       "Not found — fine for Cowork-only setups. Claude Code users: create this file.")
        # warn counts as partial pass
        passed += 0.5
        total -= 0.5  # adjust so warn doesn't penalise Cowork users

    # 4. Soul file exists
    total += 1
    soul_paths = [
        palace / "soul" / "SOUL.md",
        palace / "soul.md",
        palace / "SOUL.md",
    ]
    soul_found = next((p for p in soul_paths if p.exists()), None)
    if check("Soul file present", soul_found is not None,
             f"Expected at: {palace}/soul/SOUL.md" if not soul_found else str(soul_found)):
        passed += 1

    # 5. Rooms directory
    total += 1
    rooms_paths = [palace / "rooms", palace / "_rooms"]
    rooms_found = next((p for p in rooms_paths if p.is_dir()), None)
    if rooms_found:
        room_count = sum(1 for _ in rooms_found.rglob("CLAUDE.md"))
        if check("Rooms configured", room_count > 0,
                 f"{room_count} room(s) found in {rooms_found.name}/"):
            passed += 1
    else:
        check("Rooms configured", False,
              f"No rooms/ directory found. Create rooms/ with at least one room CLAUDE.md.")

    # 6. Session history (handovers)
    total += 1
    handover_paths = [
        palace / "soul" / "handovers",
        palace / "handovers",
    ]
    handovers_found = next((p for p in handover_paths if p.is_dir()), None)
    if handovers_found:
        deltas = list(handovers_found.glob("*.md"))
        if deltas:
            latest = max(deltas, key=lambda f: f.stat().st_mtime)
            if check("Session history present", True,
                     f"{len(deltas)} delta(s) — latest: {latest.name}"):
                passed += 1
        else:
            check("Session history present", "warn",
                  "handovers/ exists but is empty — write your first session delta to activate.")
            passed += 0.5
            total -= 0.5
    else:
        check("Session history present", "warn",
              "No handovers/ directory found — create soul/handovers/ after your first session.")
        passed += 0.5
        total -= 0.5

    # Summary
    score = int(passed)
    maxi = int(total)
    print()
    if score == maxi:
        print(f"  ✦  All checks passed ({score}/{maxi}). Two-tier wiring complete.\n")
        sys.exit(0)
    elif score >= maxi * 0.7:
        print(f"  ◈  {score}/{maxi} checks passed. Minor gaps — see warnings above.\n")
        sys.exit(1)
    else:
        print(f"  ◇  {score}/{maxi} checks passed. Two-tier wiring incomplete — see above.\n")
        print("     Reference: TWO-TIER-SETUP.md\n")
        sys.exit(1)

if __name__ == "__main__":
    main()
