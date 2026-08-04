"""
Spike helpers: GFM parse via markdown-it-py + line-level bullet probe.
"""

from __future__ import annotations

import re
from dataclasses import dataclass

from markdown_it import MarkdownIt
from mdit_py_plugins.front_matter import front_matter_plugin


def make_md() -> MarkdownIt:
    # Avoid gfm-like's linkify (needs optional linkify-it-py).
    md = MarkdownIt("commonmark", {"html": False, "linkify": False})
    md.enable(["table"])
    md.use(front_matter_plugin)
    return md


@dataclass
class BulletLine:
    line_no: int
    marker: str  # "-" | "+" | "*" | "1."
    text: str


_BULLET = re.compile(
    r"^(?P<indent>\s*)(?P<marker>[-+*]|\d+\.)\s+(?P<text>.*)$"
)


def scan_bullets(source: str) -> list[BulletLine]:
    """Line-level scan: preserves '-' vs '+' regardless of MD library."""
    out: list[BulletLine] = []
    for i, line in enumerate(source.splitlines(), start=1):
        m = _BULLET.match(line)
        if not m:
            continue
        out.append(
            BulletLine(line_no=i, marker=m.group("marker"), text=m.group("text"))
        )
    return out


def parse_tokens(source: str):
    return make_md().parse(source)


def split_front_matter(source: str) -> tuple[str | None, str]:
    """
    Split leading --- ... --- front matter from body.
    Body-internal --- is left untouched.
    """
    if not source.startswith("---"):
        return None, source
    lines = source.splitlines(keepends=True)
    if not lines or lines[0].strip() != "---":
        return None, source
    for i in range(1, len(lines)):
        if lines[i].strip() == "---":
            fm = "".join(lines[1:i])
            body = "".join(lines[i + 1 :])
            return fm, body
    return None, source


_IMPORT = re.compile(r"^>\s*(?P<path>\S+\.mq\.md)\s*$")


def front_matter_imports(fm: str) -> list[str]:
    paths: list[str] = []
    for line in fm.splitlines():
        m = _IMPORT.match(line.strip())
        if m:
            paths.append(m.group("path"))
    return paths
