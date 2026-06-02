#!/usr/bin/env python3
"""
One-shot migration: wrap existing <nav>...</nav> and <footer>...</footer>
blocks in include sentinels, then run build.py to substitute partials.

Run once; delete after Step 3 lands.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).parent

# Page identity for active-link highlighting.
# Maps file path (relative to landing/) → data-page value.
PAGE_KEYS = {
    "about.html": "about",
    "comparison.html": "compare",
    "download.html": "download",
    "guide.html": "guide",
    "manifesto.html": "resources",
    "privacy.html": "resources",
    "roadmap.html": "roadmap",
    "start.html": "start",
    "seed/index.html": "resources",
    "seed/rendszerbontas.html": "resources",
    "seed/beagyazas.html": "resources",
    "zoku/index.html": "resources",
    "releases/index.html": "releases",
}

# Regexes. DOTALL so . matches newlines. Non-greedy so we get the first block.
NAV_RE = re.compile(r"<nav\b[^>]*>.*?</nav>", re.DOTALL)
FOOTER_RE = re.compile(r"<footer\b[^>]*>.*?</footer>", re.DOTALL)
BODY_OPEN_RE = re.compile(r"<body\b([^>]*)>")
HAS_INCLUDE_NAV = re.compile(r"<!--\s*#include\s+partials/_nav\.html\s*-->")
HAS_INCLUDE_FOOTER = re.compile(r"<!--\s*#include\s+partials/_footer\.html\s*-->")


def wrap_with_include(html: str, block_re: re.Pattern, partial: str) -> tuple[str, bool]:
    m = block_re.search(html)
    if not m:
        return html, False
    sentinel = (
        f"<!-- #include partials/{partial} -->\n"
        f"{m.group(0)}\n"
        f"<!-- /include -->"
    )
    return html[: m.start()] + sentinel + html[m.end() :], True


def add_data_page(html: str, key: str) -> tuple[str, bool]:
    m = BODY_OPEN_RE.search(html)
    if not m:
        return html, False
    attrs = m.group(1)
    if 'data-page=' in attrs:
        return html, False  # already has one
    new_attrs = attrs.rstrip() + f' data-page="{key}"'
    new_body = f"<body{new_attrs}>"
    return html[: m.start()] + new_body + html[m.end() :], True


def migrate(path: Path, key: str) -> str:
    src = path.read_text(encoding="utf-8")
    changes: list[str] = []

    if HAS_INCLUDE_NAV.search(src):
        changes.append("nav: already migrated")
    else:
        src, did = wrap_with_include(src, NAV_RE, "_nav.html")
        changes.append("nav: wrapped" if did else "nav: not found")

    if HAS_INCLUDE_FOOTER.search(src):
        changes.append("footer: already migrated")
    else:
        src, did = wrap_with_include(src, FOOTER_RE, "_footer.html")
        changes.append("footer: wrapped" if did else "footer: not found")

    src, did = add_data_page(src, key)
    changes.append(f"data-page={key}: {'added' if did else 'kept'}")

    path.write_text(src, encoding="utf-8")
    return " | ".join(changes)


def main() -> int:
    for rel, key in PAGE_KEYS.items():
        p = ROOT / rel
        if not p.exists():
            print(f"SKIP (missing): {rel}")
            continue
        report = migrate(p, key)
        print(f"  {rel:36s} {report}")
    print("\nMigration complete. Now run: python3 build.py")
    return 0


if __name__ == "__main__":
    sys.exit(main())
