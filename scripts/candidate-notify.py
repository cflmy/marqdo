#!/usr/bin/env python3
"""Verify one candidate notification before any automation acts on it.

The ecosystem uses manual PR gates; this verifier is the authorization step for
a (future) bot that may only create candidate branches/PRs. It accepts a
normalized notification (from a reviewed webhook adapter), never a raw remote
payload, and returns a machine plan. It refuses forged, replayed, stale,
unknown or failed candidates instead of trusting the sender.

Checks, in order:

1. notification schema and 40-hex commits;
2. the repository is an allowlisted component remote (or the root repository);
3. the CI result is `success` on an allowlisted CI host with an https URL;
4. the commit is reachable in the owning local repository;
5. a component candidate starts from the exact SHA currently pinned by the
   accepted root graph (a stale/retargeted base is rejected);
6. the (repository, head SHA) pair is not a replay; a new accepted baseline
   allows a re-validation and marks the result `revalidate`;
7. impact is computed with the shared union-graph analyzer; undetermined scope
   widens to full re-validation instead of skipping.

The verifier never merges, pushes, grants permissions or fetches floating
branches. Its only proposed action is recording the candidate / creating a
candidate branch that a human still reviews.
"""

from __future__ import annotations

import argparse
import datetime as dt
import importlib.util
import json
import subprocess
import sys
import tomllib
import urllib.parse
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parents[1]
ROOT = PROJECT_ROOT
SCHEMA = "marqdo.candidate-notification"
PLAN_SCHEMA = "marqdo.candidate-plan"
SCHEMA_VERSION = 1
DEFAULT_CI_HOST = "ci.cflmy.cn"


def load_impact():
    """Load the shared analyzer and point it at the verifier's root checkout."""
    spec = importlib.util.spec_from_file_location(
        "marqdo_impact", PROJECT_ROOT / "scripts" / "impact.py"
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    module.ROOT = ROOT
    return module


def git(repo: Path, *args: str) -> str:
    result = subprocess.run(["git", "-C", str(repo), *args], capture_output=True)
    if result.returncode != 0:
        raise NotificationError(
            f"git {' '.join(args)} failed in {repo}: "
            f"{result.stderr.decode(errors='replace').strip()}"
        )
    return result.stdout.decode(errors="replace")


class NotificationError(Exception):
    pass


def is_sha(value) -> bool:
    return isinstance(value, str) and len(value) == 40 and all(
        ch in "0123456789abcdef" for ch in value
    )


def repository_allowlist(accepted_revision: str) -> dict[str, dict]:
    """Map `owner/name` -> component identity from the accepted root graph."""
    impact = load_impact()
    graph, unverified = impact.load_graph(ROOT, accepted_revision, "root")
    if unverified:
        raise NotificationError(
            "accepted graph cannot be expanded: " + "; ".join(unverified)
        )
    allowlist: dict[str, dict] = {}
    for component in graph.values():
        remote = component.get("remote")
        if not remote:
            continue
        parsed = urllib.parse.urlparse(remote)
        name = parsed.path.strip("/")
        if name.endswith(".git"):
            name = name[:-4]
        if name:
            allowlist[name] = component
    return allowlist


def check_schema(notification: dict) -> None:
    if notification.get("schema") != SCHEMA:
        raise NotificationError(f"unsupported notification schema: {notification.get('schema')!r}")
    if notification.get("schema_version") != SCHEMA_VERSION:
        raise NotificationError("unsupported notification schema_version")
    if notification.get("kind") not in ("component", "root"):
        raise NotificationError("kind must be 'component' or 'root'")
    for field in ("repository", "head_sha", "base_sha"):
        if not isinstance(notification.get(field), str) or not notification[field]:
            raise NotificationError(f"notification has no {field}")
    if not is_sha(notification["head_sha"]) or not is_sha(notification["base_sha"]):
        raise NotificationError("head_sha/base_sha must be full lowercase commit SHAs")
    if notification["head_sha"] == notification["base_sha"]:
        raise NotificationError("head_sha equals base_sha; nothing to verify")


def check_ci(notification: dict, ci_hosts: list[str]) -> dict:
    ci = notification.get("ci")
    if not isinstance(ci, dict):
        raise NotificationError("notification has no ci object")
    if ci.get("status") != "success":
        raise NotificationError(f"CI status is not success: {ci.get('status')!r}")
    url = ci.get("url")
    if not isinstance(url, str):
        raise NotificationError("notification ci.url must be a string")
    parsed = urllib.parse.urlparse(url)
    if parsed.scheme != "https" or parsed.hostname not in ci_hosts:
        raise NotificationError(
            f"CI URL must be https on one of {ci_hosts}: {url!r}"
        )
    return ci


def load_state(path: Path) -> dict:
    if not path.is_file():
        return {"schema": "marqdo.candidate-state", "schema_version": 1, "seen": {}}
    state = json.loads(path.read_text())
    if not isinstance(state.get("seen"), dict):
        raise NotificationError(f"invalid candidate state file: {path}")
    return state


def verify(
    notification: dict,
    accepted_ref: str,
    state_path: Path,
    ci_hosts: list[str],
    root_repository: str,
    write_state: bool,
) -> dict:
    check_schema(notification)
    ci = check_ci(notification, ci_hosts)
    impact = load_impact()
    impact.ROOT = ROOT
    accepted_revision = impact.resolve(ROOT, accepted_ref)
    reasons: list[str] = []
    revalidate = False
    kind = notification["kind"]
    repository = notification["repository"]
    allowlist = repository_allowlist(accepted_revision)
    if kind == "root":
        if repository != root_repository:
            raise NotificationError(
                f"unknown root repository {repository!r}; allowlist is {root_repository!r}"
            )
        if not subprocess.run(
            ["git", "-C", str(ROOT), "cat-file", "-e", notification["head_sha"] + "^{commit}"],
            capture_output=True,
        ).returncode == 0:
            raise NotificationError("candidate commit is not reachable in the root repository")
        report = impact.analyze(accepted_revision, notification["head_sha"])
        base_sha = accepted_revision
        key = f"root@{notification['head_sha']}"
    else:
        component = allowlist.get(repository)
        if component is None:
            raise NotificationError(
                f"unknown repository {repository!r}; allowlist: {sorted(allowlist)}"
            )
        component_id = notification.get("component") or component["id"]
        if component_id != component["id"]:
            raise NotificationError(
                f"notification component {component_id!r} does not own repository {repository!r}"
            )
        checkout = ROOT / component["path"]
        if not (checkout / ".git").exists():
            raise NotificationError(f"missing component checkout: {checkout}")
        if not subprocess.run(
            ["git", "-C", str(checkout), "cat-file", "-e", notification["head_sha"] + "^{commit}"],
            capture_output=True,
        ).returncode == 0:
            raise NotificationError("candidate commit is not reachable in the component repository")
        if component.get("sha") != notification["base_sha"]:
            raise NotificationError(
                "base_sha is stale: the accepted root graph pins "
                f"{component.get('sha')} for {component_id}"
            )
        report = impact.consumers_of(component_id, accepted_revision)
        base_sha = component["sha"]
        key = f"{repository}@{notification['head_sha']}"
    state = load_state(state_path)
    previous = state["seen"].get(key)
    if previous is not None:
        if previous.get("accepted") == accepted_revision:
            raise NotificationError(
                "duplicate/replay: this candidate was already verified against the same accepted baseline"
            )
        revalidate = True
        reasons.append(
            "new accepted baseline since first notification; re-run the full candidate gates"
        )
    if report.get("unverified"):
        reasons.append("impact scope undetermined; full re-validation required")
    state["seen"][key] = {
        "accepted": accepted_revision,
        "base_sha": base_sha,
        "kind": kind,
        "repository": repository,
        "first_verified": previous.get("first_verified")
        if previous
        else dt.datetime.now(dt.timezone.utc).isoformat(),
        "last_verified": dt.datetime.now(dt.timezone.utc).isoformat(),
    }
    if write_state:
        state_path.parent.mkdir(parents=True, exist_ok=True)
        state_path.write_text(json.dumps(state, indent=2) + "\n")
    return {
        "schema": PLAN_SCHEMA,
        "schema_version": SCHEMA_VERSION,
        "verified": True,
        "revalidate": revalidate,
        "reasons": reasons,
        "kind": kind,
        "repository": repository,
        "head_sha": notification["head_sha"],
        "base_sha": notification["base_sha"],
        "accepted": {"ref": accepted_ref, "revision": accepted_revision},
        "ci": ci,
        "impact": {
            "affected": report.get("affected", []),
            "changed": report.get("changed", []),
            "checks_with": report.get("checks_with", []),
            "fallback": report.get("fallback"),
            "unverified": report.get("unverified", []),
        },
        "actions": {
            "auto_promote": False,
            "auto_merge": False,
            "create_candidate_branch": True,
            "requires_human_review": True,
        },
    }


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--notification", required=True, help="normalized notification JSON")
    parser.add_argument("--accepted", required=True, help="accepted root revision")
    parser.add_argument("--state", required=True, help="candidate dedupe state file")
    parser.add_argument(
        "--ci-host", action="append", default=None, help="allowlisted CI host (repeatable)"
    )
    parser.add_argument("--root-repository", default="cflmy/marqdo")
    parser.add_argument("--out", default=None)
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--dry-run", action="store_true", help="do not write the dedupe state")
    args = parser.parse_args(argv)
    ci_hosts = args.ci_host or [DEFAULT_CI_HOST]
    try:
        notification = json.loads(Path(args.notification).read_text())
        if not isinstance(notification, dict):
            raise NotificationError("notification must be a JSON object")
        plan = verify(
            notification,
            args.accepted,
            Path(args.state),
            ci_hosts,
            args.root_repository,
            write_state=not args.dry_run,
        )
    except (NotificationError, json.JSONDecodeError, OSError) as error:
        rejected = {
            "schema": PLAN_SCHEMA,
            "schema_version": SCHEMA_VERSION,
            "verified": False,
            "reasons": [str(error)],
            "actions": {
                "auto_promote": False,
                "auto_merge": False,
                "create_candidate_branch": False,
                "requires_human_review": True,
            },
        }
        text = json.dumps(rejected, ensure_ascii=False, indent=2) + "\n"
        if args.out:
            Path(args.out).write_text(text)
        if args.json:
            print(text, end="")
        print(f"candidate-notify: rejected: {error}", file=sys.stderr)
        return 1
    text = json.dumps(plan, ensure_ascii=False, indent=2) + "\n"
    if args.out:
        Path(args.out).write_text(text)
    if args.json:
        print(text, end="")
    else:
        print(
            f"verified {plan['kind']} candidate {plan['repository']}@{plan['head_sha'][:12]} "
            f"affected={len(plan['impact']['affected'])} revalidate={plan['revalidate']}"
        )
        for reason in plan["reasons"]:
            print(f"  note: {reason}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
