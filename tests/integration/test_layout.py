import importlib.util
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

SCRIPT = Path(__file__).resolve().parents[2] / "scripts/check-layout.py"
SPEC = importlib.util.spec_from_file_location("check_layout", SCRIPT)
layout = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(layout)

CLI = '''
[[component]]
id = "cli"
path = "cli"
kind = "core"
state = "active"
owner = "maintainer"
scope = "core implementation"
requires = []
checks_with = []
release = "base"
entry = "scripts/test.sh"
'''

PLANNED = '''
[[component]]
id = "std-core"
path = "stdlib/core"
kind = "stdlib"
state = "planned"
owner = "maintainer"
scope = "primitive utilities"
requires = ["cli"]
checks_with = []
release = "base"
'''


class LayoutTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.enterContext(patch.object(layout, "ROOT", self.root))
        self.enterContext(patch.object(layout, "git", side_effect=self.git))
        self.git("init", "-q")
        self.git("-c", "user.name=Test", "-c", "user.email=test@example.invalid", "commit", "--allow-empty", "-qm", "fixture")
        self.sha = self.git("rev-parse", "HEAD").strip()

    def git(self, *args, cwd=None):
        return subprocess.check_output(["git", "-C", cwd or self.root, *args]).decode()

    def candidate(self, metadata=CLI, url="https://git.cflmy.cn/cflmy/marqdo-cli.git", extra=None):
        (self.root / "components.toml").write_text(metadata)
        (self.root / ".gitmodules").write_text(f'[submodule "cli"]\npath = cli\nurl = {url}\n')
        self.git("add", ".gitmodules", "components.toml")
        self.git("update-index", "--add", "--cacheinfo", f"160000,{self.sha},cli")
        if extra:
            self.git("update-index", "--add", "--cacheinfo", f"160000,{self.sha},{extra}")
        self.git("-c", "user.name=Test", "-c", "user.email=test@example.invalid", "commit", "-qm", "candidate")

    def test_accepts_active_gitlink_and_planned_component(self):
        self.candidate(CLI + PLANNED)
        layout.validate("HEAD", False)

    def test_rejects_planned_gitlink(self):
        self.candidate(CLI + PLANNED, extra="stdlib/core")
        with self.assertRaisesRegex(ValueError, "gitlink/catalog mismatch"):
            layout.validate("HEAD", False)

    def test_rejects_active_dependency_on_planned(self):
        self.candidate(CLI.replace("requires = []", 'requires = ["std-core"]') + PLANNED)
        with self.assertRaisesRegex(ValueError, "active depends on planned"):
            layout.validate("HEAD", False)

    def test_rejects_other_host(self):
        self.candidate(url="https://example.invalid/cflmy/marqdo-cli.git")
        with self.assertRaisesRegex(ValueError, "invalid submodule URL"):
            layout.validate("HEAD", False)

    def test_rejects_wrong_repository_at_authority(self):
        self.candidate(url="https://git.cflmy.cn/cflmy/marqdo-view.git")
        with self.assertRaisesRegex(ValueError, "invalid submodule URL"):
            layout.validate("HEAD", False)

    def test_rejects_unstaged_checkout(self):
        self.candidate()
        with self.assertRaisesRegex(ValueError, "missing submodule checkout"):
            layout.validate("HEAD", True)


if __name__ == "__main__":
    unittest.main()
