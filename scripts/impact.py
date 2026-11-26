#!/usr/bin/env python3
"""Recursive impact analysis between an accepted root commit and a candidate.

Compares the recursively expanded governance tree of two root revisions. A
component identity is `(global_id, parent, remote, SHA, state)`; a domain
gitlink change is expanded to its child identities so leaf changes are visible
even when the parent pointer did not change as a string.

The affected set is computed in the order the development contract requires:

1. changed or removed leaves;
2. reverse transitive closure over `requires` edges from the union of both
   graphs (removed edges cannot hide a consumer);
3. parent domains of affected components;
4. `checks_with` fixed point (co-validation is not a build prerequisite).

When a domain cannot be expanded (missing object/checkout) the result is marked
UNVERIFIED and the whole domain is treated as affected rather than silently
skipping it.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = "marqdo.impact"
SCHEMA_VERSION = 1


class ImpactError(Exception):
    pass


def git(repo: Path, *args: str) -> bytes:
    result = subprocess.run(["git", "-C", str(repo), *args], capture_output=True)
    if result.returncode != 0:
        raise ImpactError(
            f"git {' '.join(args)} failed in {repo}: "
            f"{result.stderr.decode(errors='replace').strip()}"
        )
    return result.stdout


def show(repo: Path, revision: str, path: str) -> bytes | None:
    result = subprocess.run(
        ["git", "-C", str(repo), "show", f"{revision}:{path}"], capture_output=True
    )
    return result.stdout if result.returncode == 0 else None


def resolve(repo: Path, revision: str) -> str:
    try:
        return (
            git(repo, "rev-parse", "--verify", revision + "^{commit}")
            .decode()
            .strip()
        )
    except ImpactError as error:
        raise ImpactError(f"cannot resolve revision {revision!r}: {error}") from error


def tree_entries(repo: Path, revision: str) -> dict[str, tuple[str, str]]:
    output = git(repo, "ls-tree", "-r", "-z", revision)
    entries: dict[str, tuple[str, str]] = {}
    for record in output.split(b"\0"):
        if not record:
            continue
        header, _, raw_path = record.partition(b"\t")
        fields = header.decode().split()
        if len(fields) != 3:
            raise ImpactError(f"invalid tree record in {repo}@{revision}")
        mode, _kind, sha = fields
        entries[raw_path.decode()] = (mode, sha)
    return entries


def submodule_urls(repo: Path, revision: str) -> dict[str, str]:
    """Map `.gitmodules` path to URL at a revision (authority for remotes)."""
    raw = show(repo, revision, ".gitmodules")
    if raw is None:
        return {}
    config = git(
        repo, "config", "--null", "--blob", f"{revision}:.gitmodules", "--list"
    )
    urls: dict[str, str] = {}
    pending: dict[str, dict[str, str]] = {}
    for record in config.split(b"\0"):
        if not record:
            continue
        key, _, value = record.decode(errors="replace").partition("\n")
        match = re.fullmatch(r"submodule\.(.+)\.(path|url)", key)
        if match:
            name, field = match.groups()
            pending.setdefault(name, {})[field] = value
    for fields in pending.values():
        if "path" in fields and "url" in fields:
            urls[fields["path"]] = fields["url"]
    return urls


def read_catalog(repo: Path, revision: str, label: str) -> list[dict]:
    raw = show(repo, revision, "components.toml")
    if raw is None:
        raise ImpactError(f"{label}: revision {revision} has no components.toml")
    try:
        catalog = tomllib.loads(raw.decode())
    except (UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
        raise ImpactError(f"{label}: invalid components.toml: {error}") from error
    components = catalog.get("component")
    if not isinstance(components, list):
        raise ImpactError(f"{label}: components.toml has no [[component]] entries")
    return components


def load_graph(
    repo: Path, revision: str, label: str, depth: int = 0
) -> tuple[dict[str, dict], list[str]]:
    """Expand one governance layer (root then domains) into global identities."""
    components = read_catalog(repo, revision, label)
    entries = tree_entries(repo, revision)
    urls = submodule_urls(repo, revision)
    gitlinks = {path: sha for path, (mode, sha) in entries.items() if mode == "160000"}
    graph: dict[str, dict] = {}
    unverified: list[str] = []
    for component in components:
        component_id = component.get("id")
        path = component.get("path")
        if not isinstance(component_id, str) or not isinstance(path, str):
            raise ImpactError(f"{label}: component without id/path")
        sha = gitlinks.get(path)
        identity = {
            "id": component_id,
            "parent": None,
            "path": path,
            "remote": urls.get(path),
            "sha": sha,
            "state": component.get("state"),
            "kind": component.get("kind"),
            "requires": sorted(component.get("requires", [])),
            "checks_with": sorted(component.get("checks_with", [])),
            "layer": label,
        }
        graph[component_id] = identity
        if component.get("kind") != "domain" or depth > 0:
            continue
        domain_repo = repo / path
        if sha is None or not (domain_repo / ".git").exists():
            unverified.append(
                f"{label}: domain {component_id} ({path}) cannot be expanded: "
                "missing gitlink or checkout"
            )
            continue
        try:
            children, child_unverified = load_graph(
                domain_repo, sha, f"domain {component_id}", depth + 1
            )
        except ImpactError as error:
            unverified.append(
                f"{label}: domain {component_id} ({path})@{sha} cannot be expanded: {error}"
            )
            continue
        unverified.extend(child_unverified)
        for child_id, child in children.items():
            if child_id in graph:
                raise ImpactError(f"{label}: duplicate global id {child_id}")
            child["parent"] = component_id
            graph[child_id] = child
    return graph, unverified


def identity_fields(component: dict) -> tuple:
    return (
        component.get("parent"),
        component.get("remote"),
        component.get("sha"),
        component.get("state"),
        tuple(component.get("requires", [])),
        tuple(component.get("checks_with", [])),
    )


def changed_components(
    accepted: dict[str, dict], candidate: dict[str, dict]
) -> list[dict]:
    changed = []
    for component_id in sorted(set(accepted) | set(candidate)):
        before = accepted.get(component_id)
        after = candidate.get(component_id)
        if before is None:
            changed.append({"id": component_id, "reason": "added"})
            continue
        if after is None:
            changed.append({"id": component_id, "reason": "removed"})
            continue
        if before.get("sha") != after.get("sha"):
            changed.append({"id": component_id, "reason": "sha"})
        elif before.get("state") != after.get("state"):
            changed.append({"id": component_id, "reason": "state"})
        elif before.get("remote") != after.get("remote"):
            changed.append({"id": component_id, "reason": "remote"})
        elif before.get("parent") != after.get("parent"):
            changed.append({"id": component_id, "reason": "parent"})
        elif before.get("requires") != after.get("requires") or before.get(
            "checks_with"
        ) != after.get("checks_with"):
            changed.append({"id": component_id, "reason": "dependencies"})
    return changed


def affected_set(
    accepted: dict[str, dict], candidate: dict[str, dict], changed: list[dict]
) -> tuple[list[str], list[str], list[str]]:
    graph = {**accepted, **candidate}
    dependents: dict[str, set[str]] = {component_id: set() for component_id in graph}
    for component in graph.values():
        for dependency in component.get("requires", []):
            dependents.setdefault(dependency, set()).add(component["id"])
    affected = {item["id"] for item in changed}
    queue = list(affected)
    while queue:
        current = queue.pop()
        for consumer in dependents.get(current, set()):
            if consumer not in affected:
                affected.add(consumer)
                queue.append(consumer)
    domains = set()
    for component_id in list(affected):
        parent = graph[component_id].get("parent")
        while parent:
            if parent not in affected:
                domains.add(parent)
            parent = graph[parent].get("parent") if parent in graph else None
    co_validation: set[str] = set()
    while True:
        additions = set()
        for component_id in affected:
            for partner in graph[component_id].get("checks_with", []):
                if partner not in affected:
                    additions.add(partner)
        if not additions:
            break
        affected |= additions
        co_validation |= additions
    return sorted(affected), sorted(domains), sorted(co_validation)


def consumers_of(component_id: str, revision: str) -> dict:
    """Consumers of one component at a revision (component-PR impact scope).

    Uses the accepted graph only: a component candidate cannot change the root
    graph it is evaluated against. Unknown component or unexpandable domain
    widens the scope to the whole graph (never silently skip).
    """
    revision_sha = resolve(ROOT, revision)
    graph, unverified = load_graph(ROOT, revision_sha, "root")
    if component_id not in graph:
        graph, unverified = load_graph(ROOT, revision_sha, "root")
        return {
            "schema": SCHEMA,
            "schema_version": SCHEMA_VERSION,
            "accepted": {"ref": revision, "revision": revision_sha},
            "component": component_id,
            "affected": sorted(graph),
            "checks_with": [],
            "domain_ancestors": [],
            "fallback": "full",
            "unverified": unverified
            + [f"unknown component {component_id!r} at {revision_sha}; scope widened"],
        }
    dependents: dict[str, set[str]] = {key: set() for key in graph}
    for component in graph.values():
        for dependency in component.get("requires", []):
            dependents.setdefault(dependency, set()).add(component["id"])
    affected = {component_id}
    queue = [component_id]
    while queue:
        current = queue.pop()
        for consumer in dependents.get(current, set()):
            if consumer not in affected:
                affected.add(consumer)
                queue.append(consumer)
    domains = set()
    for item in list(affected):
        parent = graph[item].get("parent")
        while parent:
            if parent not in affected:
                domains.add(parent)
            parent = graph[parent].get("parent") if parent in graph else None
    co_validation: set[str] = set()
    while True:
        additions = {
            partner
            for item in affected
            for partner in graph[item].get("checks_with", [])
            if partner not in affected
        }
        if not additions:
            break
        affected |= additions
        co_validation |= additions
    fallback = None
    if unverified:
        affected = set(graph)
        domains = set()
        co_validation = set()
        fallback = "full"
    return {
        "schema": SCHEMA,
        "schema_version": SCHEMA_VERSION,
        "accepted": {"ref": revision, "revision": revision_sha},
        "component": component_id,
        "affected": sorted(affected),
        "checks_with": sorted(co_validation),
        "domain_ancestors": sorted(domains),
        "fallback": fallback,
        "unverified": unverified,
    }


def analyze(accepted_ref: str, candidate_ref: str) -> dict:
    accepted_revision = resolve(ROOT, accepted_ref)
    candidate_revision = resolve(ROOT, candidate_ref)
    accepted, accepted_unverified = load_graph(ROOT, accepted_revision, "root")
    candidate, candidate_unverified = load_graph(ROOT, candidate_revision, "root")
    changed = changed_components(accepted, candidate)
    affected, domains, co_validation = affected_set(accepted, candidate, changed)
    unverified = accepted_unverified + candidate_unverified
    fallback = None
    if unverified:
        # Undetermined scope must widen, never silently skip.
        affected = sorted(set(accepted) | set(candidate))
        domains = []
        co_validation = []
        fallback = "full"
    return {
        "schema": SCHEMA,
        "schema_version": SCHEMA_VERSION,
        "accepted": {"ref": accepted_ref, "revision": accepted_revision},
        "candidate": {"ref": candidate_ref, "revision": candidate_revision},
        "counts": {
            "accepted_components": len(accepted),
            "candidate_components": len(candidate),
            "changed": len(changed),
            "affected": len(affected),
        },
        "changed": changed,
        "affected": affected,
        "domain_ancestors": domains,
        "checks_with": co_validation,
        "fallback": fallback,
        "unverified": unverified,
    }


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--accepted", required=True, help="accepted root revision")
    parser.add_argument("--candidate", default="HEAD", help="candidate root revision")
    parser.add_argument("--out", default=None)
    parser.add_argument("--json", action="store_true")
    parser.add_argument(
        "--component",
        default=None,
        help="report consumers of one component at --accepted instead of a root diff",
    )
    parser.add_argument(
        "--fail-on-unverified",
        action="store_true",
        help="exit 2 when a domain cannot be expanded",
    )
    args = parser.parse_args(argv)
    try:
        if args.component:
            report = consumers_of(args.component, args.accepted)
        else:
            report = analyze(args.accepted, args.candidate)
    except ImpactError as error:
        print(f"impact: error: {error}", file=sys.stderr)
        return 1
    text = json.dumps(report, ensure_ascii=False, indent=2) + "\n"
    if args.out:
        Path(args.out).write_text(text)
    if args.json:
        print(text, end="")
    else:
        if "candidate" in report:
            print(
                f"accepted {report['accepted']['revision'][:12]} "
                f"candidate {report['candidate']['revision'][:12]}"
            )
            print(
                f"changed={report['counts']['changed']} "
                f"affected={report['counts']['affected']}"
            )
            for item in report["changed"]:
                print(f"  changed {item['id']}: {item['reason']}")
        else:
            print(
                f"component {report['component']} at "
                f"{report['accepted']['revision'][:12]}"
            )
            print(f"consumers={len(report['affected'])}")
        print("affected: " + ", ".join(report["affected"]))
        if report["checks_with"]:
            print("co-validation: " + ", ".join(report["checks_with"]))
        if report["fallback"]:
            print("fallback: full (impact scope undetermined)")
        for note in report["unverified"]:
            print(f"UNVERIFIED: {note}", file=sys.stderr)
    if args.fail_on_unverified and report["unverified"]:
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
