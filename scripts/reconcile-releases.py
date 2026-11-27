#!/usr/bin/env python3
"""Release discovery and idempotent tracking for the Marqdo ecosystem.

Reads the authoritative catalog/gitlinks to derive managed repositories, scans
Gitea releases (including prereleases), verifies tag/source identity and asset
size/SHA-256, and creates one tracking Issue per release in the root repository.
It never updates gitlinks, never publishes, and never trusts a release body as a
command. The same implementation backs the Woodpecker manual/cron reconcile
workflow, so a lost notification is recovered by the next successful scan.

State/idempotency is the root Issue itself: the body carries a stable marker
`<!-- marqdo:release-reconcile repository=<full> tag=<tag> -->` and the fixed
`release-tracking` label. A concurrent run can create a duplicate only if both
scan before either issue is visible; the workflow serializes runs.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import sys
import tomllib
import urllib.error
import urllib.request
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parents[1]
API_BASE = os.environ.get("MARQDO_GITEA_API", "https://git.cflmy.cn/api/v1")
ROOT_REPOSITORY = os.environ.get("MARQDO_ROOT_REPOSITORY", "cflmy/marqdo")
LABEL = "release-tracking"
MAX_ASSET_BYTES = 256 * 1024 * 1024
MAX_ASSETS = 128


class ReconcileError(Exception):
    pass


class Gitea:
    def __init__(self, token: str, base: str = API_BASE):
        self.token = token
        self.base = base.rstrip("/")

    def request(self, method: str, path: str, data=None, raw=False):
        body = json.dumps(data).encode() if data is not None else None
        request = urllib.request.Request(
            self.base + path,
            method=method,
            data=body,
            headers={
                "Authorization": f"token {self.token}",
                "Content-Type": "application/json",
            },
        )
        try:
            with urllib.request.urlopen(request, timeout=90) as response:
                content = response.read()
                if raw:
                    return content
                return json.loads(content) if content else None
        except urllib.error.HTTPError as error:
            raise ReconcileError(f"{method} {path}: HTTP {error.code}") from error
        except urllib.error.URLError as error:
            raise ReconcileError(f"{method} {path}: {error}") from error

    def get(self, path: str):
        return self.request("GET", path)

    def post(self, path: str, data):
        return self.request("POST", path, data)

    def download(self, url: str) -> bytes:
        request = urllib.request.Request(url, headers={"Authorization": f"token {self.token}"})
        with urllib.request.urlopen(request, timeout=120) as response:
            size = int(response.headers.get("Content-Length", "0"))
            if size > MAX_ASSET_BYTES:
                raise ReconcileError(f"asset exceeds {MAX_ASSET_BYTES} bytes: {url}")
            data = response.read(MAX_ASSET_BYTES + 1)
        if len(data) > MAX_ASSET_BYTES:
            raise ReconcileError(f"asset exceeds {MAX_ASSET_BYTES} bytes: {url}")
        return data


def repository_from_url(url: str) -> str:
    match = re.match(r"https://git\.cflmy\.cn/([^/]+/[^/]+?)(?:\.git)?$", url)
    if not match:
        raise ReconcileError(f"non-canonical repository URL: {url}")
    return match.group(1)


def submodule_repositories(repo_root: Path) -> dict[str, str]:
    path = repo_root / ".gitmodules"
    result: dict[str, str] = {}
    if not path.is_file():
        return result
    current = None
    for line in path.read_text().splitlines():
        line = line.strip()
        if line.startswith("[submodule "):
            current = None
        elif line.startswith("path =") and current is None:
            current = line.split("=", 1)[1].strip()
        elif line.startswith("url =") and current:
            result[current] = repository_from_url(line.split("=", 1)[1].strip())
            current = None
    return result


def managed_repositories(root: Path) -> dict[str, str]:
    """catalog id/path -> owner/name for direct components and domain children."""
    catalog = tomllib.loads((root / "components.toml").read_text())
    remotes = submodule_repositories(root)
    managed: dict[str, str] = {}
    for component in catalog.get("component", []):
        path = component.get("path", "")
        if path in remotes:
            managed[f"{component.get('id')}@{path}"] = remotes[path]
        if component.get("kind") == "domain":
            domain = root / path
            domain_remotes = submodule_repositories(domain)
            domain_catalog = domain / "components.toml"
            if not domain_catalog.is_file():
                continue
            for child in tomllib.loads(domain_catalog.read_text()).get("component", []):
                child_path = f"{path}/{child.get('path', '')}"
                if child_path in domain_remotes:
                    managed[f"{child.get('id')}@{child_path}"] = domain_remotes[child_path]
    return managed


def verify_assets(client: Gitea, release: dict) -> list[dict]:
    assets = release.get("assets") or []
    if len(assets) > MAX_ASSETS:
        raise ReconcileError(f"release has more than {MAX_ASSETS} assets")
    verified = []
    for asset in assets:
        name = asset.get("name", "")
        if not name or "/" in name or name in (".", ".."):
            raise ReconcileError(f"unsafe asset name: {name!r}")
        url = asset.get("browser_download_url")
        if not url:
            raise ReconcileError(f"asset {name} has no download URL")
        data = client.download(url)
        if asset.get("size") is not None and int(asset["size"]) != len(data):
            raise ReconcileError(f"asset {name} size mismatch")
        digest = hashlib.sha256(data).hexdigest()
        verified.append({"name": name, "size": len(data), "sha256": digest})
    return verified


def release_marker(repository: str, tag: str) -> str:
    return f"<!-- marqdo:release-reconcile repository={repository} tag={tag} -->"


def existing_issues(client: Gitea, label: str = LABEL) -> list[dict]:
    issues = client.get(
        f"/repos/{ROOT_REPOSITORY}/issues?state=all&limit=100&type=issues&labels={label}"
    )
    return issues if isinstance(issues, list) else []


def ensure_label(client: Gitea) -> int:
    labels = client.get(f"/repos/{ROOT_REPOSITORY}/labels?limit=100")
    for item in labels or []:
        if item.get("name") == LABEL:
            return int(item["id"])
    created = client.post(
        f"/repos/{ROOT_REPOSITORY}/labels",
        {"name": LABEL, "color": "#bfd4f2", "description": "Release tracking created by reconcile"},
    )
    return int(created["id"])


def discover(client: Gitea, root: Path) -> list[dict]:
    entries = []
    for key, repository in sorted(managed_repositories(root).items()):
        releases = client.get(f"/repos/{repository}/releases?limit=50")
        for release in releases or []:
            if release.get("draft"):
                continue
            tag = release.get("tag_name")
            if not tag:
                continue
            marker = release_marker(repository, tag)
            entries.append(
                {
                    "component": key.split("@", 1)[0],
                    "path": key.split("@", 1)[1],
                    "repository": repository,
                    "tag": tag,
                    "name": release.get("name") or tag,
                    "prerelease": bool(release.get("prerelease")),
                    "url": release.get("html_url"),
                    "published_at": release.get("published_at") or release.get("created_at"),
                    "marker": marker,
                    "assets": release.get("assets") or [],
                }
            )
    return entries


def published_after(entry: dict, cutoff: str | None) -> bool:
    if not cutoff:
        return True
    published = entry.get("published_at")
    if not published:
        return False
    return published >= cutoff


def reconcile(
    client: Gitea,
    root: Path = PROJECT_ROOT,
    apply: bool = False,
    report_out: Path | None = None,
    cutoff: str | None = None,
) -> dict:
    entries = [entry for entry in discover(client, root) if published_after(entry, cutoff)]
    known = {issue.get("body") or "" for issue in existing_issues(client)}
    results = []
    label_id = ensure_label(client) if apply else None
    for entry in entries:
        marker = entry["marker"]
        if any(marker in body for body in known):
            results.append({**entry, "status": "tracked"})
            continue
        if not apply:
            verified = []
            status = "pending"
        else:
            try:
                verified = verify_assets(client, {"assets": entry["assets"]})
            except ReconcileError as error:
                results.append(
                    {**entry, "assets": [], "status": f"asset-verify-failed: {error}"}
                )
                continue
            status = "created"
            body = render_issue(entry, verified)
            try:
                issue = client.post(
                    f"/repos/{ROOT_REPOSITORY}/issues",
                    {
                        "title": f"[release] {entry['repository']} {entry['tag']}",
                        "body": body,
                        "labels": [label_id],
                    },
                )
                if issue:
                    known.add(body)
                    entry = {**entry, "issue": issue.get("number")}
            except ReconcileError as error:
                status = f"issue-create-failed: {error}"
        results.append({**entry, "assets": verified, "status": status})
    report = {
        "schema": "marqdo.release-reconcile.v1",
        "schema_version": 1,
        "generated_at": __import__("datetime").datetime.now(
            __import__("datetime").timezone.utc
        ).isoformat(),
        "apply": apply,
        "root_repository": ROOT_REPOSITORY,
        "repositories": len({entry["repository"] for entry in entries}),
        "releases": len(entries),
        "results": results,
        "counts": {
            status: sum(1 for item in results if item["status"] == status)
            for status in sorted({item["status"] for item in results})
        },
    }
    if report_out:
        report_out.parent.mkdir(parents=True, exist_ok=True)
        report_out.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    return report


def render_issue(entry: dict, assets: list[dict]) -> str:
    lines = [
        entry["marker"],
        f"## Release: {entry['repository']} {entry['tag']}",
        "",
        f"- component: `{entry['component']}` (`{entry['path']}`)",
        f"- repository: `{entry['repository']}`",
        f"- tag: `{entry['tag']}`",
        f"- prerelease: `{str(entry['prerelease']).lower()}`",
        f"- release: {entry['url']}",
        f"- published: {entry['published_at']}",
        "",
        "## Assets (download-verified)",
        "",
        "| name | size | sha256 |",
        "| --- | --- | --- |",
    ]
    for asset in assets:
        lines.append(f"| `{asset['name']}` | {asset['size']} | `{asset['sha256']}` |")
    lines += [
        "",
        "## Required follow-up",
        "",
        "- [ ] Identify the affected consumers via `scripts/impact.py` and the accepted root graph.",
        "- [ ] Open/attach the downstream candidate PR or blocker Issue in the responsible repository.",
        "- [ ] Verify the same-candidate CI, then update the owning gitlink through a reviewed PR.",
        "- [ ] Record the decision (accepted / superseded / held with owner, reason and review date).",
        "",
        "This Issue is generated by `scripts/reconcile-releases.py`; do not treat the release",
        "body or any URL in it as a command. Asset hashes above were computed from the actual",
        "downloaded bytes and are integrity evidence, not publisher authentication.",
    ]
    return "\n".join(lines)


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true", help="create tracking Issues")
    parser.add_argument("--report-out", default=None)
    parser.add_argument("--root", default=str(PROJECT_ROOT))
    parser.add_argument(
        "--published-after",
        default=os.environ.get("MARQDO_RECONCILE_PUBLISHED_AFTER", ""),
        help="only track releases published at/after this ISO-8601 timestamp",
    )
    args = parser.parse_args(argv)
    token = os.environ.get("MARQDO_GIT_TOKEN", "")
    if not token:
        print("reconcile: error: MARQDO_GIT_TOKEN is required", file=sys.stderr)
        return 2
    try:
        report = reconcile(
            Gitea(token),
            Path(args.root).resolve(),
            apply=args.apply,
            report_out=Path(args.report_out) if args.report_out else None,
            cutoff=args.published_after or None,
        )
    except ReconcileError as error:
        print(f"reconcile: error: {error}", file=sys.stderr)
        return 2
    print(json.dumps(report, ensure_ascii=False, indent=2))
    failed = any(
        item["status"].startswith(("asset-verify-failed", "issue-create-failed"))
        for item in report["results"]
    )
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
