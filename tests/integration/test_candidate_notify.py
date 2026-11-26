"""Candidate notification authorization contract tests (issue #3).

Fixtures are real temporary Git repositories: a root catalog with `.gitmodules`
and gitlinks, a component repository with two commits, and normalized
notifications. The verifier must refuse forged/replayed/stale/failed candidates
and never claim an auto-promotion action.
"""

import importlib.util
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
IMPACT_SPEC = importlib.util.spec_from_file_location("impact", ROOT / "scripts/impact.py")
impact = importlib.util.module_from_spec(IMPACT_SPEC)
IMPACT_SPEC.loader.exec_module(impact)
NOTIFY_SPEC = importlib.util.spec_from_file_location(
    "candidate_notify", ROOT / "scripts/candidate-notify.py"
)
notify = importlib.util.module_from_spec(NOTIFY_SPEC)
NOTIFY_SPEC.loader.exec_module(notify)

GIT_ENV = {
    "GIT_AUTHOR_NAME": "test",
    "GIT_AUTHOR_EMAIL": "test@example.invalid",
    "GIT_COMMITTER_NAME": "test",
    "GIT_COMMITTER_EMAIL": "test@example.invalid",
}
CI = {"status": "success", "url": "https://ci.cflmy.cn/repos/6/pipeline/1/1"}


def git(cwd, *args, check=True):
    env = os.environ.copy()
    env.update(GIT_ENV)
    result = subprocess.run(["git", "-C", str(cwd), *args], capture_output=True, env=env)
    if check and result.returncode != 0:
        raise AssertionError(
            f"git {' '.join(args)} failed: {result.stderr.decode(errors='replace')}"
        )
    return result.stdout.decode()


def init(path: Path) -> Path:
    path.mkdir(parents=True, exist_ok=True)
    git(path, "init", "-q")
    return path


def write(path: Path, data: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(data)


def commit(repo: Path, message: str = "fixture") -> str:
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", message)
    return git(repo, "rev-parse", "HEAD").strip()


def catalog(cli_sha_gitlink: bool = True) -> str:
    return """[[component]]
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

[[component]]
id = "std-text"
path = "stdlib/text"
kind = "stdlib"
state = "active"
owner = "t"
scope = "s"
requires = ["cli"]
checks_with = []
release = "base"
entry = "scripts/test.sh"
"""


GITMODULES = """[submodule "cli"]
\tpath = cli
\turl = https://git.cflmy.cn/cflmy/marqdo-cli.git
[submodule "stdlib/text"]
\tpath = stdlib/text
\turl = https://git.cflmy.cn/cflmy/marqdo-std-text.git
"""


class NotifyTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.base = Path(self.tmp.name)
        self.root = self.base / "root"
        init(self.root)
        write(self.root / "components.toml", catalog())
        write(self.root / ".gitmodules", GITMODULES)
        self.cli = init(self.root / "cli")
        write(self.cli / "README.md", "cli")
        self.cli_sha1 = commit(self.cli)
        self.text = init(self.root / "stdlib/text")
        write(self.text / "README.md", "text")
        self.text_sha1 = commit(self.text)
        git(self.root, "add", "components.toml", ".gitmodules")
        git(self.root, "update-index", "--add", "--cacheinfo", f"160000,{self.cli_sha1},cli")
        git(
            self.root, "update-index", "--add", "--cacheinfo",
            f"160000,{self.text_sha1},stdlib/text",
        )
        git(self.root, "commit", "-q", "-m", "accepted root")
        self.accepted = git(self.root, "rev-parse", "HEAD").strip()
        # Candidate component commit after the accepted root was frozen.
        write(self.cli / "README.md", "cli v2")
        self.cli_sha2 = commit(self.cli, "candidate")
        # Candidate root commit moving the cli gitlink.
        git(self.root, "update-index", "--cacheinfo", f"160000,{self.cli_sha2},cli")
        git(self.root, "commit", "-q", "-m", "candidate root")
        self.candidate_root = git(self.root, "rev-parse", "HEAD").strip()
        self.notify_original_root = notify.ROOT
        self.impact_original_root = impact.ROOT
        notify.ROOT = self.root
        impact.ROOT = self.root
        self.state = self.base / "state.json"

    def tearDown(self):
        notify.ROOT = self.notify_original_root
        impact.ROOT = self.impact_original_root
        self.tmp.cleanup()

    def notification(self, **overrides) -> Path:
        data = {
            "schema": "marqdo.candidate-notification",
            "schema_version": 1,
            "repository": "cflmy/marqdo-cli",
            "kind": "component",
            "component": "cli",
            "head_sha": self.cli_sha2,
            "base_sha": self.cli_sha1,
            "ci": dict(CI),
            "pull_request": 17,
        }
        data.update(overrides)
        path = self.base / "notification.json"
        path.write_text(json.dumps(data))
        return path

    def run_verify(self, path: Path, accepted: str | None = None, write_state: bool = True):
        try:
            return notify.verify(
                json.loads(path.read_text()),
                accepted or self.accepted,
                self.state,
                ["ci.cflmy.cn"],
                "cflmy/marqdo",
                write_state=write_state,
            )
        except notify.NotificationError as error:
            return {"verified": False, "reason": str(error)}

    def test_component_candidate_verified_with_consumers_and_no_auto_promotion(self):
        plan = self.run_verify(self.notification())
        self.assertTrue(plan["verified"])
        self.assertIn("std-text", plan["impact"]["affected"])
        self.assertFalse(plan["actions"]["auto_promote"])
        self.assertFalse(plan["actions"]["auto_merge"])
        self.assertTrue(plan["actions"]["requires_human_review"])
        self.assertTrue(self.state.is_file())

    def test_unknown_repository_is_rejected(self):
        plan = self.run_verify(self.notification(repository="cflmy/unknown"))
        self.assertFalse(plan["verified"])
        self.assertIn("unknown repository", plan["reason"])

    def test_failed_ci_is_rejected(self):
        plan = self.run_verify(self.notification(ci={"status": "failure", "url": CI["url"]}))
        self.assertFalse(plan["verified"])
        self.assertIn("CI status", plan["reason"])

    def test_non_allowlisted_ci_host_is_rejected(self):
        plan = self.run_verify(
            self.notification(ci={"status": "success", "url": "https://evil.example/p/1"})
        )
        self.assertFalse(plan["verified"])
        self.assertIn("CI URL", plan["reason"])

    def test_unreachable_commit_is_rejected(self):
        plan = self.run_verify(self.notification(head_sha="0" * 40))
        self.assertFalse(plan["verified"])
        self.assertIn("not reachable", plan["reason"])

    def test_stale_base_sha_is_rejected(self):
        plan = self.run_verify(
            self.notification(base_sha=self.text_sha1)
        )
        self.assertFalse(plan["verified"])
        self.assertIn("base_sha is stale", plan["reason"])

    def test_replay_is_rejected_and_new_baseline_requires_revalidation(self):
        first = self.run_verify(self.notification())
        self.assertTrue(first["verified"])
        replay = self.run_verify(self.notification())
        self.assertFalse(replay["verified"])
        self.assertIn("duplicate/replay", replay["reason"])
        # A new accepted baseline that still pins the same component SHA (here
        # another component moved) must re-validate the same candidate.
        write(self.text / "README.md", "text v2")
        text_sha2 = commit(self.text, "text candidate")
        git(self.root, "update-index", "--cacheinfo", f"160000,{self.cli_sha1},cli")
        git(self.root, "update-index", "--cacheinfo", f"160000,{text_sha2},stdlib/text")
        git(self.root, "commit", "-q", "-m", "baseline 2")
        baseline2 = git(self.root, "rev-parse", "HEAD").strip()
        revalidated = self.run_verify(self.notification(), accepted=baseline2)
        self.assertTrue(revalidated["verified"], revalidated.get("reason"))
        self.assertTrue(revalidated["revalidate"])

    def test_root_candidate_uses_union_graph_diff(self):
        plan = self.run_verify(
            self.notification(
                repository="cflmy/marqdo",
                kind="root",
                component=None,
                head_sha=self.candidate_root,
                base_sha=self.accepted,
            )
        )
        self.assertTrue(plan["verified"])
        self.assertEqual([c["id"] for c in plan["impact"]["changed"]], ["cli"])
        self.assertIn("std-text", plan["impact"]["affected"])

    def test_root_repository_must_be_named_explicitly(self):
        plan = self.run_verify(
            self.notification(
                repository="cflmy/marqdo-cli",
                kind="root",
                head_sha=self.candidate_root,
                base_sha=self.accepted,
            )
        )
        self.assertFalse(plan["verified"])
        self.assertIn("unknown root repository", plan["reason"])

    def test_state_is_not_written_in_dry_run(self):
        plan = self.run_verify(self.notification(), write_state=False)
        self.assertTrue(plan["verified"])
        self.assertFalse(self.state.exists())


if __name__ == "__main__":
    unittest.main()
