#!/usr/bin/env python3
"""Validate a Marqdo ecosystem Git tree.

The checker follows the hierarchical governance rules of the Marqdo ecosystem
specification (sections 2-4, 13):

* the root catalog describes direct components and domain aggregates;
* a registered ``kind = "domain"`` gitlink is expanded at its exact pinned
  commit for ``--checkouts`` or to validate URLs before updating its children;
* each aggregate layer has its own ``components.toml`` / ``.gitmodules`` and
  registers only its direct children;
* governance depth is limited to two gitlink levels (root -> domain -> leaf);
* global ids and remotes are unique, dependency edges resolve globally and
  ``requires`` stays acyclic while ``checks_with`` may contain cycles.

Commit identity comes from the Git tree (gitlinks), the authority for remotes
is ``.gitmodules`` and the catalog never repeats SHAs.  The checker never
fetches from a network remote, never runs ``git submodule update`` and only
reads objects already present in the local repositories.
"""

import argparse
import graphlib
import re
import subprocess
import sys
import tomllib
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
NAME = r"[a-z][a-z0-9]*(?:-[a-z0-9]+)*"
FIELDS = {"id", "path", "kind", "state", "owner", "scope", "requires", "checks_with", "release"}
ROOT_FILES = {".gitattributes", ".gitignore", ".gitmodules", "AGENTS.md", "LICENSE", "README.md", "components.toml"}
ROOT_DIRS = {".gitea", ".woodpecker", "ci", "docs", "examples", "scripts"}
CATEGORY_READMES = {"stdlib/README.md", "extensions/README.md"}
RELEASE_ROLES = {"core": "base", "stdlib": "base", "extension": "optional", "tool": "optional", "test": "none"}
DOMAIN_RELEASE = {"none", "base"}
DOMAIN_FILES = {".gitattributes", ".gitignore", ".gitmodules", "AGENTS.md", "LICENSE", "README.md", "components.toml"}
DOMAIN_DIRS = {".gitea", ".woodpecker", "docs", "examples", "scripts", "tests"}
DOMAIN_CHILD_KINDS = {"stdlib", "extension", "tool"}
AUTHORITY = "https://git.cflmy.cn/cflmy"
ENTRY = r"(?:[a-zA-Z0-9_-]+/)*[a-zA-Z0-9_-][a-zA-Z0-9_.-]*"


def git(*args, cwd=ROOT):
    return subprocess.check_output(["git", "-C", str(cwd), *args]).decode()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def nfc(value):
    return unicodedata.normalize("NFC", value)


class Layer:
    """One governance level: the root tree or one expanded domain aggregate."""

    def __init__(self, repo, revision, label, depth, order, components, gitlinks, linked, urls):
        self.repo = Path(repo)
        self.revision = revision
        self.label = label
        self.depth = depth
        self.order = order
        self.components = components
        self.gitlinks = gitlinks
        self.linked = linked
        self.urls = urls


def read_blob(repo, revision, path, label):
    try:
        return git("show", f"{revision}:{path}", cwd=repo)
    except subprocess.CalledProcessError:
        return None


def check_relative_path(path, id, label):
    require(isinstance(path, str) and path, f"{label}: invalid path for {id}: {path!r}")
    require("\\" not in path and not path.startswith("/") and not path.endswith("/"),
            f"{label}: invalid path for {id}: {path!r}")
    segments = path.split("/")
    require(all(segment and segment not in {".", ".."} for segment in segments),
            f"{label}: invalid path for {id}: {path!r}")
    require(all(re.fullmatch(NAME, segment) for segment in segments),
            f"{label}: invalid path for {id}: {path!r}")
    require(nfc(path) == path, f"{label}: non-NFC path for {id}: {path!r}")
    return path


def expected_root_path(component):
    kind = component["kind"]
    id = component["id"]
    if kind == "core":
        return "cli" if id == "cli" else None
    if kind == "test":
        return "tests/conformance" if id == "conformance" else None
    if kind == "stdlib":
        return f"stdlib/{id.removeprefix('std-')}" if id.startswith("std-") else None
    if kind == "extension":
        return f"extensions/{id}"
    if kind == "tool":
        return f"tools/{id}"
    if kind == "domain":
        return "stdlib" if id == "stdlib" else f"extensions/{id}"
    return None


def expected_remote(component, depth):
    """Derive the single approved self-hosted HTTPS remote for a component.

    Root direct components keep their historical ``marqdo-<id>`` /
    ``marqdo-ext-<id>`` / ``marqdo-extension-conformance`` repository names.
    Components registered by a domain aggregate are independently named
    ``marqdo-<id>`` (for example ``marqdo-web-design``).
    """
    id = component["id"]
    kind = component["kind"]
    if depth == 0:
        if kind == "test":
            name = "extension-conformance"
        elif kind == "extension":
            name = f"ext-{id}"
        elif kind == "domain":
            name = "stdlib" if id == "stdlib" else f"ext-{id}"
        else:
            name = id
    else:
        name = id
    return f"{AUTHORITY}/marqdo-{name}.git"


def check_component(raw, depth, label, by_id, paths, path_keys):
    require(isinstance(raw, dict), f"{label}: invalid component entry: {raw!r}")
    id = raw.get("id")
    state = raw.get("state")
    require(state in {"active", "planned"}, f"{label}: invalid state: {id!r}")
    fields = FIELDS | ({"entry"} if state == "active" else set())
    require(set(raw) == fields, f"{label}: invalid fields for {id!r}: {sorted(set(raw) ^ fields)}")
    require(isinstance(id, str) and re.fullmatch(NAME, id), f"{label}: invalid id: {id!r}")

    for key in ("requires", "checks_with"):
        items = raw[key]
        require(isinstance(items, list) and all(isinstance(item, str) for item in items),
                f"{label}: invalid {key}: {id}")
        require(id not in items and len(set(items)) == len(items), f"{label}: invalid {key}: {id}")

    kind = raw["kind"]
    require(kind in RELEASE_ROLES or kind == "domain", f"{label}: unknown kind: {id}: {kind!r}")
    if kind == "domain":
        require(depth == 0, f"{label}: domain kind is only allowed at the root layer: {id}")
        require(raw["release"] in DOMAIN_RELEASE, f"{label}: invalid release role: {id}")
        require(raw["requires"] == [], f"{label}: domain requires must be empty: {id}")
    else:
        require(depth == 0 or kind in DOMAIN_CHILD_KINDS,
                f"{label}: component kind is not allowed inside a domain: {id} ({kind})")
        require(raw["release"] == RELEASE_ROLES[kind], f"{label}: invalid release role: {id}")

    path = check_relative_path(raw["path"], id, label)
    if depth == 0:
        expected = expected_root_path(raw)
        require(expected is not None and path == expected,
                f"{label}: invalid path: {id}: {path} (expected {expected})")
    require(path not in paths, f"{label}: invalid or duplicate path: {path}")
    key = nfc(path).casefold()
    require(key not in path_keys, f"{label}: path case/NFC conflict: {id}: {path}")
    for other in paths:
        require(not path.startswith(other + "/") and not other.startswith(path + "/"),
                f"{label}: overlapping component path: {path} / {other}")

    for key in ("owner", "scope"):
        require(isinstance(raw[key], str) and raw[key].strip(), f"{label}: missing {key}: {id}")
    if state == "active":
        require(isinstance(raw["entry"], str) and re.fullmatch(ENTRY, raw["entry"]),
                f"{label}: unsafe entry: {id}")

    paths.add(path)
    path_keys.add(key)
    by_id[id] = raw
    return raw


def parse_catalog(repo, revision, label):
    text = read_blob(repo, revision, "components.toml", label)
    require(text is not None, f"{label}: missing components.toml at {revision}")
    try:
        data = tomllib.loads(text)
    except tomllib.TOMLDecodeError as error:
        raise ValueError(f"{label}: invalid components.toml: {error}") from None
    require(set(data) == {"component"} and isinstance(data["component"], list), f"{label}: invalid catalog")
    return data["component"]


def tree_entries(repo, revision, label):
    try:
        output = git("ls-tree", "-rz", "--full-tree", revision, cwd=repo)
    except subprocess.CalledProcessError:
        raise ValueError(f"{label}: missing Git tree at {revision}") from None
    entries = {}
    for record in output.split("\0"):
        if record:
            metadata, path = record.split("\t", 1)
            mode, _, sha = metadata.split()
            entries[path] = (mode, sha)
    return entries


def check_root_tree(entries, linked, unlinked, label):
    allowed_readmes = CATEGORY_READMES | {
        f'{item["path"]}/README.md'
        for item in unlinked
        if item["kind"] in {"stdlib", "extension", "domain"}
    }
    for path, (mode, _) in entries.items():
        root = path.split("/", 1)[0]
        require(
            path in linked or (mode == "100644" and path in allowed_readmes)
            or (mode in {"100644", "100755"} and (
                path in ROOT_FILES or root in ROOT_DIRS and "/" in path or path.startswith("tests/integration/")
            )),
            f"{label}: unexpected tracked path or mode: {path} ({mode})",
        )


def check_domain_tree(entries, linked, unlinked, label):
    """Aggregate layers only carry governance, docs, examples and tests.

    Production implementations stay in the child repositories: an aggregate
    may not add a renderer, parser or driver under its own tree.
    """
    allowed_readmes = {f'{item["path"]}/README.md' for item in unlinked}
    for path, (mode, _) in entries.items():
        if path in linked:
            continue
        if mode == "100644" and path in allowed_readmes:
            continue
        top = path.split("/", 1)[0]
        top_level_doc = mode == "100644" and "/" not in path and path.endswith(".md")
        require(
            mode in {"100644", "100755"} and (
                path in DOMAIN_FILES or top_level_doc or (top in DOMAIN_DIRS and "/" in path)
            ),
            f"{label}: unexpected aggregate tracked path or mode: {path} ({mode})",
        )


def read_submodules(repo, revision, label):
    if read_blob(repo, revision, ".gitmodules", label) is None:
        return {}
    try:
        output = git("config", "--null", "--blob", f"{revision}:.gitmodules", "--list", cwd=repo)
    except subprocess.CalledProcessError:
        raise ValueError(f"{label}: invalid .gitmodules") from None
    config = {}
    for record in output.split("\0"):
        if record:
            key, value = record.split("\n", 1)
            require(key not in config, f"{label}: duplicate submodule setting: {key}")
            config[key] = value
    return config


def check_submodules(config, linked, depth, label, urls):
    """Match ``.gitmodules`` sections by ``path``, not by section name.

    Git configuration keys are keyed by the submodule section name, which is
    not required to equal the checked-out path.  Path and URL remain the
    authority for identity; any other setting (for example an ``update``
    command or a relative override) is rejected as unexpected.
    """
    sections = {}
    extras = []
    for key, value in config.items():
        match = re.fullmatch(r"submodule\.(.+)\.(path|url)", key)
        if not match:
            extras.append(key)
            continue
        name, field = match.groups()
        sections.setdefault(name, {})[field] = value
    by_path = {}
    for name, fields in sections.items():
        require(set(fields) == {"path", "url"},
                f"{label}: unexpected submodule settings: "
                f"{sorted(f'submodule.{name}.{field}' for field in set(fields) ^ {'path', 'url'})}")
        path = fields["path"]
        require(path not in by_path, f"{label}: duplicate submodule path: {path}")
        by_path[path] = fields["url"]
    require(not extras, f"{label}: unexpected submodule settings: {sorted(extras)}")
    for path, component in linked.items():
        require(path in by_path, f"{label}: missing or invalid submodule path: {path}")
        url = by_path.pop(path)
        expected = expected_remote(component, depth)
        require(url == expected, f"{label}: invalid submodule URL: {path}: {url!r}")
        require(url not in urls, f"{label}: duplicate submodule remote: {path}: {url}")
        urls[url] = path
    require(not by_path, f"{label}: unexpected submodule settings: {sorted(by_path)}")


def read_layer(repo, revision, label, depth):
    repo = Path(repo)
    order = []
    components = {}
    paths = set()
    path_keys = set()
    for raw in parse_catalog(repo, revision, label):
        order.append(check_component(raw, depth, label, components, paths, path_keys))
    entries = tree_entries(repo, revision, label)
    gitlinks = {path: sha for path, (mode, sha) in entries.items() if mode == "160000"}
    active_paths = {item["path"] for item in order if item["state"] == "active"}
    mismatch = (active_paths - gitlinks.keys()) | (gitlinks.keys() - paths)
    require(not mismatch, f"{label}: gitlink/catalog mismatch: {sorted(mismatch)}")
    linked = {item["path"]: item for item in order if item["path"] in gitlinks}
    unlinked = [item for item in order if item["path"] not in gitlinks]
    if depth == 0:
        check_root_tree(entries, linked, unlinked, label)
    else:
        check_domain_tree(entries, linked, unlinked, label)
    urls = {}
    check_submodules(read_submodules(repo, revision, label), linked, depth, label, urls)
    return Layer(repo, revision, label, depth, order, components, gitlinks, linked, urls)


def verify_checkout(layer, component):
    path = component["path"]
    checkout = layer.repo / path
    sha = layer.gitlinks[path]
    require(checkout.resolve() == checkout and (checkout / ".git").exists(),
            f"{layer.label}: missing submodule checkout: {path}")
    try:
        git("cat-file", "-e", f"{sha}^{{commit}}", cwd=checkout)
    except subprocess.CalledProcessError:
        raise ValueError(f"{layer.label}: missing committed object: {path}@{sha}") from None
    require(git("rev-parse", "HEAD", cwd=checkout).strip() == sha,
            f"{layer.label}: wrong checked-out commit: {path}")
    status = ["status", "--porcelain"]
    if component["kind"] == "domain":
        # Child dirt is the responsibility of the child checkout and is
        # reported by its own leaf check, not by the aggregate repository.
        status.append("--ignore-submodules=all")
    require(not git(*status, cwd=checkout),
            f"{layer.label}: dirty submodule: {path}")
    if component["state"] == "active":
        entry = checkout / component["entry"]
        require(entry.is_file() and entry.resolve() == entry,
                f"{layer.label}: missing or symbolic entry: {path}/{component['entry']}")
    return checkout


def verify_leaf(layer, component):
    checkout = verify_checkout(layer, component)
    sha = layer.gitlinks[component["path"]]
    label = f'{layer.label} leaf {component["id"]} ({component["path"]})'
    nested = tree_entries(checkout, sha, label)
    gitlinks = sorted(path for path, (mode, _) in nested.items() if mode == "160000")
    require(not gitlinks, f"{label}: leaf must not register gitlinks: {gitlinks}")


def validate_graph(layers):
    components = {}
    for layer in layers:
        for component in layer.order:
            id = component["id"]
            require(id not in components, f"{layer.label}: duplicate global id: {id}")
            components[id] = (component, layer)
    remotes = {}
    for layer in layers:
        for url, path in layer.urls.items():
            require(url not in remotes, f"{layer.label}: duplicate global remote: {path}: {url}")
            remotes[url] = path
    for layer in layers:
        for component in layer.order:
            id = component["id"]
            for field in ("requires", "checks_with"):
                for dependency in component[field]:
                    require(dependency in components, f"{layer.label}: unknown dependency: {id} -> {dependency}")
                    target, _ = components[dependency]
                    require(component["state"] != "active" or target["state"] == "active",
                            f"{layer.label}: active depends on planned: {id} -> {dependency}")
                    if field != "requires":
                        continue
                    target_kind = target["kind"]
                    source_kind = component["kind"]
                    require(target_kind != "domain",
                            f"{layer.label}: requires must not target a domain: {id} -> {dependency}")
                    if source_kind in {"core", "domain"}:
                        require(False, f"{layer.label}: dependency violation: {id} must not require {dependency}")
                    if source_kind == "stdlib":
                        require(target_kind in {"core", "stdlib"},
                                f"{layer.label}: stdlib dependency violation: {id} -> {dependency}")
                    elif source_kind == "tool":
                        require(target_kind in {"core", "extension"},
                                f"{layer.label}: tool dependency violation: {id} -> {dependency}")
                    elif source_kind == "extension":
                        require(target_kind in {"core", "stdlib", "extension"},
                                f"{layer.label}: extension dependency violation: {id} -> {dependency}")
    try:
        graph = {id: list(item["requires"]) for id, (item, _) in components.items()}
        tuple(graphlib.TopologicalSorter(graph).static_order())
    except graphlib.CycleError as error:
        raise ValueError(f"root: requires DAG cycle: {error}") from None


def validate(revision, checkouts, expand_domains=False):
    layers = [read_layer(ROOT, revision, "root", 0)]
    if checkouts or expand_domains:
        root = layers[0]
        for path, component in root.linked.items():
            if component["kind"] != "domain":
                if checkouts:
                    verify_leaf(root, component)
                continue
            domain_repo = verify_checkout(root, component)
            domain = read_layer(domain_repo, root.gitlinks[path],
                                f"domain {component['id']} ({path})", 1)
            layers.append(domain)
            if checkouts:
                for child in domain.linked.values():
                    verify_leaf(domain, child)
    validate_graph(layers)
    return layers


def active_entries(layers):
    root = layers[0]
    root_domains = {root.repo / path: item["id"] for path, item in root.linked.items()
                    if item["kind"] == "domain"}
    entries = {}
    requirements = {}
    for layer in layers:
        parent = root_domains.get(layer.repo)
        for path, item in sorted(layer.linked.items()):
            if item["state"] != "active":
                continue
            component_id = item["id"]
            relative = (layer.repo / path).relative_to(root.repo).as_posix()
            entries[component_id] = (component_id, item["kind"], relative, item["entry"])
            requirements[component_id] = list(item["requires"])
            if parent in requirements:
                requirements[parent].append(component_id)
    for component_id in graphlib.TopologicalSorter(requirements).static_order():
        yield entries[component_id]


def main():
    parser = argparse.ArgumentParser(description="Validate a Marqdo ecosystem Git tree")
    parser.add_argument("revision", nargs="?", default="HEAD")
    parser.add_argument("--checkouts", action="store_true")
    paths = parser.add_mutually_exclusive_group()
    paths.add_argument("--active-paths", action="store_true", help="print only direct active component paths")
    paths.add_argument("--active-entries", action="store_true", help="print validated active entries in dependency order")
    paths.add_argument("--domain-paths", action="store_true", help="validate domain URLs and print domain paths")
    paths.add_argument("--root-domain-paths", action="store_true", help="print root domain gitlinks before checkout")
    args = parser.parse_args()
    try:
        revision = git("rev-parse", "--verify", args.revision + "^{commit}").strip()
        layers = validate(revision, args.checkouts or args.active_entries, expand_domains=args.domain_paths)
        if args.active_entries:
            for entry in active_entries(layers):
                print("\t".join(entry))
        elif args.active_paths:
            for path, component in sorted(layers[0].linked.items()):
                if component["state"] == "active":
                    print(path)
        elif args.domain_paths or args.root_domain_paths:
            for path, component in sorted(layers[0].linked.items()):
                if component["kind"] == "domain":
                    print(path)
        else:
            for layer in layers:
                for path, component in sorted(layer.linked.items()):
                    relative = (layer.repo / path).relative_to(ROOT).as_posix()
                    print(f'{component["id"]} {relative} {layer.gitlinks[path]} {component["state"]}')
    except (AttributeError, KeyError, OSError, subprocess.CalledProcessError, TypeError, ValueError) as error:
        sys.exit(f"layout: {error}")


if __name__ == "__main__":
    main()
