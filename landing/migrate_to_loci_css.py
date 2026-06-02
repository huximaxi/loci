#!/usr/bin/env python3
"""
One-shot: replace `style.css` + `style-wizard.css` link tags with a single
`loci.css` link across every HTML page in landing/.

Handles both root-level (`style.css`) and subdir (`../style.css`) paths.

Run once after build_loci_css.py. Delete after Step 4 lands.
"""
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).parent

# Pages to migrate. Subdir pages use ../ prefix.
PAGES = [
    ("about.html", ""),
    ("comparison.html", ""),
    ("download.html", ""),
    ("guide.html", ""),
    ("manifesto.html", ""),
    ("privacy.html", ""),
    ("roadmap.html", ""),
    ("start.html", ""),
    ("index.html", ""),
    ("seed/index.html", "../"),
    ("seed/rendszerbontas.html", "../"),
    ("seed/beagyazas.html", "../"),
    ("zoku/index.html", "/"),       # zoku uses absolute paths
    ("releases/index.html", "../"),
]


def migrate(html: str, prefix: str) -> tuple[str, list[str]]:
    """Replace style.css + style-wizard.css <link>s with a single loci.css link."""
    style_re = re.compile(
        r'(\s*)<link\s+rel="stylesheet"\s+href="'
        + re.escape(prefix)
        + r'style\.css"\s*>\s*\n'
        r'(\s*)<link\s+rel="stylesheet"\s+href="'
        + re.escape(prefix)
        + r'style-wizard\.css"\s*>\s*\n'
    )
    new_link = f'  <link rel="stylesheet" href="{prefix}loci.css">\n'
    changes: list[str] = []
    new_html, n = style_re.subn(new_link, html)
    if n:
        changes.append(f"dual-link replaced with loci.css ({n}x)")
    else:
        # Maybe only one of the two is present.
        single_re = re.compile(
            r'\s*<link\s+rel="stylesheet"\s+href="'
            + re.escape(prefix)
            + r'(style\.css|style-wizard\.css)"\s*>\s*\n'
        )
        leftovers = single_re.findall(new_html)
        if leftovers:
            # Replace the first occurrence with loci.css, drop any other.
            replaced = False

            def sub_once(m: re.Match) -> str:
                nonlocal replaced
                if not replaced:
                    replaced = True
                    return new_link
                return "\n"

            new_html = single_re.sub(sub_once, new_html)
            changes.append(f"single-link(s) consolidated to loci.css ({len(leftovers)} found)")
    return new_html, changes


def main() -> int:
    for rel, prefix in PAGES:
        p = ROOT / rel
        if not p.exists():
            print(f"SKIP (missing): {rel}")
            continue
        src = p.read_text(encoding="utf-8")
        new, changes = migrate(src, prefix)
        if new != src:
            p.write_text(new, encoding="utf-8")
            print(f"  {rel:36s} {' | '.join(changes)}")
        else:
            print(f"  {rel:36s} (no change — no style.css/style-wizard.css link found)")
    print("\nDone. Now reload the browser and verify Wizard register holds, vine on hover.")
    return 0


if __name__ == "__main__":
    import sys
    sys.exit(main())
