"""Release reconciliation tests for marqdo#10."""

import hashlib
import importlib.util
import io
import json
import os
import sys
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest.mock import patch
from urllib.parse import parse_qs, urlsplit

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "reconcile_releases", ROOT / "scripts" / "reconcile-releases.py"
)
reconcile = importlib.util.module_from_spec(SPEC)
with patch.object(sys, "path", [str(ROOT / "scripts"), *sys.path]):
    SPEC.loader.exec_module(reconcile)

REPO = "cflmy/marqdo-std-text"
SOURCE = "a" * 40
BASE = "b" * 40
TAG = "v0.1.0-rc.1"
PUBLISHED = "2026-11-27T08:00:00+08:00"
ASSET = "text-source.tar.gz"
ISSUES = "/repos/cflmy/marqdo/issues"
CONSUMER_ISSUES = "/repos/cflmy/marqdo-ext-archive/issues"
FORGE = reconcile.FORGE
ACTOR = {"login": reconcile.RECONCILE_ACCOUNT}
COMPONENT = {"id": "std-text", "sha": BASE, "path": "stdlib/text",
             "parent": None, "requires": [], "checks_with": [], "state": "active"}


class FakeForge:
    def __init__(self):
        self.writes = []
        self.issues = []
        self.consumer_issues = []
        self.comments = {}
        self.pages = []
        self.pipeline_reads = []
        self.downloads = []
        self.blobs = {ASSET: b"abc"}
        self.evidence = {
            "repository": REPO, "component_id": COMPONENT["id"], "tag": TAG, "source_sha": SOURCE,
            "ci": {"pipeline_number": 12, "source_sha": SOURCE,
                   "url": f"{reconcile.CI}/repos/8/pipeline/12"},
            "assets": [{"name": ASSET, "size": 3, "target": "source",
                        "sha256": hashlib.sha256(self.blobs[ASSET]).hexdigest()}],
            "dependencies": [],
        }
        self.release = {"tag_name": TAG, "draft": False, "prerelease": True,
                        "published_at": PUBLISHED}
        self.releases = [self.release]
        self.protected = True
        self.root_sha = BASE
        self.tag_sha = SOURCE
        self.pipeline_sha = SOURCE
        self.wrong_account = False
        self.fail_page = None
        self.refresh_assets()

    def refresh_assets(self):
        self.blobs[reconcile.INDEX] = json.dumps(self.evidence).encode()
        self.release["assets"] = [
            {"name": name, "size": len(blob),
             "browser_download_url": f"{FORGE}/{REPO}/releases/download/{TAG}/{name}"}
            for name, blob in self.blobs.items()
        ]

    def get(self, url):
        parsed = urlsplit(url)
        path = parsed.path
        query = parse_qs(parsed.query)
        if query:
            page = int(query["page"][0])
            self.pages.append((path, page))
            if path.endswith("/releases") and page == self.fail_page:
                raise reconcile.ReconcileError("API HTTP 503")
            records = {
                f"/repos/{REPO}/releases": self.releases,
                ISSUES: self.issues,
                CONSUMER_ISSUES: self.consumer_issues,
                "/repos/cflmy/marqdo/labels": [{"id": 5, "name": reconcile.LABEL}],
                **self.comments,
            }.get(path, [])
            return records[(page - 1) * reconcile.PAGE_SIZE: page * reconcile.PAGE_SIZE]

        return {
            f"/repos/{REPO}": {"id": 10},
            f"/repos/{REPO}/tags/{TAG}": {"name": TAG, "commit": {"sha": self.tag_sha}},
            "/repos/cflmy/marqdo": {"default_branch": "main"},
            "/repos/cflmy/marqdo/branches/main": {"protected": self.protected,
                                                 "commit": {"id": self.root_sha}},
            "/user": ({"login": "admin", "is_admin": True} if self.wrong_account
                      else {**ACTOR, "is_admin": False}),
        }[path]

    def ci(self, path):
        self.pipeline_reads.append(path)
        if path == f"/repos/lookup/{REPO}":
            return {"id": 8, "full_name": REPO, "forge_remote_id": "10"}
        if path == "/repos/8/pipelines/12":
            return {"number": 12, "commit": self.pipeline_sha, "status": "success", "event": "push",
                    "workflows": [{"name": "ci", "state": "success",
                                   "children": [{"type": "commands", "state": "success"}]}]}
        raise AssertionError(path)

    def asset(self, path, size, *, index=False):
        self.downloads.append(path)
        data = self.blobs[path.rsplit("/", 1)[1]]
        if len(data) != size:
            raise reconcile.ReconcileError("asset size mismatch")
        return hashlib.sha256(data).hexdigest(), data if index else b""

    def post(self, path, data):
        self.writes.append((path, data))
        item = {"body": data["body"], "user": ACTOR, "state": "open"}
        if path in {ISSUES, CONSUMER_ISSUES}:
            issues = self.issues if path == ISSUES else self.consumer_issues
            item["number"] = 100 + len(issues)
            issues.append(item)
        else:
            self.comments.setdefault(path, []).append(item)
        return item


class ReconcileTest(unittest.TestCase):
    def setUp(self):
        graph = {COMPONENT["id"]: COMPONENT}
        catalog = (BASE, graph, {REPO: COMPONENT})
        mocked = patch.object(reconcile, "managed_repositories", return_value=catalog)
        mocked.start()
        self.addCleanup(mocked.stop)
        self.forge = FakeForge()

    def run_scan(self, **kwargs):
        return reconcile.reconcile(self.forge, **kwargs)

    def test_native_dispatch_reuses_a_run_for_the_same_task_and_root(self):
        task = {"repository": REPO, "baseline_sha": BASE, "issue": 42, "key": "c" * 64}
        routes = {
            f"/repos/{REPO}/branches/master": {"protected": True, "commit": {"id": BASE}},
            f"/repos/{REPO}/contents/.woodpecker/upstream.yml?ref={BASE}": {"type": "file"},
            "/repos/cflmy/marqdo/branches/main": {"protected": True, "commit": {"id": BASE}},
        }
        runs = []
        posts = []

        def dispatch(path, data=None):
            if path == "/repos/8/permissions":
                return {"push": True, "admin": False}
            if data is None:
                self.assertTrue(path.startswith("/repos/8/pipelines?"))
                return runs
            self.assertEqual(path, "/repos/8/pipelines")
            self.assertEqual(data["branch"], "master")
            self.assertEqual(data["workflow"], "upstream")
            self.assertEqual(data["variables"]["MARQDO_UPSTREAM_ISSUE"], "42")
            posts.append(data)
            runs.append({"number": 15, "status": "pending", "author": ACTOR["login"],
                         "event": "manual", "branch": data["branch"], "commit": BASE,
                         "selected_workflow": data["workflow"], "variables": data["variables"]})
            return runs[-1]

        with (patch.object(reconcile, "ci_repository", return_value=(8, {"default_branch": "master"})),
              patch.object(self.forge, "get", side_effect=routes.__getitem__),
              patch.object(self.forge, "dispatch_ci", side_effect=dispatch, create=True)):
            self.assertTrue(reconcile.dispatch_consumer(self.forge, BASE, task, PUBLISHED, True))
            self.assertFalse(reconcile.dispatch_consumer(self.forge, BASE, task, PUBLISHED, True))
            self.assertEqual(task["pipeline"], f"{reconcile.CI}/repos/8/pipeline/15")
            self.assertEqual(len(posts), 1)
            routes[f"/repos/{REPO}/branches/master"]["protected"] = False
            with self.assertRaisesRegex(reconcile.ReconcileError, "unprotected"):
                reconcile.dispatch_consumer(self.forge, BASE, task, PUBLISHED, True)
            self.assertEqual(len(posts), 1)

    def test_dispatch_rejects_owner_or_admin_identity_before_writing_to_ci(self):
        for identity in ({"login": "cflmy"}, {**ACTOR, "admin": True}):
            with patch.object(self.forge, "dispatch_ci", return_value=identity, create=True) as ci:
                report = self.run_scan(apply=True, dispatch=True)
            ci.assert_called_once_with("/user")
            self.assertTrue(report["scan_complete"])
            self.assertEqual(report["dispatch_status"], "unavailable")

    def test_dry_run_fully_verifies_without_writing(self):
        report = self.run_scan()
        self.assertEqual(self.forge.writes, [])
        entry, = report["results"]
        self.assertEqual(entry["status"], "pending")
        self.assertEqual(entry["source_sha"], SOURCE)
        self.assertEqual(entry["governance_owner"], "cflmy/marqdo")
        self.assertEqual(len(self.forge.downloads), 2)
        self.assertEqual(len(self.forge.pipeline_reads), 2)

    def test_consumer_inputs_are_tied_to_assets_dependencies_and_accepted_sha(self):
        consumer = {"id": "archive", "state": "active", "sha": "c" * 40,
                    "remote": FORGE + "/cflmy/marqdo-ext-archive.git",
                    "requires": ["std-text", "cli"], "checks_with": []}
        cli = {"id": "cli", "state": "active", "sha": "d" * 40,
               "requires": [], "checks_with": []}
        graph = {item["id"]: item for item in (COMPONENT, consumer, cli)}
        with patch.object(reconcile, "managed_repositories", return_value=(BASE, graph, {REPO: COMPONENT})):
            entry, = self.run_scan()["results"]
            task, = entry["consumer_tasks"]
            self.assertEqual(task["consumer"], "archive")
            self.assertEqual(task["baseline_sha"], consumer["sha"])
            self.assertEqual(task["inputs"]["release"]["assets"][0]["name"], ASSET)
            self.assertEqual(task["inputs"]["accepted_dependencies"], [{"id": "cli", "source_sha": cli["sha"]}])
            self.assertEqual(task["status"], "pending")
            cli["sha"] = "e" * 40
            first_key = task["key"]
            entry, = self.run_scan()["results"]
            task, = entry["consumer_tasks"]
            self.assertNotEqual(task["key"], first_key)
            self.forge.evidence["dependencies"] = [{"id": "cli", "source_sha": "d" * 40}]
            self.forge.refresh_assets()
            entry, = self.run_scan()["results"]
            task, = entry["consumer_tasks"]
            self.assertEqual(task["status"], "waiting-for-dependencies")
            self.assertEqual(task["different_dependencies"], ["cli"])
        self.assertEqual(self.forge.writes, [])

    def test_apply_creates_one_consumer_issue_and_records_new_inputs_in_it(self):
        consumer = {"id": "archive", "state": "active", "sha": "c" * 40,
                    "remote": FORGE + "/cflmy/marqdo-ext-archive.git",
                    "requires": ["std-text"], "checks_with": []}
        graph = {item["id"]: item for item in (COMPONENT, consumer)}
        with patch.object(reconcile, "managed_repositories", return_value=(BASE, graph, {REPO: COMPONENT})):
            entry, = self.run_scan(apply=True)["results"]
            task, = entry["consumer_tasks"]
            self.assertEqual(task["registration"], "recorded")
            self.assertEqual(task["issue"], 100)
            self.assertIn(FORGE + "/cflmy/marqdo/issues/100", self.forge.consumer_issues[0]["body"])
            self.run_scan(apply=True)
            self.assertEqual(len(self.forge.writes), 2)
            consumer["sha"] = "e" * 40
            self.run_scan(apply=True)
            self.assertEqual(len(self.forge.consumer_issues), 1)
            self.assertEqual(len(self.forge.comments[CONSUMER_ISSUES + "/100/comments"]), 1)
            self.run_scan(apply=True)
            self.assertEqual(len(self.forge.comments[CONSUMER_ISSUES + "/100/comments"]), 1)
        new_root = "f" * 40
        self.forge.root_sha = new_root
        with patch.object(reconcile, "managed_repositories", return_value=(new_root, graph, {REPO: COMPONENT})):
            self.run_scan(apply=True)
            self.run_scan(apply=True)
        notes = self.forge.comments[CONSUMER_ISSUES + "/100/comments"]
        self.assertEqual(len(notes), 2)
        self.assertIn(f"- Root candidate: {new_root}", notes[-1]["body"])

    def test_apply_creates_one_verified_tracker_then_rechecks_it(self):
        entry, = self.run_scan(apply=True)["results"]
        self.assertEqual(entry["status"], "created")
        self.assertEqual(entry["issue"], 100)
        self.assertEqual(self.forge.writes[0][1]["labels"], [5])
        body = self.forge.issues[0]["body"]
        self.assertIn(reconcile.release_marker(REPO, TAG), body)
        self.assertIn(reconcile.identity_marker(entry), body)
        entry, = self.run_scan(apply=True)["results"]
        self.assertEqual(entry["status"], "tracked")
        self.assertEqual(len(self.forge.writes), 1)
        self.assertEqual(len(self.forge.downloads), 4)

    def test_partial_upload_and_wrong_ci_sha_block_once(self):
        self.forge.pipeline_sha = BASE
        for _ in range(2):
            entry, = self.run_scan(apply=True)["results"]
            self.assertEqual(entry["status"], "blocked")
            self.assertIn("exact SHA", entry["error"])
        self.assertEqual(len(self.forge.writes), 1)
        self.forge.pipeline_sha = SOURCE
        self.forge.release["assets"].pop(0)
        entry, = self.run_scan()["results"]
        self.assertEqual(entry["status"], "blocked")
        self.assertIn("cover the published assets", entry["error"])

    def test_recorded_release_block_does_not_fail_a_complete_operational_scan(self):
        self.forge.pipeline_sha = BASE
        with (patch.dict(os.environ, {"MARQDO_GIT_TOKEN": "fixture"}),
              patch.object(reconcile, "Gitea", return_value=self.forge),
              redirect_stdout(io.StringIO()) as stdout):
            result = reconcile.main(["--apply"])
        self.assertEqual(result, 0)
        report = json.loads(stdout.getvalue())
        self.assertTrue(report["scan_complete"])
        self.assertEqual(report["results"][0]["status"], "blocked")
        self.assertEqual(len(self.forge.issues), 1)

    def test_asset_api_outage_leaves_the_scan_incomplete(self):
        with patch.object(self.forge, "asset", side_effect=reconcile.http_error("GET", "/asset", 503)):
            report = self.run_scan(apply=True)
        self.assertFalse(report["scan_complete"])
        self.assertEqual(report["scan_errors"][REPO], "GET /asset: HTTP 503")
        self.assertEqual(self.forge.writes, [])

    def test_validated_tracker_detects_an_updated_index_and_disappearing_release(self):
        self.run_scan(apply=True)
        self.forge.evidence["assets"][0]["target"] = "x86_64-unknown-linux-gnu"
        self.forge.refresh_assets()
        entry, = self.run_scan(apply=True)["results"]
        self.assertEqual(entry["status"], "blocked")
        self.assertIn("index digest changed", entry["error"])
        self.assertEqual(len(self.forge.issues), 1)
        self.forge.releases = []
        entry, = self.run_scan()["results"]
        self.assertIn("absent", entry["error"])

    def test_previously_blocked_tracker_records_verified_identity_in_bot_comment(self):
        self.forge.pipeline_sha = BASE
        self.run_scan(apply=True)
        self.forge.pipeline_sha = SOURCE
        entry, = self.run_scan(apply=True)["results"]
        self.assertEqual(entry["status"], "pending")
        self.assertIn(reconcile.identity_marker(entry), self.forge.writes[-1][1]["body"])
        entry, = self.run_scan(apply=True)["results"]
        self.assertEqual(entry["status"], "tracked")
        self.assertEqual(len(self.forge.writes), 2)

    def test_uses_normalized_publish_timestamp_and_skips_draft(self):
        self.assertEqual(self.run_scan(cutoff="2026-11-27T00:00:01Z")["results"], [])
        self.assertTrue(self.run_scan(cutoff="2026-11-27T00:00:00Z")["results"])
        self.forge.release["draft"] = True
        self.assertEqual(self.run_scan()["results"], [])

    def test_multiple_pages_and_partial_list_error(self):
        self.forge.releases = [{"draft": True}] * reconcile.PAGE_SIZE + [self.forge.release]
        report = self.run_scan()
        self.assertTrue(report["scan_complete"])
        self.assertEqual(len(report["results"]), 1)
        self.assertIn((f"/repos/{REPO}/releases", 2), self.forge.pages)
        self.forge.fail_page = 2
        report = self.run_scan(apply=True)
        self.assertFalse(report["scan_complete"])
        self.assertEqual(report["scan_errors"][REPO], "API HTTP 503")
        self.assertEqual(self.forge.writes, [])

    def test_asset_location_name_and_digest_are_checked(self):
        asset = self.forge.release["assets"][0]
        asset["browser_download_url"] = "https://invalid.example/capture"
        entry, = self.run_scan()["results"]
        self.assertIn("non-canonical URL", entry["error"])
        self.assertEqual(self.forge.downloads, [])
        self.forge.refresh_assets()
        self.forge.release["assets"][0]["name"] = "../escape.tar.gz"
        entry, = self.run_scan()["results"]
        self.assertIn("unsafe release asset name", entry["error"])
        self.forge.refresh_assets()
        self.forge.blobs[ASSET] = b"xxx"
        entry, = self.run_scan()["results"]
        self.assertIn("digest mismatch", entry["error"])

    def test_apply_requires_current_protected_branch_and_non_admin_identity(self):
        self.forge.protected = False
        with self.assertRaisesRegex(reconcile.ReconcileError, "protected root"):
            self.run_scan(apply=True)
        self.forge.wrong_account = True
        with self.assertRaisesRegex(reconcile.ReconcileError, "non-admin"):
            self.run_scan(apply=True)
        self.assertEqual(self.forge.writes, [])


if __name__ == "__main__":
    unittest.main()
