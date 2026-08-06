#!/usr/bin/env python3
"""Migrate branch arms: + cond -> N. cond (params + `name` unchanged)."""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SKIP_DIRS = {"target", ".git", "node_modules"}

PARAM_LINE = re.compile(r"^(\s{4,})\+ `([^`]+)`(=.*)?\s*$")
ORDERED = re.compile(r"^(\s*)(\d+)\.\s+(.*)$")
PLUS_BRANCH = re.compile(r"^(\s*)\+ (.+)$")


def migrate_text(text: str) -> str:
    lines = text.splitlines(keepends=True)
    out: list[str] = []
    arm = 0
    for line in lines:
        raw = line.rstrip("\n\r")
        end = line[len(raw) :]
        if not raw.strip():
            arm = 0
            out.append(line)
            continue
        if PARAM_LINE.match(raw):
            arm = 0
            out.append(line)
            continue
        m = PLUS_BRANCH.match(raw)
        if m:
            arm += 1
            indent, rest = m.groups()
            out.append(f"{indent}{arm}. {rest}{end}")
            continue
        om = ORDERED.match(raw)
        if om:
            arm = int(om.group(2))
            out.append(line)
            continue
        stripped = raw.lstrip()
        indent = raw[: len(raw) - len(stripped)]
        if indent == 0 and (
            stripped.startswith("#")
            or stripped.startswith(">")
            or (stripped.startswith("*") and stripped.endswith("*"))
            or stripped.startswith("`")
            or stripped.startswith("|")
            or stripped.startswith("---")
        ):
            arm = 0
        out.append(line)
    return "".join(out)


def should_skip(path: Path) -> bool:
    return any(part in SKIP_DIRS for part in path.parts)


def main() -> int:
    changed = 0
    for path in sorted(ROOT.rglob("*.mq.md")):
        if should_skip(path):
            continue
        text = path.read_text(encoding="utf-8")
        new_text = migrate_text(text)
        if new_text != text:
            path.write_text(new_text, encoding="utf-8")
            print(path.relative_to(ROOT))
            changed += 1
    print(f"migrated {changed} files")
    return 0


if __name__ == "__main__":
    sys.exit(main())
