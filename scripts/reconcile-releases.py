#!/usr/bin/env python3
"""Verify published releases and reconcile their tracking Issues (marqdo#10)."""

from __future__ import annotations

import argparse
import datetime
import hashlib
import http.client
import json
import os
import re
import sys
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

import impact

PROJECT_ROOT = Path(__file__).resolve().parents[1]
FORGE = "https://git.cflmy.cn"
CI = "https://ci.cflmy.cn"
ROOT_REPOSITORY = "cflmy/marqdo"
LABEL = "release-tracking"
RECONCILE_ACCOUNT = "marqdo-reconcile"
INDEX = "release-evidence.json"
PAGE_SIZE = 50
MAX_PAGES = 100
MAX_INDEX_BYTES = 1024 * 1024
MAX_ASSET_BYTES = 256 * 1024 * 1024
MAX_ASSETS = 128
MAX_DEPENDENCIES = 256
SHA = re.compile(r"[0-9a-f]{40}")
DIGEST = re.compile(r"[0-9a-f]{64}")
BASENAME = re.compile(r"[A-Za-z0-9][A-Za-z0-9._+-]{0,254}")
TAG = re.compile(r"[A-Za-z0-9][A-Za-z0-9._+-]{0,127}")
REPOSITORY = re.compile(r"https://git\.cflmy\.cn/(cflmy/[A-Za-z0-9-]+)\.git")
HISTORICAL_CLI = ("cflmy/marqdo-cli", "v3.0.0-rc.5",
                  "c44b0f943875d563bb7c170f8ccd3fd56a82360f")


class ReconcileError(Exception):
    pass


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, message, headers, newurl):
        return None


class Gitea:
    def __init__(self, token: str):
        self.token = token
        self.opener = urllib.request.build_opener(NoRedirect())

    def open(self, method, url, data=None, authenticated=False):
        headers = {}
        if authenticated and self.token:
            headers["Authorization"] = f"token {self.token}"
        if data is not None:
            headers["Content-Type"] = "application/json"
        request = urllib.request.Request(url, data=data, headers=headers, method=method)
        return self.opener.open(request, timeout=30)

    def request(self, method, base, path, data=None, authenticated=False):
        try:
            with self.open(method, base + path, data, authenticated) as response:
                payload = response.read(8 * MAX_INDEX_BYTES + 1)
                require(len(payload) <= 8 * MAX_INDEX_BYTES, "API response exceeds size limit")
                return json.loads(payload)
        except urllib.error.HTTPError as error:
            raise ReconcileError(f"{method} {path}: HTTP {error.code}") from error
        except (OSError, ValueError, http.client.HTTPException) as error:
            raise ReconcileError(f"{method} {path}: {error}") from error

    def get(self, path):
        return self.request("GET", FORGE + "/api/v1", path, authenticated=True)

    def post(self, path, data):
        return self.request("POST", FORGE + "/api/v1", path,
                            json.dumps(data).encode(), authenticated=True)

    def ci(self, path):
        return self.request("GET", CI + "/api", path)

    def asset(self, path, size, *, index=False):
        limit = MAX_INDEX_BYTES if index else MAX_ASSET_BYTES
        require(type(size) is int and 0 < size <= limit, "invalid asset size")
        digest = hashlib.sha256()
        payload = bytearray()
        received = 0
        try:
            with self.open("GET", FORGE + path) as response:
                while chunk := response.read(min(1024 * 1024, size - received + 1)):
                    received += len(chunk)
                    require(received <= size, f"asset {path} size mismatch")
                    digest.update(chunk)
                    if index:
                        payload.extend(chunk)
        except urllib.error.HTTPError as error:
            raise ReconcileError(f"GET {path}: HTTP {error.code}") from error
        except (OSError, http.client.HTTPException) as error:
            raise ReconcileError(f"GET {path}: {error}") from error
        require(received == size, f"asset {path} size mismatch")
        return digest.hexdigest(), bytes(payload)


def require(condition, message):
    if not condition:
        raise ReconcileError(message)


def match(pattern, value):
    return isinstance(value, str) and pattern.fullmatch(value) is not None


def exact_fields(value, fields, optional=()):
    return (isinstance(value, dict) and set(fields) <= value.keys()
            and value.keys() <= set(fields) | set(optional))


def unique_object(pairs):
    result = dict(pairs)
    require(len(result) == len(pairs), "release evidence contains duplicate JSON fields")
    return result


def pages(client, path, **params):
    for page in range(1, MAX_PAGES + 1):
        query = urllib.parse.urlencode(dict(params, limit=PAGE_SIZE, page=page))
        batch = client.get(f"{path}?{query}")
        require(isinstance(batch, list) and all(isinstance(item, dict) for item in batch),
                f"{path}: invalid page {page}")
        require(len(batch) <= PAGE_SIZE, f"{path}: invalid page size")
        yield from batch
        if len(batch) < PAGE_SIZE:
            return
    raise ReconcileError(f"{path}: pagination limit reached")


def timestamp(value):
    try:
        result = datetime.datetime.fromisoformat(value)
    except (TypeError, ValueError) as error:
        raise ReconcileError("invalid release timestamp") from error
    require(result.tzinfo is not None, "release timestamp requires a time zone")
    return result.astimezone(datetime.timezone.utc)


def managed_repositories(root):
    try:
        revision = impact.resolve(root, "HEAD")
        graph, unverified = impact.load_graph(root, revision, "root")
    except impact.ImpactError as error:
        raise ReconcileError(str(error)) from error
    require(not unverified, "; ".join(unverified))
    managed = {}
    for cid, item in sorted(graph.items()):
        if not item["sha"]:
            continue
        remote = item["remote"] or ""
        canonical = REPOSITORY.fullmatch(remote)
        require(canonical is not None, f"{cid}: non-canonical repository")
        repository = canonical[1]
        require(repository not in managed, f"duplicate repository {repository}")
        path = item["path"]
        if item["parent"]:
            path = f"{graph[item['parent']]['path']}/{path}"
        managed[repository] = {**item, "path": path, "repository": repository}
    return revision, graph, managed


def verify_pipeline(client, repository, source_sha, evidence):
    require(exact_fields(evidence, {"pipeline_number", "url", "source_sha"}),
            "invalid component CI evidence")
    number = evidence["pipeline_number"]
    require(type(number) is int and number > 0 and evidence["source_sha"] == source_sha,
            "release has no component CI evidence for its exact SHA")
    repo = client.ci(f"/repos/lookup/{repository}")
    forge_repo = client.get(f"/repos/{repository}")
    require(isinstance(repo, dict) and repo.get("full_name") == repository
            and type(repo.get("id")) is int and repo["id"] > 0
            and isinstance(forge_repo, dict) and type(forge_repo.get("id")) is int
            and repo.get("forge_remote_id") == str(forge_repo["id"]),
            "CI repository identity mismatch")
    expected_url = f"{CI}/repos/{repo['id']}/pipeline/{number}"
    require(evidence["url"] == expected_url, "component CI URL mismatch")
    pipeline = client.ci(f"/repos/{repo['id']}/pipelines/{number}")
    workflow = "main" if repository == "cflmy/marqdo-cli" else "ci"
    require(isinstance(pipeline, dict) and pipeline.get("number") == number
            and pipeline.get("commit") == source_sha and pipeline.get("status") == "success"
            and pipeline.get("event") in {"push", "pull_request", "manual"}
            and any(w.get("name") == workflow and w.get("state") == "success"
                    and any(step.get("type") == "commands" and step.get("state") == "success"
                            for step in w.get("children", []))
                    for w in pipeline.get("workflows", [])), "component CI did not verify the exact SHA")


def asset_path(repository, tag, asset):
    name = asset.get("name")
    require(match(BASENAME, name), "unsafe release asset name")
    escaped = urllib.parse.quote(name, safe="")
    path = f"/{repository}/releases/download/{urllib.parse.quote(tag, safe='')}/{escaped}"
    require(asset.get("browser_download_url") == FORGE + path,
            f"asset {name} has a non-canonical URL")
    return path


def verify_release(client, repository, component, tag, release, graph):
    identity = client.get(f"/repos/{repository}/tags/{urllib.parse.quote(tag, safe='')}")
    require(isinstance(identity, dict) and identity.get("name") == tag
            and isinstance(identity.get("commit"), dict)
            and match(SHA, identity["commit"].get("sha")), "invalid release tag identity")
    source_sha = identity["commit"]["sha"]
    attachments = release.get("assets")
    require(isinstance(attachments, list) and 0 < len(attachments) <= MAX_ASSETS + 1,
            "missing or excessive release assets")
    assets = {}
    for asset in attachments:
        require(isinstance(asset, dict), "invalid release asset")
        path = asset_path(repository, tag, asset)
        name = asset["name"]
        require(name not in assets, "duplicate release asset")
        assets[name] = (path, asset.get("size"))
    if INDEX not in assets and (repository, tag, source_sha) == HISTORICAL_CLI:
        return {"source_sha": source_sha, "historical": True}

    require(INDEX in assets, "release evidence index missing")
    index_digest, payload = client.asset(*assets.pop(INDEX), index=True)
    try:
        evidence = json.loads(payload, object_pairs_hook=unique_object)
    except (UnicodeDecodeError, ValueError, RecursionError) as error:
        raise ReconcileError("invalid release evidence JSON") from error
    require(exact_fields(evidence, {"repository", "component_id", "tag", "source_sha",
                                    "ci", "assets", "dependencies"}), "invalid release evidence fields")
    require((evidence["repository"], evidence["component_id"], evidence["tag"], evidence["source_sha"])
            == (repository, component["id"], tag, source_sha), "release evidence identity mismatch")
    verify_pipeline(client, repository, source_sha, evidence["ci"])

    dependencies = evidence["dependencies"]
    require(isinstance(dependencies, list) and len(dependencies) <= MAX_DEPENDENCIES,
            "invalid release dependency list")
    seen = set()
    for dependency in dependencies:
        require(exact_fields(dependency, {"id", "source_sha"}, {"asset_sha256"})
                and isinstance(dependency["id"], str) and dependency["id"] in graph
                and dependency["id"] not in seen and dependency["id"] != component["id"]
                and match(SHA, dependency["source_sha"])
                and ("asset_sha256" not in dependency or match(DIGEST, dependency["asset_sha256"])),
                "invalid release dependency identity")
        seen.add(dependency["id"])
    require(set(component["requires"]) <= seen, "release evidence is missing a declared dependency")

    records = evidence["assets"]
    require(isinstance(records, list) and 0 < len(records) == len(assets),
            "release evidence does not cover the published assets")
    for item in records:
        require(exact_fields(item, {"name", "size", "target", "sha256"})
                and match(BASENAME, item["name"]) and item["name"] in assets
                and match(BASENAME, item["target"]) and match(DIGEST, item["sha256"])
                and type(item["size"]) is int, "invalid release asset evidence")
        path, size = assets.pop(item["name"])
        require(size == item["size"], f"asset {item['name']} size mismatch")
        digest, _ = client.asset(path, size)
        require(digest == item["sha256"], f"asset {item['name']} digest mismatch")
    return {"source_sha": source_sha, "index_sha256": index_digest,
            "ci": evidence["ci"], "assets": records, "dependencies": dependencies}


def release_marker(repository, tag):
    return f"<!-- marqdo:release-reconcile repository={repository} tag={tag} -->"


def identity_marker(entry):
    return f"<!-- marqdo:verified-release {entry['source_sha']} {entry['index_sha256']} -->"


def tracking_body(entry):
    lines = [entry["marker"], f"## {entry['repository']} {entry['tag']}",
             "", f"- Component: {entry['component']} ({entry['path']})",
             f"- Published: {entry['published_at']}", f"- Accepted gitlink: {entry['accepted_sha']}",
             f"- Release: {FORGE}/{entry['repository']}/releases/tag/{entry['tag']}"]
    if "error" in entry:
        return "\n".join(lines + ["", "## Blocked", "", f"- {entry['error']}"])
    return "\n".join(lines + ["", identity_marker(entry),
        f"- Source SHA: {entry['source_sha']}", f"- Index SHA-256: {entry['index_sha256']}",
        f"- Component CI: {entry['ci']['url']}",
        "", "## Verified assets", "",
        *(f"- {a['name']} ({a['target']}, {a['size']} bytes): {a['sha256']}" for a in entry["assets"]),
        "", "## Pending consumer validation", "",
        f"- Direct consumers: {', '.join(entry['direct_consumers']) or 'none'}",
        f"- Affected: {', '.join(entry['affected']) or 'none'}",
        f"- Governance owner: {entry['governance_owner']}"])


def is_writer(record):
    return (record.get("user") or {}).get("login") == RECONCILE_ACCOUNT


def tracking_comments(client, issues_url, tracker):
    if not is_writer(tracker):
        return []
    comments_url = f"{issues_url}/{tracker['number']}/comments"
    return [comment.get("body") or "" for comment in pages(client, comments_url)
            if is_writer(comment)]


def ensure_tracking_note(client, issues_url, tracker, entry, comments):
    if not is_writer(tracker):
        entry["tracking_update"] = "tracker author differs from configured account"
        return
    note = tracking_body(entry)
    if note not in comments and note != tracker.get("body"):
        client.post(f"{issues_url}/{tracker['number']}/comments", {"body": note})


def verify_apply_context(client, root_sha):
    writer = client.get("/user")
    require(isinstance(writer, dict) and writer.get("login") == RECONCILE_ACCOUNT
            and writer.get("is_admin") is False, "apply requires the non-admin reconcile account")
    root = client.get(f"/repos/{ROOT_REPOSITORY}")
    require(isinstance(root, dict) and root.get("default_branch") == "main",
            "apply requires the root default branch")
    branch = client.get(f"/repos/{ROOT_REPOSITORY}/branches/main")
    require(isinstance(branch, dict) and branch.get("protected") is True
            and (branch.get("commit") or {}).get("id") == root_sha,
            "apply requires the current protected root commit")


def reconcile(client, root=PROJECT_ROOT, apply=False, report_out=None, cutoff=None):
    root_sha, graph, managed = managed_repositories(root)
    cutoff_time = timestamp(cutoff) if cutoff else None
    issues_url = f"/repos/{ROOT_REPOSITORY}/issues"
    issues = list(pages(client, issues_url, state="all", type="issues", labels=LABEL))
    if apply:
        verify_apply_context(client, root_sha)
    labels = f"/repos/{ROOT_REPOSITORY}/labels"
    label_id = next((x["id"] for x in pages(client, labels) if x.get("name") == LABEL), None) if apply else None
    results, errors = [], {}
    for repository, component in managed.items():
        known_tags = set()
        try:
            releases = list(pages(client, f"/repos/{repository}/releases"))
            for release in releases:
                if release.get("draft"):
                    continue
                tag = release.get("tag_name")
                require(match(TAG, tag), "invalid release tag in listing")
                require(tag not in known_tags, f"duplicate release in paginated listing: {tag}")
                known_tags.add(tag)
                published = timestamp(release.get("published_at") or release.get("created_at"))
                marker = release_marker(repository, tag)
                trackers = [issue for issue in issues
                            if (issue.get("body") or "").splitlines()[:1] == [marker]]
                if cutoff_time and published < cutoff_time and not trackers:
                    continue
                entry = {
                    "component": component["id"], "path": component["path"], "repository": repository,
                    "tag": tag, "marker": marker, "published_at": published.isoformat(),
                    "accepted_sha": component["sha"],
                    "component_state": component["state"],
                }
                comments = tracking_comments(client, issues_url, trackers[0]) if len(trackers) == 1 else []
                recorded = comments + ([trackers[0].get("body") or ""] if trackers and
                                       is_writer(trackers[0]) else [])
                try:
                    require(len(trackers) <= 1, "multiple release tracking Issues")
                    entry.update(verify_release(client, repository, component, tag, release, graph))
                    if entry.get("historical"):
                        entry["status"] = "historical-baseline"
                    else:
                        affected, domains, co_validation = impact.affected_set(
                            graph, graph, [{"id": component["id"]}])
                        direct = [cid for cid, item in graph.items()
                                  if component["id"] in item["requires"]]
                        entry.update(status="pending", affected=affected, domains=domains,
                                     checks_with=co_validation,
                                     direct_consumers=sorted(cid for cid in direct if graph[cid]["state"] == "active"),
                                     planned_consumers=sorted(cid for cid in direct if graph[cid]["state"] == "planned"),
                                     governance_owner=component["parent"] or ROOT_REPOSITORY)
                        old = {line for body in recorded for line in body.splitlines()
                               if line.startswith("<!-- marqdo:verified-release ")}
                        require(not old or old == {identity_marker(entry)},
                                "published release identity or index digest changed")
                except ReconcileError as error:
                    entry.update(status="blocked", error=str(error))

                if trackers:
                    entry["issue"] = trackers[0]["number"]
                    if entry["status"] == "pending" and any(
                            identity_marker(entry) in body.splitlines() for body in recorded):
                        entry["status"] = "tracked"
                    elif apply and len(trackers) == 1 and entry["status"] in {"pending", "blocked"}:
                        ensure_tracking_note(client, issues_url, trackers[0], entry, comments)
                elif apply and entry["status"] in {"pending", "blocked"}:
                    require(label_id is not None, f"tracking label {LABEL!r} is missing")
                    created = client.post(issues_url, {
                        "title": f"[release] {repository} {tag}", "labels": [label_id],
                        "body": tracking_body(entry),
                    })
                    entry["issue"] = created["number"]
                    if entry["status"] == "pending":
                        entry["status"] = "created"
                results.append(entry)
            prefix = f"<!-- marqdo:release-reconcile repository={repository} tag="
            for issue in issues:
                marker = (issue.get("body") or "").partition("\n")[0]
                if marker.startswith(prefix) and marker.endswith(" -->"):
                    tag = marker[len(prefix):-4]
                    if match(TAG, tag) and tag not in known_tags:
                        error = "previously tracked release is absent from the public list"
                        entry = {"repository": repository, "component": component["id"],
                                 "tag": tag, "issue": issue["number"], "status": "blocked",
                                 "error": error}
                        if apply and is_writer(issue):
                            note = f"{marker}\n\n## Blocked\n\n- {error}"
                            if note not in tracking_comments(client, issues_url, issue):
                                client.post(f"{issues_url}/{issue['number']}/comments", {"body": note})
                        results.append(entry)
        except ReconcileError as error:
            errors[repository] = str(error)
    report = {
        "schema": "marqdo.release-reconcile.v1", "schema_version": 1,
        "root_repository": ROOT_REPOSITORY, "root_sha": root_sha,
        "apply": apply, "repositories": len(managed), "releases": len(results),
        "scan_complete": not errors, "scan_errors": errors, "results": results,
        "counts": {status: sum(item["status"] == status for item in results)
                   for status in sorted({item["status"] for item in results})},
        "generated_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    }
    if report_out:
        report_out.parent.mkdir(parents=True, exist_ok=True)
        report_out.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    return report


def main(argv):
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--plan", action="store_true", help="verify without writing to Gitea (default)")
    mode.add_argument("--apply", action="store_true", help="reconcile release tracking Issues")
    parser.add_argument("--report-out", type=Path)
    parser.add_argument("--root", type=Path, default=PROJECT_ROOT)
    parser.add_argument("--published-after", default=os.environ.get("MARQDO_RECONCILE_PUBLISHED_AFTER"))
    args = parser.parse_args(argv)
    token = os.environ.get("MARQDO_GIT_TOKEN", "")
    if args.apply and not token:
        parser.error("--apply requires MARQDO_GIT_TOKEN")
    try:
        if args.published_after:
            timestamp(args.published_after)
    except ReconcileError as error:
        parser.error(str(error))
    try:
        report = reconcile(Gitea(token), args.root.resolve(), args.apply,
                           args.report_out, args.published_after)
    except (OSError, ReconcileError) as error:
        print(f"reconcile: {error}", file=sys.stderr)
        return 1
    print(json.dumps(report, ensure_ascii=False, indent=2))
    return int(not report["scan_complete"] or any(
        item["status"] == "blocked" or "tracking_update" in item for item in report["results"]))


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
