#!/usr/bin/env python3
"""
One-shot: strip Scholar + LLMAGE personas from index.html.

ORDER OF OPERATIONS MATTERS:
1. Wrap <nav>...</nav> and <footer>...</footer> with include sentinels FIRST.
   This means everything inside the nav (theme-switcher, persona-spanned links,
   triple SVG variants, etc.) gets replaced by the canonical partial on build.
   Anything we'd otherwise strip inside the nav is now dead weight handled by
   replacement.
2. Then strip everything else (CSS rules, body-content persona spans, etc.).

Run once; delete after Step 5 lands.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).parent
TARGET = ROOT / "index.html"


# ─── Block-level wrap (run FIRST) ───────────────────────────────────────
NAV_RE = re.compile(r'<nav\b[^>]*>.*?</nav>', re.DOTALL)
FOOTER_RE = re.compile(r'<footer\b[^>]*>.*?</footer>', re.DOTALL)
BODY_OPEN_RE = re.compile(r'<body\b([^>]*)>')

# ─── Press Start 2P removal ─────────────────────────────────────────────
GOOGLE_FONTS_RE = re.compile(
    r'(<link\s+href="https://fonts\.googleapis\.com/css2[^"]*)'
    r'&family=Press\+Start\+2P([^"]*"\s+rel="stylesheet">)'
)
PRESS_START_DECL_RE = re.compile(
    r'font-family:\s*[\'"]Press Start 2P[\'"]\s*,\s*monospace\s*;\s*',
)
PRESS_START_VAR_RE = re.compile(
    r'^[ \t]*--font-display:\s*[\'"]Press Start 2P[\'"]\s*,\s*monospace\s*;\s*\n',
    re.MULTILINE,
)
# var(--font-display) — the variable is gone after the strip; fall through to mono.
FONT_DISPLAY_VAR_USE_RE = re.compile(r'var\(\s*--font-display\s*\)')

# ─── Body-content persona markup ───────────────────────────────────────
SCHOLAR_SPAN_RE = re.compile(
    r'<span\s+class="t-scholar"[^>]*>.*?</span>',
    re.DOTALL,
)
LLMAGE_SPAN_RE = re.compile(
    r'<span\s+class="t-llmage"[^>]*>.*?</span>',
    re.DOTALL,
)
WIZARD_SPAN_RE = re.compile(
    r'<span\s+class="t-wizard"[^>]*>(.*?)</span>',
    re.DOTALL,
)

# Visibility shim rule
T_VISIBILITY_RE = re.compile(
    r'^\.t-scholar,\s*\.t-wizard,\s*\.t-llmage\s*\{[^}]*\}\s*\n',
    re.MULTILINE,
)

# Hero crystal SVG variants (the body-area swap)
SCHOLAR_CRYSTAL_RE = re.compile(
    r'<!--\s*Scholar crystal[^-]*-->.*?<svg class="crystal-scholar".*?</svg>',
    re.DOTALL,
)
LLMAGE_CRYSTAL_RE = re.compile(
    r'<!--\s*LLMAGE[^-]*-->.*?<svg class="crystal-llmage".*?</svg>',
    re.DOTALL,
)


def strip_theme_rules(css: str, theme: str) -> tuple[str, int]:
    """Delete every CSS rule whose selector contains [data-theme="<theme>"].

    Uses brace-counting so nested-rule edge cases don't bite.
    """
    out: list[str] = []
    i = 0
    n = len(css)
    selector_re = re.compile(r'\[data-theme="' + re.escape(theme) + r'"\]')
    removed = 0

    while i < n:
        m = selector_re.search(css, i)
        if not m:
            out.append(css[i:])
            break

        # Find the start of the CSS rule (after last delimiter).
        # Boundary tokens have lengths 1 ('}', ';') or 2 ('*/'); track length.
        rule_start = m.start()
        candidates = [
            (css.rfind('}', 0, rule_start), 1),
            (css.rfind('*/', 0, rule_start), 2),
            (css.rfind(';', 0, rule_start), 1),
        ]
        best_pos, best_len = max(candidates, key=lambda c: c[0])
        rule_actual_start = 0 if best_pos == -1 else best_pos + best_len
        while rule_actual_start < n and css[rule_actual_start] in " \t":
            rule_actual_start += 1
        if rule_actual_start < n and css[rule_actual_start] == "\n":
            rule_actual_start += 1

        # Find the matching close brace.
        brace_open = css.find('{', m.end())
        if brace_open == -1:
            out.append(css[i:])
            break
        depth = 1
        j = brace_open + 1
        while j < n and depth > 0:
            if css[j] == '{':
                depth += 1
            elif css[j] == '}':
                depth -= 1
            j += 1
        rule_end = j
        if rule_end < n and css[rule_end] == "\n":
            rule_end += 1

        out.append(css[i:rule_actual_start])
        removed += 1
        i = rule_end

    return ''.join(out), removed


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


def main() -> int:
    src = TARGET.read_text(encoding="utf-8")
    original_len = len(src)
    original_lines = src.count("\n")
    print(f"index.html before: {original_lines} lines, {original_len:,} bytes\n")

    # 1. Wrap <nav> in include sentinel FIRST — its contents will be replaced.
    src, did = wrap_with_include(src, NAV_RE, "_nav.html")
    print(f"  <nav> wrapped with include sentinel              {'1' if did else '0':>5} wrapped")

    # 2. Wrap <footer> — or inject one if absent.
    src, did = wrap_with_include(src, FOOTER_RE, "_footer.html")
    if did:
        print(f"  <footer> wrapped with include sentinel           1 wrapped")
    else:
        src = src.replace(
            "</body>",
            "  <!-- #include partials/_footer.html -->\n"
            "<footer></footer>\n"
            "<!-- /include -->\n</body>",
            1,
        )
        print(f"  <footer> injected before </body>                 1 injected")

    # 3. Google Fonts link — drop Press+Start+2P
    before = src
    src = GOOGLE_FONTS_RE.sub(r"\1\2", src)
    print(f"  Google Fonts link                                "
          f"{'Press+Start+2P param dropped' if before != src else '(no Press+Start+2P found)'}")

    # 4. Press Start 2P font-family declarations (inline or line-anchored)
    n1 = len(PRESS_START_DECL_RE.findall(src))
    src = PRESS_START_DECL_RE.sub("", src)
    n2 = len(PRESS_START_VAR_RE.findall(src))
    src = PRESS_START_VAR_RE.sub("", src)
    n3 = len(FONT_DISPLAY_VAR_USE_RE.findall(src))
    src = FONT_DISPLAY_VAR_USE_RE.sub("var(--mono)", src)
    print(f"  Press Start 2P declarations                      {n1 + n2:>5} removed")
    if n3:
        print(f"  var(--font-display) → var(--mono)                {n3:>5} rewritten")

    # 5. [data-theme="scholar"] CSS rules
    src, n = strip_theme_rules(src, "scholar")
    print(f"  [data-theme=\"scholar\"] CSS rules                 {n:>5} removed")

    # 6. [data-theme="llmage"] CSS rules
    src, n = strip_theme_rules(src, "llmage")
    print(f"  [data-theme=\"llmage\"] CSS rules                  {n:>5} removed")

    # 7. .t-scholar body-content spans (nav is already wrapped, so safe)
    before = len(SCHOLAR_SPAN_RE.findall(src))
    src = SCHOLAR_SPAN_RE.sub("", src)
    print(f"  .t-scholar markup spans                          {before:>5} removed")

    # 8. .t-llmage body-content spans
    before = len(LLMAGE_SPAN_RE.findall(src))
    src = LLMAGE_SPAN_RE.sub("", src)
    print(f"  .t-llmage markup spans                           {before:>5} removed")

    # 9. .t-wizard body-content spans → unwrap (keep inner content)
    before = len(WIZARD_SPAN_RE.findall(src))
    src = WIZARD_SPAN_RE.sub(r"\1", src)
    print(f"  .t-wizard markup spans                           {before:>5} unwrapped")

    # 10. Visibility shim rule
    if T_VISIBILITY_RE.search(src):
        src = T_VISIBILITY_RE.sub("", src)
        print(f"  .t-* visibility shim                                 1 removed")

    # 11. Hero crystal SVG variants
    before = len(SCHOLAR_CRYSTAL_RE.findall(src))
    src = SCHOLAR_CRYSTAL_RE.sub("", src)
    print(f"  Scholar hero-crystal SVG                         {before:>5} removed")
    before = len(LLMAGE_CRYSTAL_RE.findall(src))
    src = LLMAGE_CRYSTAL_RE.sub("", src)
    print(f"  LLMAGE hero-crystal SVG                          {before:>5} removed")

    # 12. data-page="home"
    m = BODY_OPEN_RE.search(src)
    if m and 'data-page=' not in m.group(1):
        attrs = m.group(1).rstrip() + ' data-page="home"'
        src = src[: m.start()] + f"<body{attrs}>" + src[m.end() :]
        print(f"  data-page=\"home\" added to <body>                   1 attribute")

    TARGET.write_text(src, encoding="utf-8")
    new_len = len(src)
    new_lines = src.count("\n")
    diff_lines = original_lines - new_lines
    diff_bytes = original_len - new_len
    print(f"\nindex.html after:  {new_lines} lines, {new_len:,} bytes")
    print(f"DELTA: -{diff_lines} lines, -{diff_bytes:,} bytes "
          f"({diff_bytes * 100 // original_len}% reduction)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
