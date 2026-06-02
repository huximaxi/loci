#!/usr/bin/env python3
"""
loci.garden static site include processor.

Walks .html files under landing/, replaces sentinel comment blocks with
the corresponding partial from landing/partials/_*.html, and writes the
result back in place. Source files are committed with substitutions
already applied, so the deploy path stays a plain git pull.

Sentinel format:
    <!-- #include partials/_nav.html -->
    ...existing content (anything here is replaced on next build)...
    <!-- /include -->

Idempotent: re-running after a partial edit re-substitutes cleanly.

Usage:
    python3 build.py            # process every page
    python3 build.py --check    # exit 1 if anything would change (CI guard)
    python3 build.py path/...   # process specific files

Adds .nav-link.active to the matching nav anchor based on a page's
<body data-page="X"> attribute, so each page highlights its own entry.
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).parent
PARTIALS = ROOT / "partials"

INCLUDE_RE = re.compile(
    r"<!--\s*#include\s+(?P<path>partials/[A-Za-z0-9_./-]+\.html)\s*-->"
    r".*?"
    r"<!--\s*/include\s*-->",
    re.DOTALL,
)

BODY_DATA_PAGE_RE = re.compile(r'<body[^>]*\bdata-page="(?P<page>[^"]+)"')


def load_partial(rel_path: str) -> str:
    """Read a partial file. Cached per-run."""
    p = ROOT / rel_path
    if not p.exists():
        raise FileNotFoundError(f"partial not found: {p}")
    return p.read_text(encoding="utf-8").rstrip("\n")


def apply_active(nav_html: str, page_key: str | None) -> str:
    """Add .active class to the nav anchor whose data-page matches."""
    if not page_key:
        return nav_html
    pattern = re.compile(
        r'(<a\b[^>]*\bdata-page="' + re.escape(page_key) + r'"[^>]*\bclass=")([^"]*)(")'
    )
    return pattern.sub(
        lambda m: m.group(1) + (m.group(2) + " active").strip() + m.group(3),
        nav_html,
        count=1,
    )


def process(html_text: str) -> str:
    """Substitute all #include sentinels in a single HTML string."""
    page_key_match = BODY_DATA_PAGE_RE.search(html_text)
    page_key = page_key_match.group("page") if page_key_match else None

    def sub(m: re.Match) -> str:
        partial_path = m.group("path")
        partial_text = load_partial(partial_path)
        if partial_path.endswith("_nav.html"):
            partial_text = apply_active(partial_text, page_key)
        return (
            f"<!-- #include {partial_path} -->\n"
            f"{partial_text}\n"
            f"<!-- /include -->"
        )

    return INCLUDE_RE.sub(sub, html_text)


def iter_html(paths: list[Path]) -> list[Path]:
    out: list[Path] = []
    for p in paths:
        if p.is_dir():
            out.extend(sorted(p.rglob("*.html")))
        elif p.suffix == ".html":
            out.append(p)
    return [p for p in out if "/partials/" not in str(p)]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "paths",
        nargs="*",
        default=[str(ROOT)],
        help="files or directories to process (default: landing/)",
    )
    ap.add_argument(
        "--check",
        action="store_true",
        help="exit 1 if any file would change (no writes)",
    )
    args = ap.parse_args()

    changed = 0
    seen = 0
    for path in iter_html([Path(p) for p in args.paths]):
        seen += 1
        src = path.read_text(encoding="utf-8")
        out = process(src)
        if out != src:
            changed += 1
            if args.check:
                print(f"would change: {path.relative_to(ROOT)}")
            else:
                path.write_text(out, encoding="utf-8")
                print(f"updated: {path.relative_to(ROOT)}")

    if args.check:
        print(f"\n{changed}/{seen} files would change")
        return 1 if changed else 0

    print(f"\n{changed}/{seen} files updated")
    return 0


if __name__ == "__main__":
    sys.exit(main())
