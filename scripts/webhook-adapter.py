#!/usr/bin/env python3
"""Trusted adapter for candidate webhooks and release/candidate reconciliation.

This turns an authenticated Gitea webhook (push / pull_request) into the
normalized candidate notification verified by `scripts/candidate-notify.py`, and
provides the periodic reconciliation path required when a fast notification is
lost. It is deliberately small: no long-running service, no merge/push rights,
no trust in payload asset URLs or commands.

Trust rules:

* the raw body is authenticated with the repository webhook secret
  (`X-Gitea-Signature`, HMAC-SHA256 hex) — an unsigned or wrong-signature body
  never reaches verification;
* the repository must be an allowlisted component remote (or the root
  repository) discovered from the accepted root graph, never from the payload;
* the candidate base is the SHA currently pinned by the accepted graph (payload
  branch/base is only a hint);
* the commit must be reachable in the owning local repository;
* CI aggregation requires at least one status and only `success`; the URL must
  be an https URL on an allowlisted CI host;
* replay/duplicate/revalidation is handled by the shared candidate state file;
* reconciliation scans open PRs in allowlisted repositories, so a lost webhook
  is recovered by the next successful run (idempotent with the same state).

The adapter outputs a plan only (`auto_promote=false`, `auto_merge=false`);
a human still reviews candidate branches/PRs.
"""

from __future__ import annotations

import argparse
import hashlib
import hmac
import importlib.util
import json
import random
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parents[1]
ROOT = PROJECT_ROOT
DEFAULT_CI_HOST = "ci.cflmy.cn"
DEFAULT_ROOT_REPOSITORY = "cflmy/marqdo"


class AdapterError(Exception):
    pass


def load_notify():
    spec = importlib.util.spec_from_file_location(
        "marqdo_candidate_notify", PROJECT_ROOT / "scripts" / "candidate-notify.py"
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    module.ROOT = ROOT
    return module


def resolve_revision(notify, ref: str) -> str:
    impact = notify.load_impact()
    impact.ROOT = ROOT
    return impact.resolve(ROOT, ref)


def verify_signature(raw: bytes, signature: str | None, secret: str) -> None:
    if not secret:
        raise AdapterError("webhook secret is required")
    if not signature:
        raise AdapterError("missing X-Gitea-Signature")
    expected = hmac.new(secret.encode(), raw, hashlib.sha256).hexdigest()
    if not hmac.compare_digest(expected, signature.strip().lower()):
        raise AdapterError("webhook signature does not match the raw body")


def parse_event(raw: bytes, event: str, signature: str | None, secret: str) -> dict:
    verify_signature(raw, signature, secret)
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError as error:
        raise AdapterError(f"webhook body is not JSON: {error}") from error
    if not isinstance(payload, dict):
        raise AdapterError("webhook body must be a JSON object")
    repository = (payload.get("repository") or {}).get("full_name")
    if not isinstance(repository, str) or not repository:
        raise AdapterError("webhook payload has no repository.full_name")
    if event == "push":
        if payload.get("deleted"):
            raise AdapterError("branch deletion is not a candidate")
        before = payload.get("before")
        after = payload.get("after")
        if not isinstance(before, str) or not isinstance(after, str):
            raise AdapterError("push payload has no before/after commit")
        if set(after) == {"0"}:
            raise AdapterError("push payload deletes the branch")
        return {
            "kind": "push",
            "repository": repository,
            "head_sha": after.lower(),
            "payload_base": before.lower(),
            "ref": payload.get("ref"),
        }
    if event == "pull_request":
        action = payload.get("action")
        if action not in ("opened", "synchronized", "reopened"):
            raise AdapterError(f"pull_request action {action!r} is not a new candidate")
        pull = payload.get("pull_request") or {}
        head = (pull.get("head") or {}).get("sha")
        base = (pull.get("base") or {}).get("sha")
        if not isinstance(head, str) or not isinstance(base, str):
            raise AdapterError("pull_request payload has no head/base SHA")
        return {
            "kind": "pull_request",
            "repository": repository,
            "head_sha": head.lower(),
            "payload_base": base.lower(),
            "number": pull.get("number"),
            "action": action,
        }
    raise AdapterError(f"unsupported webhook event {event!r}; expected push or pull_request")


def aggregate_ci(statuses: list[dict], ci_hosts: list[str]) -> dict:
    if not statuses:
        return {"status": "pending", "url": None}
    states = [str(item.get("status", "pending")) for item in statuses]
    url = None
    for item in statuses:
        candidate = item.get("target_url") or ""
        if isinstance(candidate, str) and candidate.startswith("https://"):
            url = candidate
    if any(state == "failure" or state == "error" for state in states):
        return {"status": "failure", "url": url}
    if all(state == "success" for state in states):
        return {"status": "success", "url": url}
    return {"status": "pending", "url": url}


def build_notification(
    event: dict, ci: dict, accepted_revision: str, root_repository: str
) -> dict:
    notify = load_notify()
    if event["repository"] == root_repository:
        kind = "root"
        component = None
        base_sha = accepted_revision
    else:
        allowlist = notify.repository_allowlist(accepted_revision)
        component_record = allowlist.get(event["repository"])
        if component_record is None:
            raise AdapterError(
                f"unknown repository {event['repository']!r}; allowlist: {sorted(allowlist)}"
            )
        kind = "component"
        component = component_record["id"]
        base_sha = component_record.get("sha")
        if not isinstance(base_sha, str) or not base_sha:
            raise AdapterError(
                f"accepted graph has no pinned SHA for {component}; cannot verify a candidate"
            )
    notification = {
        "schema": "marqdo.candidate-notification",
        "schema_version": 1,
        "repository": event["repository"],
        "kind": kind,
        "head_sha": event["head_sha"],
        "base_sha": base_sha,
        "ci": ci,
    }
    if component is not None:
        notification["component"] = component
    if event.get("number") is not None:
        notification["pull_request"] = event["number"]
    return notification


def verify_webhook(
    raw: bytes,
    event_name: str,
    signature: str | None,
    secret: str,
    statuses: list[dict],
    accepted: str,
    state_path: Path,
    ci_hosts: list[str] | None = None,
    root_repository: str = DEFAULT_ROOT_REPOSITORY,
    write_state: bool = True,
) -> dict:
    event = parse_event(raw, event_name, signature, secret)
    notify = load_notify()
    accepted_revision = resolve_revision(notify, accepted)
    ci = aggregate_ci(statuses, ci_hosts or [DEFAULT_CI_HOST])
    notification = build_notification(event, ci, accepted_revision, root_repository)
    try:
        return notify.verify(
            notification,
            accepted,
            state_path,
            ci_hosts or [DEFAULT_CI_HOST],
            root_repository,
            write_state=write_state,
        )
    except notify.NotificationError as error:
        return {"verified": False, "reason": str(error)}


def http_json(url: str, token: str, attempts: int = 3) -> object:
    last: Exception | None = None
    for attempt in range(attempts):
        request = urllib.request.Request(
            url,
            headers={"Authorization": f"token {token}", "Accept": "application/json"},
        )
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                return json.loads(response.read())
        except urllib.error.HTTPError as error:
            if error.code in (401, 403):
                raise AdapterError(f"authorization rejected for {url}: HTTP {error.code}") from error
            last = error
        except (urllib.error.URLError, TimeoutError) as error:
            last = error
        if attempt + 1 < attempts:
            time.sleep(min(2 ** attempt, 4) + random.random() * 0.2)
    raise AdapterError(f"cannot read {url} after {attempts} attempts: {last}")


def fetch_statuses(api_base: str, repository: str, sha: str, token: str) -> list[dict]:
    data = http_json(
        f"{api_base}/repos/{repository}/commits/{sha}/statuses", token
    )
    return data if isinstance(data, list) else []


def list_open_prs(api_base: str, repository: str, token: str) -> list[dict]:
    data = http_json(
        f"{api_base}/repos/{repository}/pulls?state=open&limit=50", token
    )
    return data if isinstance(data, list) else []


def reconcile(
    accepted: str,
    state_path: Path,
    repositories: list[str],
    accepted_revision: str,
    statuses_fetcher,
    prs_fetcher,
    ci_hosts: list[str] | None = None,
    root_repository: str = DEFAULT_ROOT_REPOSITORY,
) -> dict:
    notify = load_notify()
    ci_hosts = ci_hosts or [DEFAULT_CI_HOST]
    results = []
    for repository in sorted(set(repositories)):
        try:
            pulls = prs_fetcher(repository)
        except AdapterError as error:
            results.append({"repository": repository, "status": "ERROR", "reason": str(error)})
            continue
        for pull in pulls:
            head = ((pull.get("head") or {}).get("sha")) or ""
            number = pull.get("number")
            if len(head) != 40:
                results.append(
                    {"repository": repository, "status": "ERROR", "reason": "PR has no head SHA"}
                )
                continue
            try:
                statuses = statuses_fetcher(repository, head)
                ci = aggregate_ci(statuses, ci_hosts)
                event = {
                    "kind": "pull_request",
                    "repository": repository,
                    "head_sha": head,
                    "payload_base": ((pull.get("base") or {}).get("sha")) or "",
                    "number": number,
                }
                notification = build_notification(
                    event, ci, accepted_revision, root_repository
                )
                plan = notify.verify(
                    notification,
                    accepted,
                    state_path,
                    ci_hosts,
                    root_repository,
                    write_state=True,
                )
                results.append(
                    {
                        "repository": repository,
                        "pull_request": number,
                        "head_sha": head,
                        "status": "VERIFIED",
                        "revalidate": plan["revalidate"],
                        "affected": plan["impact"]["affected"],
                    }
                )
            except notify.NotificationError as error:
                message = str(error)
                status = "DUPLICATE" if "duplicate/replay" in message else "REJECTED"
                results.append(
                    {
                        "repository": repository,
                        "pull_request": number,
                        "head_sha": head,
                        "status": status,
                        "reason": message,
                    }
                )
    return {
        "schema": "marqdo.candidate-reconcile",
        "schema_version": 1,
        "accepted": {"ref": accepted, "revision": accepted_revision},
        "repositories": sorted(set(repositories)),
        "results": results,
        "verified": sum(1 for item in results if item["status"] == "VERIFIED"),
        "pending": sum(
            1 for item in results if item["status"] in ("DUPLICATE", "REJECTED", "ERROR")
        ),
    }


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--webhook", help="raw webhook body file")
    group.add_argument("--reconcile", action="store_true", help="recover lost notifications")
    parser.add_argument("--event", default="push", help="X-Gitea-Event value")
    parser.add_argument("--signature", default=None, help="X-Gitea-Signature value")
    parser.add_argument("--secret-env", default="MARQDO_WEBHOOK_SECRET")
    parser.add_argument("--statuses", default=None, help="JSON array of commit statuses")
    parser.add_argument("--accepted", required=True)
    parser.add_argument("--state", required=True)
    parser.add_argument("--ci-host", action="append", default=None)
    parser.add_argument("--root-repository", default=DEFAULT_ROOT_REPOSITORY)
    parser.add_argument("--api-base", default="https://git.cflmy.cn/api/v1")
    parser.add_argument("--token-env", default="MARQDO_GIT_TOKEN")
    parser.add_argument("--out", default=None)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args(argv)
    ci_hosts = args.ci_host or [DEFAULT_CI_HOST]
    try:
        if args.webhook:
            secret = __import__("os").environ.get(args.secret_env, "")
            raw = Path(args.webhook).read_bytes()
            statuses = []
            if args.statuses:
                statuses = json.loads(Path(args.statuses).read_text())
            plan = verify_webhook(
                raw,
                args.event,
                args.signature,
                secret,
                statuses,
                args.accepted,
                Path(args.state),
                ci_hosts,
                args.root_repository,
            )
        else:
            import os

            token = os.environ.get(args.token_env, "")
            if not token:
                raise AdapterError(f"{args.token_env} is required for reconciliation")
            notify = load_notify()
            accepted_revision = resolve_revision(notify, args.accepted)
            repositories = sorted(notify.repository_allowlist(accepted_revision)) + [
                args.root_repository
            ]
            plan = reconcile(
                args.accepted,
                Path(args.state),
                repositories,
                accepted_revision,
                lambda repository, sha: fetch_statuses(
                    args.api_base, repository, sha, token
                ),
                lambda repository: list_open_prs(args.api_base, repository, token),
                ci_hosts,
                args.root_repository,
            )
    except (AdapterError, json.JSONDecodeError, OSError) as error:
        print(f"webhook-adapter: rejected: {error}", file=sys.stderr)
        return 1
    text = json.dumps(plan, ensure_ascii=False, indent=2) + "\n"
    if args.out:
        Path(args.out).write_text(text)
    if args.json:
        print(text, end="")
    else:
        print(
            f"{plan.get('schema')}: verified={plan.get('verified', 0)} "
            f"pending={plan.get('pending', 0)}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
