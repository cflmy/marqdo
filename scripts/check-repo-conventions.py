#!/usr/bin/env python3
"""Check ecosystem repository development conventions.

This is a repository-hygiene gate, not a new catalog: it reads the existing
root (and domain) components.toml, .gitmodules and gitlinks. It enforces the
rules in docs/REPOSITORY-CONVENTIONS.md for active repositories, the AGENTS.md
marker/links, real entry files, no duplicated descendant paths in the root
catalog, no nested submodules in leaf components and no second catalog.

Exit code 0 = PASS, 1 = violations, 2 = usage/environment failure.
"""

from __future__ import annotations

import argparse
import re
import sys
import tomllib
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parents[1]
MARKER = "<!-- marqdo:repo-conventions v1 -->"
CONVENTIONS_URL = (
    "https://git.cflmy.cn/cflmy/marqdo/src/branch/main/docs/REPOSITORY-CONVENTIONS.md"
)
ROOT_AGENTS_LINK = "docs/REPOSITORY-CONVENTIONS.md"


class Error(Exception):
    pass


def read_toml(path: Path) -> dict:
    try:
        with path.open("rb") as handle:
            return tomllib.load(handle)
    except FileNotFoundError as error:
        raise Error(f"missing catalog: {path.relative_to(PROJECT_ROOT)}") from error
    except tomllib.TOMLDecodeError as error:
        raise Error(f"invalid TOML {path.relative_to(PROJECT_ROOT)}: {error}") from error


def components(root: Path) -> list[dict]:
    return read_toml(root / "components.toml").get("component", [])


def submodule_urls(root: Path) -> dict[str, str]:
    path = root / ".gitmodules"
    if not path.is_file():
        return {}
    result: dict[str, str] = {}
    current_path = None
    for raw in path.read_text().splitlines():
        line = raw.strip()
        if line.startswith("[submodule "):
            current_path = None
            continue
        if line.startswith("path ="):
            current_path = line.split("=", 1)[1].strip()
        elif line.startswith("url =") and current_path:
            result[current_path] = line.split("=", 1)[1].strip()
    return result


def check_active(root: Path, component: dict, errors: list[str]) -> None:
    identifier = component.get("id", "?")
    rel = component.get("path", "")
    checkout = root / rel
    if not rel:
        errors.append(f"{identifier}: active component has no path")
        return
    if not checkout.is_dir():
        errors.append(f"{identifier}: checkout path missing: {rel}")
        return
    readme = checkout / "README.md"
    if not readme.is_file() or readme.stat().st_size < 100:
        errors.append(f"{identifier}: README.md missing or too small")
    agents = checkout / "AGENTS.md"
    if not agents.is_file():
        errors.append(f"{identifier}: AGENTS.md missing")
    else:
        text = agents.read_text()
        if MARKER not in text:
            errors.append(f"{identifier}: AGENTS.md lacks `{MARKER}`")
        if CONVENTIONS_URL not in text:
            errors.append(f"{identifier}: AGENTS.md does not link the root conventions")
        if len(text) < 300:
            errors.append(f"{identifier}: AGENTS.md is too small to state boundaries")
    entry = component.get("entry", "")
    if not entry:
        errors.append(f"{identifier}: active component has no entry")
    elif not (checkout / entry).is_file():
        errors.append(f"{identifier}: entry file missing: {rel}/{entry}")
    for forbidden in ("components.toml", ".gitmodules"):
        if (checkout / forbidden).is_file() and component.get("kind") != "domain":
            errors.append(f"{identifier}: leaf component must not own {forbidden}")


def check_planned(component: dict, errors: list[str]) -> None:
    identifier = component.get("id", "?")
    if component.get("entry"):
        errors.append(f"{identifier}: planned component must not declare entry")


def flatten(root: Path) -> tuple[list[dict], dict[str, dict]]:
    root_components = components(root)
    by_id: dict[str, dict] = {}
    for component in root_components:
        identifier = component.get("id")
        if identifier in by_id:
            raise Error(f"duplicate global component id: {identifier}")
        by_id[identifier] = component
    children: dict[str, dict] = {}
    for component in root_components:
        if component.get("kind") != "domain":
            continue
        domain = root / component.get("path", "")
        catalog = domain / "components.toml"
        if not catalog.is_file():
            raise Error(f"domain {component.get('id')} has no components.toml")
        for child in read_toml(catalog).get("component", []):
            child = dict(child)
            child["domain_path"] = component["path"]
            identifier = child.get("id")
            if identifier in by_id:
                raise Error(f"duplicate global component id: {identifier}")
            by_id[identifier] = child
            child["path"] = f"{component['path']}/{child.get('path', '')}"
            children[identifier] = child
    return root_components + list(children.values()), by_id


def check(root: Path) -> list[str]:
    errors: list[str] = []
    flat, by_id = flatten(root)
    root_components = components(root)
    root_paths = {component.get("path") for component in root_components}
    domain_paths = [
        component["path"]
        for component in root_components
        if component.get("kind") == "domain" and component.get("path")
    ]
    for component in root_components:
        path = component.get("path", "")
        for domain_path in domain_paths:
            if path.startswith(f"{domain_path}/"):
                errors.append(
                    f"{component.get('id')}: descendant of {domain_path} is also registered in the root catalog"
                )
    for component in flat:
        if component.get("state") == "active":
            check_active(root, component, errors)
        elif component.get("state") == "planned":
            check_planned(component, errors)
        else:
            errors.append(f"{component.get('id')}: unknown state {component.get('state')!r}")
    root_agents = root / "AGENTS.md"
    if not root_agents.is_file():
        errors.append("root AGENTS.md missing")
    else:
        text = root_agents.read_text()
        if ROOT_AGENTS_LINK not in text:
            errors.append(f"root AGENTS.md does not link {ROOT_AGENTS_LINK}")
        if "防膨胀" not in text and "REPOSITORY-CONVENTIONS" not in text:
            errors.append("root AGENTS.md does not state the repository conventions")
    urls = submodule_urls(root)
    for component in root_components:
        path = component.get("path", "")
        if path not in urls:
            errors.append(f"{component.get('id')}: path {path!r} is not in root .gitmodules")
        else:
            url = urls[path]
            if not url.startswith("https://git.cflmy.cn/cflmy/") or not url.endswith(".git"):
                errors.append(f"{component.get('id')}: non-canonical submodule URL {url!r}")
    for component in root_components:
        if component.get("kind") == "domain":
            continue
        rel = component.get("path", "")
        checkout = root / rel
        if (checkout / "components.toml").is_file():
            errors.append(f"{component.get('id')}: leaf component owns a second catalog")
    return errors


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", default=str(PROJECT_ROOT), help="checkout root to inspect")
    parser.add_argument("--report", action="store_true", help="print the checked component set")
    args = parser.parse_args(argv)
    root = Path(args.root).resolve()
    try:
        errors = check(root)
        flat, _ = flatten(root)
    except Error as error:
        print(f"repo-conventions: error: {error}", file=sys.stderr)
        return 2
    if args.report:
        for component in flat:
            print(component.get("id"), component.get("path"), component.get("state"))
    if errors:
        for error in errors:
            print(f"repo-conventions: FAIL: {error}", file=sys.stderr)
        return 1
    print(f"repo-conventions: PASS ({len(flat)} components)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
