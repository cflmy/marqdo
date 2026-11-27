"""Trusted webhook adapter and reconciliation tests (issue #3 / #10 gap).

The fixtures are real temporary Git repositories. Webhook bodies are signed with
a test secret; the adapter must refuse tampered bodies, unknown repositories,
failed CI and replayed candidates, and reconciliation must recover a candidate
whose fast notification was lost while remaining idempotent.
"""

import hashlib
import hmac
import importlib.util
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ADAPTER_SPEC = importlib.util.spec_from_file_location(
    "marqdo_webhook_adapter", ROOT / "scripts" / "webhook-adapter.py"
)
adapter = importlib.util.module_from_spec(ADAPTER_SPEC)
ADAPTER_SPEC.loader.exec_module(adapter)

GIT_ENV = {
    "GIT_AUTHOR_NAME": "test",
    "GIT_AUTHOR_EMAIL": "test@example.invalid",
    "GIT_COMMITTER_NAME": "test",
    "GIT_COMMITTER_EMAIL": "test@example.invalid",
}
SECRET = "test-webhook-secret"
CI_URL = "https://ci.cflmy.cn/repos/1/pipeline/7/1"
CATALOG = """[[component]]
id = "cli"
path = "cli"
kind = "core"
state = "active"
owner = "t"
scope = "s"
requires = []
checks_with = []
release = "base"
entry = "scripts/test.sh"
"""
GITMODULES = """[submodule "cli"]
\tpath = cli
\turl = https://git.cflmy.cn/cflmy/marqdo-cli.git
"""


def git(cwd, *args):
    env = os.environ.copy()
    env.update(GIT_ENV)
    result = subprocess.run(["git", "-C", str(cwd), *args], capture_output=True, env=env)
    if result.returncode != 0:
        raise AssertionError(f"git {' '.join(args)}: {result.stderr.decode(errors='replace')}")
    return result.stdout.decode()


def commit(repo, message="fixture"):
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", message)
    return git(repo, "rev-parse", "HEAD").strip()


def write(path, text):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


class WebhookAdapterTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.base = Path(self.tmp.name)
        self.root = self.base / "root"
        self.root.mkdir()
        git(self.root, "init", "-q")
        write(self.root / "components.toml", CATALOG)
        write(self.root / ".gitmodules", GITMODULES)
        self.cli = self.root / "cli"
        self.cli.mkdir()
        git(self.cli, "init", "-q")
        write(self.cli / "README.md", "cli")
        self.sha1 = commit(self.cli)
        git(self.root, "add", "components.toml", ".gitmodules")
        git(self.root, "update-index", "--add", "--cacheinfo", f"160000,{self.sha1},cli")
        git(self.root, "commit", "-q", "-m", "accepted")
        self.accepted = git(self.root, "rev-parse", "HEAD").strip()
        write(self.cli / "README.md", "cli v2")
        self.sha2 = commit(self.cli, "candidate")
        self.state = self.base / "state.json"
        self.originals = (adapter.PROJECT_ROOT, adapter.ROOT)
        adapter.ROOT = self.root

    def tearDown(self):
        adapter.PROJECT_ROOT, adapter.ROOT = self.originals
        self.tmp.cleanup()

    def push_body(self, after=None, before=None, repository="cflmy/marqdo-cli"):
        return json.dumps(
            {
                "repository": {"full_name": repository},
                "ref": "refs/heads/candidate",
                "before": before or self.sha1,
                "after": after or self.sha2,
            }
        ).encode()

    def statuses(self, status="success"):
        return [{"status": status, "context": "ci/woodpecker/pr/main", "target_url": CI_URL}]

    def verify(self, body, event="push", signature=None, statuses=None, write_state=True):
        signature = signature if signature is not None else hmac.new(
            SECRET.encode(), body, hashlib.sha256
        ).hexdigest()
        return adapter.verify_webhook(
            body,
            event,
            signature,
            SECRET,
            statuses if statuses is not None else self.statuses(),
            self.accepted,
            self.state,
            ["ci.cflmy.cn"],
            "cflmy/marqdo",
            write_state,
        )

    def test_signed_push_candidate_is_verified_without_auto_promotion(self):
        plan = self.verify(self.push_body())
        self.assertTrue(plan["verified"])
        self.assertIsNotNone(plan.get("impact"))
        self.assertFalse(plan["actions"]["auto_promote"])
        self.assertFalse(plan["actions"]["auto_merge"])
        self.assertTrue(self.state.is_file())

    def test_tampered_body_and_missing_secret_are_refused_before_verification(self):
        body = self.push_body()
        with self.assertRaises(adapter.AdapterError):
            self.verify(body, signature="0" * 64)
        with self.assertRaises(adapter.AdapterError):
            self.verify(body, signature="")
        with self.assertRaises(adapter.AdapterError):
            adapter.verify_webhook(
                body, "push", None, "", self.statuses(), self.accepted, self.state,
                ["ci.cflmy.cn"], "cflmy/marqdo",
            )

    def test_unsupported_events_and_deleted_branches_are_refused(self):
        with self.assertRaises(adapter.AdapterError):
            self.verify(self.push_body(), event="issues")
        body = json.dumps(
            {
                "repository": {"full_name": "cflmy/marqdo-cli"},
                "before": self.sha1,
                "after": "0" * 40,
                "deleted": True,
            }
        ).encode()
        with self.assertRaises(adapter.AdapterError):
            self.verify(body)

    def test_unknown_repository_and_failed_or_missing_ci_are_rejected(self):
        with self.assertRaises(adapter.AdapterError):
            self.verify(self.push_body(repository="cflmy/unknown"))
        failed = self.verify(self.push_body(), statuses=self.statuses("failure"))
        self.assertFalse(failed["verified"])
        self.assertIn("CI status", failed["reason"])
        pending = self.verify(self.push_body(), statuses=self.statuses("pending"))
        self.assertFalse(pending["verified"])
        empty = self.verify(self.push_body(), statuses=[])
        self.assertFalse(empty["verified"])

    def test_pr_event_uses_the_accepted_pin_not_the_payload_base(self):
        body = json.dumps(
            {
                "action": "opened",
                "repository": {"full_name": "cflmy/marqdo-cli"},
                "pull_request": {
                    "number": 21,
                    "head": {"sha": self.sha2},
                    "base": {"sha": self.sha1},
                },
            }
        ).encode()
        plan = self.verify(body, event="pull_request")
        self.assertTrue(plan["verified"])
        self.assertIsNotNone(plan.get("impact"))

    def test_replayed_candidate_is_rejected(self):
        self.assertTrue(self.verify(self.push_body())["verified"])
        replay = self.verify(self.push_body())
        self.assertFalse(replay["verified"])
        self.assertIn("duplicate/replay", replay["reason"])

    def test_reconciliation_recovers_a_lost_notification_and_is_idempotent(self):
        pulls = [
            {
                "number": 33,
                "head": {"sha": self.sha2},
                "base": {"sha": self.sha1},
            }
        ]

        def prs_fetcher(repository):
            return pulls if repository == "cflmy/marqdo-cli" else []

        def statuses_fetcher(repository, sha):
            return self.statuses() if sha == self.sha2 else []

        plan = adapter.reconcile(
            self.accepted,
            self.state,
            ["cflmy/marqdo-cli", "cflmy/marqdo"],
            self.accepted,
            statuses_fetcher,
            prs_fetcher,
            ["ci.cflmy.cn"],
            "cflmy/marqdo",
        )
        self.assertEqual(plan["verified"], 1)
        result = [item for item in plan["results"] if item.get("pull_request") == 33][0]
        self.assertEqual(result["status"], "VERIFIED")
        self.assertTrue(self.state.is_file())

        again = adapter.reconcile(
            self.accepted,
            self.state,
            ["cflmy/marqdo-cli", "cflmy/marqdo"],
            self.accepted,
            statuses_fetcher,
            prs_fetcher,
            ["ci.cflmy.cn"],
            "cflmy/marqdo",
        )
        self.assertEqual(again["verified"], 0)
        self.assertTrue(any(item["status"] == "DUPLICATE" for item in again["results"]))

    def test_reconciliation_reports_api_failures_without_writing_state(self):
        plan = adapter.reconcile(
            self.accepted,
            self.state,
            ["cflmy/marqdo-cli"],
            self.accepted,
            lambda repository, sha: self.statuses(),
            lambda repository: (_ for _ in ()).throw(adapter.AdapterError("HTTP 403")),
            ["ci.cflmy.cn"],
            "cflmy/marqdo",
        )
        self.assertEqual(plan["verified"], 0)
        self.assertEqual(plan["results"][0]["status"], "ERROR")
        self.assertFalse(self.state.exists())


if __name__ == "__main__":
    unittest.main()
