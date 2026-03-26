#!/usr/bin/env python3
"""
Verify that every named import in the HTML template exists as an export
in the wasm-bindgen generated JS.  Run after `wasm-bindgen` completes.

Usage:
    python3 scripts/check_wasm_exports.py <spout.js> <index.template.html>
"""
import re
import sys


def extract_html_named_imports(html: str) -> list[str]:
    """Return all names from `import init, { a, b, c } from '...'` lines."""
    names = []
    for group in re.findall(r"import\s+\w+\s*,\s*\{([^}]*)\}", html):
        for name in group.split(","):
            name = name.strip()
            if name:
                names.append(name)
    return names


def extract_js_exports(js: str) -> set[str]:
    """Return the set of names in `export { a, b, c }` blocks."""
    exports = set()
    for group in re.findall(r"export\s*\{([^}]*)\}", js):
        for name in group.split(","):
            # handle `name as alias` — take the alias (what callers see)
            parts = name.strip().split()
            if parts:
                exports.add(parts[-1])
    # also catch `export function foo` / `export async function foo`
    for name in re.findall(r"export\s+(?:async\s+)?function\s+(\w+)", js):
        exports.add(name)
    return exports


def main():
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <spout.js> <index.template.html>", file=sys.stderr)
        sys.exit(2)

    js_path, html_path = sys.argv[1], sys.argv[2]
    js = open(js_path).read()
    html = open(html_path).read()

    required = extract_html_named_imports(html)
    available = extract_js_exports(js)

    missing = [n for n in required if n not in available]
    if missing:
        print(f"ERROR: HTML imports names not found in {js_path}:", file=sys.stderr)
        for n in missing:
            print(f"  missing: {n}", file=sys.stderr)
        sys.exit(1)

    if required:
        print(f"OK: all {len(required)} named import(s) found in {js_path}: {required}")
    else:
        print(f"OK: no named imports in HTML (default-only import)")


if __name__ == "__main__":
    main()
