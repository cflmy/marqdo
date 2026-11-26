#!/usr/bin/env python3
import argparse
import graphlib
import re
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
NAME = r"[a-z][a-z0-9]*(?:-[a-z0-9]+)*"
FIELDS = {"id", "path", "kind", "state", "owner", "scope", "requires", "checks_with", "release"}
ROOT_FILES = {".gitattributes", ".gitignore", ".gitmodules", "AGENTS.md", "LICENSE", "README.md", "components.toml"}
ROOT_DIRS = {".gitea", ".woodpecker", "docs", "examples", "scripts"}
RELEASE_ROLES = {"core": "base", "stdlib": "base", "extension": "optional", "tool": "optional", "test": "none"}


def git(*args, cwd=ROOT):
    return subprocess.check_output(["git", "-C", cwd, *args]).decode()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def catalog(revision):
    components = tomllib.loads(git("show", f"{revision}:components.toml"))
    require(set(components) == {"component"} and isinstance(components["component"], list), "invalid catalog")
    by_id = {}
    paths = set()
    for component in components["component"]:
        state = component.get("state")
        fields = FIELDS | ({"entry"} if state == "active" else set())
        require(set(component) == fields and state in {"active", "planned"}, f"invalid fields: {component}")
        id = component["id"]
        kind = component["kind"]
        require(isinstance(id, str) and re.fullmatch(NAME, id) and id not in by_id, f"invalid or duplicate id: {id}")
        require(kind in RELEASE_ROLES and component["release"] == RELEASE_ROLES[kind], f"invalid release role: {id}")
        path = component["path"]
        expected = {
            "core": "cli" if id == "cli" else None,
            "test": "tests/conformance" if id == "conformance" else None,
            "stdlib": f"stdlib/{id.removeprefix('std-')}" if id.startswith("std-") else None,
            "extension": f"extensions/{id}",
            "tool": f"tools/{id}",
        }[kind]
        require(path == expected and path not in paths, f"invalid or duplicate path: {path}")
        for key in ("owner", "scope"):
            require(isinstance(component[key], str) and component[key].strip(), f"missing {key}: {id}")
        for key in ("requires", "checks_with"):
            items = component[key]
            require(isinstance(items, list) and all(isinstance(item, str) for item in items), f"invalid {key}: {id}")
            require(id not in items and len(set(items)) == len(items), f"invalid {key}: {id}")
        if state == "active":
            require(re.fullmatch(r"(?:[a-zA-Z0-9_-]+/)*[a-zA-Z0-9_-][a-zA-Z0-9_.-]*", component["entry"]), f"unsafe entry: {id}")
        paths.add(path)
        by_id[id] = component

    for id, component in by_id.items():
        for field in ("requires", "checks_with"):
            for dependency in component[field]:
                require(dependency in by_id, f"unknown dependency: {id} -> {dependency}")
                target = by_id[dependency]
                require(component["state"] != "active" or target["state"] == "active", f"active depends on planned: {id} -> {dependency}")
                if field == "requires":
                    require(component["kind"] != "stdlib" or target["kind"] in {"core", "stdlib"}, f"stdlib dependency violation: {id} -> {dependency}")
                    require(component["kind"] != "tool" or target["kind"] == "core", f"tool dependency violation: {id} -> {dependency}")
    tuple(graphlib.TopologicalSorter({id: item["requires"] for id, item in by_id.items()}).static_order())
    return by_id


def tree(revision):
    entries = {}
    for record in git("ls-tree", "-rz", "--full-tree", revision).split("\0"):
        if record:
            metadata, path = record.split("\t", 1)
            mode, _, sha = metadata.split()
            entries[path] = (mode, sha)
    return entries


def validate(revision, checkouts):
    components = catalog(revision)
    active = {item["path"]: item for item in components.values() if item["state"] == "active"}
    entries = tree(revision)
    gitlinks = {path: sha for path, (mode, sha) in entries.items() if mode == "160000"}
    require(gitlinks.keys() == active.keys(), f"gitlink/catalog mismatch: {sorted(gitlinks.keys() ^ active.keys())}")
    for path, (mode, _) in entries.items():
        root = path.split("/", 1)[0]
        require(
            path in active or (mode in {"100644", "100755"} and (
                path in ROOT_FILES or root in ROOT_DIRS and "/" in path or path.startswith("tests/integration/")
            )),
            f"unexpected tracked path or mode: {path} ({mode})",
        )

    config = {}
    for record in git("config", "--null", "--blob", f"{revision}:.gitmodules", "--list").split("\0"):
        if record:
            key, value = record.split("\n", 1)
            require(key not in config, f"duplicate submodule setting: {key}")
            config[key] = value
    for path, item in active.items():
        prefix = f"submodule.{path}."
        require(config.pop(prefix + "path", None) == path, f"missing or invalid submodule path: {path}")
        name = item["id"]
        if item["kind"] == "extension":
            name = f"ext-{name}"
        elif item["kind"] == "test":
            name = "extension-conformance"
        url = config.pop(prefix + "url", "")
        require(url == f"https://git.cflmy.cn/cflmy/marqdo-{name}.git", f"invalid submodule URL: {path}")
        if checkouts:
            checkout = ROOT / path
            sha = gitlinks[path]
            require(checkout.resolve() == checkout and (checkout / ".git").exists(), f"missing submodule checkout: {path}")
            require(git("rev-parse", "HEAD", cwd=checkout).strip() == sha, f"wrong checked-out commit: {path}")
            require(not git("status", "--porcelain", cwd=checkout), f"dirty submodule: {path}")
            entry = checkout / item["entry"]
            require(entry.is_file() and entry.resolve() == entry, f"missing or symbolic entry: {path}/{item['entry']}")
        print(f"{item['id']} {path} {gitlinks[path]}")
    require(not config, f"unexpected submodule settings: {sorted(config)}")


def main():
    parser = argparse.ArgumentParser(description="Validate a Marqdo ecosystem Git tree")
    parser.add_argument("revision", nargs="?", default="HEAD")
    parser.add_argument("--checkouts", action="store_true")
    args = parser.parse_args()
    try:
        validate(git("rev-parse", "--verify", args.revision + "^{commit}").strip(), args.checkouts)
    except (AttributeError, KeyError, OSError, subprocess.CalledProcessError, TypeError, ValueError) as error:
        sys.exit(f"layout: {error}")


if __name__ == "__main__":
    main()
