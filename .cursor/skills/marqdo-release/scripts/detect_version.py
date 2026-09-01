#!/usr/bin/env python3
"""Detect Marqdo version sources for release Phase 0. Never invent next SemVer."""
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

# …/marqdo/.cursor/skills/marqdo-release/scripts/this.py → parents[4] = repo root
ROOT = Path(__file__).resolve().parents[4]


def cargo_version(path: Path) -> str | None:
    if not path.is_file():
        return None
    m = re.search(r'^version\s*=\s*"([^"]+)"', path.read_text(encoding="utf-8"), re.M)
    return m.group(1) if m else None


def run(cmd: list[str]) -> str:
    try:
        out = subprocess.check_output(cmd, cwd=ROOT, stderr=subprocess.DEVNULL, text=True)
        return out.strip()
    except (subprocess.CalledProcessError, FileNotFoundError):
        return ""


def main() -> int:
    cargo = cargo_version(ROOT / "Cargo.toml")
    wasm = cargo_version(ROOT / "crates" / "marqdo-wasm" / "Cargo.toml")
    tags = run(["git", "tag", "-l", "v*", "--sort=-v:refname"])
    latest_tag = tags.splitlines()[0] if tags else "(none)"
    gh_list = run(["gh", "release", "list", "--limit", "3"])
    latest_gh = "(gh unavailable)"
    if gh_list:
        # first column is tag name for `gh release list`
        latest_gh = gh_list.splitlines()[0].split("\t")[0].strip()

    changelog = ROOT / "CHANGELOG.md"
    unreleased = False
    unreleased_preview = ""
    if changelog.is_file():
        text = changelog.read_text(encoding="utf-8")
        m = re.search(r"^## Unreleased\s*\n(.*?)(?=^## |\Z)", text, re.M | re.S)
        if m:
            body = m.group(1).strip()
            # nonempty if any bullet or non-heading content beyond empty section headers
            content_lines = [
                ln
                for ln in body.splitlines()
                if ln.strip() and not re.match(r"^###\s+", ln.strip())
            ]
            unreleased = bool(content_lines)
            unreleased_preview = " | ".join(content_lines[:3])[:200]

    print("=== Marqdo version detect (do NOT invent next; ask the user) ===")
    print(f"repo:              {ROOT}")
    print(f"Cargo.toml:        {cargo or '(missing)'}")
    print(f"marqdo-wasm:       {wasm or '(missing)'}")
    print(f"latest git tag:    {latest_tag}")
    print(f"latest gh release: {latest_gh}")
    print(f"Unreleased notes:  {'YES' if unreleased else 'NO/empty'}")
    if unreleased_preview:
        print(f"Unreleased peek:   {unreleased_preview}")
    if cargo and latest_tag.lstrip("v") != cargo:
        print("WARN: Cargo version != latest tag (expected during pre-bump or lag)")
    if cargo and wasm and cargo != wasm:
        print("WARN: root Cargo.toml and marqdo-wasm versions differ — sync before release")
    print()
    print("ASK USER: next SemVer X.Y.Z (tag will be vX.Y.Z)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
