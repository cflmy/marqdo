#!/usr/bin/env python3
"""One-shot migrate v0.2: params + `name`, foreach [`i`](`xs`)."""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SKIP_DIRS = {"man-test", "target", ".git", "node_modules"}

PARAM_RE = re.compile(
    r"^(\s+)- ([A-Za-z_\u4e00-\u9fff][A-Za-z0-9_\u4e00-\u9fff]*)(=(.*))?\s*$"
)
FOREACH_RE = re.compile(r"^(\s*)- \[([^\]]+)\]\(([^)]+)\)\s*$")


def wrap_id(name: str) -> str:
    name = name.strip()
    if name.startswith("`") and name.endswith("`"):
        return name
    return f"`{name}`"


def migrate_line(line: str) -> str:
    m = FOREACH_RE.match(line)
    if m:
        indent, item, coll = m.groups()
        return f"{indent}- [{wrap_id(item)}]({wrap_id(coll)})\n"
    m = PARAM_RE.match(line)
    if m:
        indent, name, _, default = m.groups()
        if default is not None:
            return f"{indent}+ {wrap_id(name)}={default.rstrip()}\n"
        return f"{indent}+ {wrap_id(name)}\n"
    return line


def should_skip(path: Path) -> bool:
    return any(part in SKIP_DIRS for part in path.parts)


def main() -> int:
    changed = 0
    for path in sorted(ROOT.rglob("*.mq.md")):
        if should_skip(path):
            continue
        text = path.read_text(encoding="utf-8")
        lines = text.splitlines(keepends=True)
        out = [migrate_line(ln) for ln in lines]
        new_text = "".join(out)
        if new_text != text:
            path.write_text(new_text, encoding="utf-8")
            print(path.relative_to(ROOT))
            changed += 1
    print(f"migrated {changed} files")
    return 0


if __name__ == "__main__":
    sys.exit(main())
