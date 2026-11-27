"""Release reconcile discovery/idempotency tests (issue #10 §6/§7)."""

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "reconcile_releases", ROOT / "scripts" / "reconcile-releases.py"
)
reconcile = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(reconcile)


class FakeGitea:
    def __init__(self, releases=None, assets=None, issues=None):
        self.releases = releases or {}
        self.asset_bytes = assets or {}
        self.issues = list(issues or [])
        self.posts = []
        self.label_created = False

    def get(self, path):
        if path.startswith("/repos/") and path.endswith("/labels?limit=100"):
            return [{"id": 5, "name": reconcile.LABEL}] if self.label_created else []
        if "/issues?" in path:
            return self.issues
        for repository, releases in self.releases.items():
            if path == f"/repos/{repository}/releases?limit=50":
                return releases
        return []

    def post(self, path, data):
        self.posts.append((path, data))
        if path.endswith("/labels"):
            self.label_created = True
            return {"id": 5, "name": data["name"]}
        if path.endswith("/issues"):
            issue = {"number": 100 + len(self.issues), "body": data["body"], "title": data["title"]}
            self.issues.append(issue)
            return issue
        return {}

    def download(self, url):
        data = self.asset_bytes.get(url)
        if data is None:
            raise AssertionError(f"unexpected asset download {url}")
        return data


def release(repository="cflmy/marqdo-cli", tag="v0.0.1", asset_url="https://example.invalid/a.tar.gz", draft=False):
    return {
        "tag_name": tag,
        "name": tag,
        "draft": draft,
        "prerelease": True,
        "html_url": f"https://git.cflmy.cn/{repository}/releases/tag/{tag}",
        "published_at": "2026-11-27T00:00:00Z",
        "assets": [{"name": "a.tar.gz", "size": 3, "browser_download_url": asset_url}],
    }


class ReconcileTest(unittest.TestCase):
    def test_dry_run_discovers_releases_without_writing(self):
        client = FakeGitea(releases={"cflmy/marqdo-cli": [release()]})
        report = reconcile.reconcile(client, ROOT, apply=False)
        self.assertFalse(client.posts)
        self.assertGreaterEqual(report["releases"], 1)
        entry = [item for item in report["results"] if item["repository"] == "cflmy/marqdo-cli"][0]
        self.assertEqual(entry["status"], "pending")
        self.assertEqual(entry["tag"], "v0.0.1")

    def test_apply_verifies_assets_and_creates_marked_issue(self):
        client = FakeGitea(
            releases={"cflmy/marqdo-cli": [release()]},
            assets={"https://example.invalid/a.tar.gz": b"abc"},
        )
        report = reconcile.reconcile(client, ROOT, apply=True)
        created = [data for path, data in client.posts if path.endswith("/issues")]
        self.assertEqual(len(created), 1)
        body = created[0]["body"]
        self.assertIn(reconcile.release_marker("cflmy/marqdo-cli", "v0.0.1"), body)
        self.assertIn("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad", body)
        self.assertEqual(created[0]["labels"], [5])
        entry = [item for item in report["results"] if item["repository"] == "cflmy/marqdo-cli"][0]
        self.assertEqual(entry["status"], "created")

    def test_second_scan_is_idempotent(self):
        client = FakeGitea(
            releases={"cflmy/marqdo-cli": [release()]},
            assets={"https://example.invalid/a.tar.gz": b"abc"},
        )
        reconcile.reconcile(client, ROOT, apply=True)
        posts_after_first = len(client.posts)
        report = reconcile.reconcile(client, ROOT, apply=True)
        self.assertEqual(len(client.posts), posts_after_first)
        entry = [item for item in report["results"] if item["repository"] == "cflmy/marqdo-cli"][0]
        self.assertEqual(entry["status"], "tracked")

    def test_published_after_filter_skips_historical_releases(self):
        client = FakeGitea(releases={"cflmy/marqdo-cli": [release()]})
        skipped = reconcile.reconcile(client, ROOT, apply=False, cutoff="2026-11-28T00:00:00Z")
        self.assertFalse([item for item in skipped["results"] if item["repository"] == "cflmy/marqdo-cli"])
        kept = reconcile.reconcile(client, ROOT, apply=False, cutoff="2026-11-26T00:00:00Z")
        self.assertTrue([item for item in kept["results"] if item["repository"] == "cflmy/marqdo-cli"])

    def test_draft_releases_are_ignored(self):
        client = FakeGitea(releases={"cflmy/marqdo-cli": [release(draft=True)]})
        report = reconcile.reconcile(client, ROOT, apply=False)
        self.assertFalse([item for item in report["results"] if item["repository"] == "cflmy/marqdo-cli"])

    def test_asset_size_mismatch_and_unsafe_name_are_rejected(self):
        bad_size = FakeGitea(
            releases={"cflmy/marqdo-cli": [release()]},
            assets={"https://example.invalid/a.tar.gz": b"abcdef"},
        )
        report = reconcile.reconcile(bad_size, ROOT, apply=True)
        entry = [item for item in report["results"] if item["repository"] == "cflmy/marqdo-cli"][0]
        self.assertTrue(entry["status"].startswith("asset-verify-failed"))
        self.assertFalse([path for path, _ in bad_size.posts if path.endswith("/issues")])
        unsafe = release()
        unsafe["assets"][0]["name"] = "../escape.tar.gz"
        client = FakeGitea(releases={"cflmy/marqdo-cli": [unsafe]}, assets={"https://example.invalid/a.tar.gz": b"abc"})
        report = reconcile.reconcile(client, ROOT, apply=True)
        entry = [item for item in report["results"] if item["repository"] == "cflmy/marqdo-cli"][0]
        self.assertTrue(entry["status"].startswith("asset-verify-failed"))


if __name__ == "__main__":
    unittest.main()
