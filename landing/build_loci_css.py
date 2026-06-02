#!/usr/bin/env python3
"""
Step 4 consolidation: produce a single loci.css from the legacy sources.

Strategy (KISS):
- style.css = canonical base (Wizard tokens at :root)
- style-wizard.css = appended verbatim (per-room palette tokens live under
  html[data-skin="wizard"] — dormant until that attribute is set)
- Version B "Garden Immersive" aliases injected into :root
- LLMAGE mono register appended as [data-theme="llmage"] overlay
- a:hover globally shifted to vine green (Version B visible signal)

Run once. Re-run idempotent — overwrites loci.css from latest sources.
"""
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).parent
STYLE = ROOT / "style.css"
WIZARD = ROOT / "style-wizard.css"
OUT = ROOT / "loci.css"

# Version B "Garden Immersive" extensions to inject at top of :root.
VERSION_B_BLOCK = """
  /* ── Version B "Garden Immersive" extensions (Nyx 2026-04-28) ─── */
  --accent-vine:        #1a3d1f;   /* hover, active, garden anchors */
  --accent-vine-light:  #2d6b35;
  --accent-vine-hi:     #4a9a4f;
  --accent-vine-soft:   rgba(26, 61, 31, 0.18);
  --accent-indigo:      #0d1b3e;   /* section bg, comparison table */
  --accent-indigo-soft: rgba(13, 27, 62, 0.55);
"""

# LLMAGE alternate register — mono-dark, Helvetica Neue mono.
# Preserved for /personas easter-egg toggle (Step 8 will wire it).
LLMAGE_BLOCK = """

/* ═══════════════════════════════════════════════════════════════════════════
   LLMAGE ALTERNATE REGISTER — monochrome, austere
   Activated on /personas (Step 8); preserved here so toggle works site-wide.
   ═══════════════════════════════════════════════════════════════════════════ */

[data-theme="llmage"] {
  --bg:           #080808;
  --bg-surface:   #0f0f0f;
  --bg-card:      #141414;
  --border:       #2a2a2a;
  --border-soft:  #1a1a1a;
  --accent:       #ebebeb;
  --accent-light: #ffffff;
  --accent-dim:   rgba(235, 235, 235, 0.10);
  --accent-glow:  rgba(235, 235, 235, 0.05);
  --accent-vine:        #2a2a2a;
  --accent-vine-hi:     #4a4a4a;
  --accent-vine-soft:   rgba(40, 40, 40, 0.18);
  --accent-indigo:      #1a1a1a;
  --accent-indigo-soft: rgba(20, 20, 20, 0.55);
  --text:         #ebebeb;
  --text-dim:     #888888;
  --text-faint:   #444444;
  --text-mono:    #aaaaaa;
  --serif:        'Helvetica Neue', 'Inter', sans-serif;
  --mono:         'Helvetica Neue', monospace;
  --sans:         'Helvetica Neue', sans-serif;
}

[data-theme="llmage"] body {
  font-family: var(--mono);
  letter-spacing: 0.02em;
}

[data-theme="llmage"] .nav-link,
[data-theme="llmage"] .nav-logo,
[data-theme="llmage"] h1,
[data-theme="llmage"] h2,
[data-theme="llmage"] h3 {
  font-family: var(--mono);
  text-transform: uppercase;
  letter-spacing: 0.08em;
}
"""


def inject_version_b_into_root(css: str) -> str:
    """Insert Version B token block at top of :root { ... }."""
    pattern = re.compile(r'(:root\s*\{)', re.MULTILINE)
    return pattern.sub(r'\1' + VERSION_B_BLOCK, css, count=1)


def shift_a_hover_to_vine(css: str) -> str:
    """Make a:hover use vine green (the Version B visible signal)."""
    pattern = re.compile(
        r'(a:hover\s*\{[^}]*?color:\s*)(var\(--accent-light\)|#[0-9a-fA-F]+)',
        re.DOTALL,
    )
    return pattern.sub(r'\1var(--accent-vine-hi)', css, count=1)


def main() -> int:
    style_css = STYLE.read_text(encoding="utf-8")
    wizard_css = WIZARD.read_text(encoding="utf-8")

    style_css = inject_version_b_into_root(style_css)
    style_css = shift_a_hover_to_vine(style_css)

    header = """/* ═══════════════════════════════════════════════════════════════════════════
   LOCI.CSS — Single canonical stylesheet for loci.garden
   Generated 2026-05-19 from style.css + style-wizard.css + Version B + LLMAGE

   Layout:
   1. :root canonical Wizard tokens (burnt amber × charcoal)
      + Version B "Garden Immersive" aliases (vine + indigo)
   2. Base, reset, typography, layout, components (from style.css)
   3. Wizard-skin per-room palette tokens (html[data-skin="wizard"], dormant
      until that attribute is set on <html>; reserved for opt-in atmospheric
      surfaces)
   4. LLMAGE alternate register ([data-theme="llmage"], easter-egg overlay)
   ═══════════════════════════════════════════════════════════════════════════ */

"""

    combined = (
        header
        + style_css
        + "\n\n/* ═══════════════════════════════════════════════════════════════════════════\n"
        + "   STYLE-WIZARD.CSS — Per-room atmospheric palette (Nyx, 2026-04-28)\n"
        + "   Dormant under html[data-skin=\"wizard\"]; activate per-page opt-in.\n"
        + "   ═══════════════════════════════════════════════════════════════════════════ */\n\n"
        + wizard_css
        + LLMAGE_BLOCK
    )

    OUT.write_text(combined, encoding="utf-8")
    lines = combined.count("\n")
    bytes_out = len(combined)
    print(f"loci.css written: {lines} lines, {bytes_out:,} bytes")
    print(f"  ├── style.css base:        {style_css.count(chr(10))} lines (+ Version B + vine hover)")
    print(f"  ├── style-wizard.css:      {wizard_css.count(chr(10))} lines (dormant)")
    print(f"  └── LLMAGE overlay:        {LLMAGE_BLOCK.count(chr(10))} lines")
    return 0


if __name__ == "__main__":
    import sys
    sys.exit(main())
