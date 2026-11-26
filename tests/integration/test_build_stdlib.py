"""Contract tests for the root-owned, unique stdlib Bundle producer.

These tests build real temporary Git repositories so the producer is exercised
through its real identity path: root catalog + exact gitlinks + component module
trees. They never call a network remote and never use the repository's own
Bundles.
"""

import importlib.util
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / "scripts" / "build-stdlib.py"
SPEC = importlib.util.spec_from_file_location("build_stdlib", SCRIPT)
producer = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(producer)

GIT_ENV = {
    "GIT_AUTHOR_NAME": "test",
    "GIT_AUTHOR_EMAIL": "test@example.invalid",
    "GIT_COMMITTER_NAME": "test",
    "GIT_COMMITTER_EMAIL": "test@example.invalid",
}


def git(cwd, *args, check=True):
    env = os.environ.copy()
    env.update(GIT_ENV)
    result = subprocess.run(
        ["git", "-C", str(cwd), *args], capture_output=True, env=env
    )
    if check and result.returncode != 0:
        raise AssertionError(
            f"git {' '.join(args)} failed: {result.stderr.decode(errors='replace')}"
        )
    return result


def init_repo(path: Path) -> Path:
    path.mkdir(parents=True, exist_ok=True)
    git(path, "init", "-q")
    return path


def commit_all(repo: Path, message: str = "fixture") -> str:
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", message)
    return git(repo, "rev-parse", "HEAD").stdout.decode().strip()


def write(path: Path, data) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data if isinstance(data, bytes) else data.encode())


def component(repo: Path, files: dict) -> str:
    init_repo(repo)
    for relative, data in files.items():
        write(repo / relative, data)
    return commit_all(repo)


def catalog(*components: dict) -> str:
    blocks = []
    for component_spec in components:
        lines = ["[[component]]"]
        for key, value in component_spec.items():
            if isinstance(value, list):
                rendered = "[" + ", ".join(json.dumps(item) for item in value) + "]"
            else:
                rendered = json.dumps(value)
            lines.append(f"{key} = {rendered}")
        blocks.append("\n".join(lines))
    return "\n\n".join(blocks) + "\n"


def root_repo(path: Path, catalog_text: str, links: dict, config: dict) -> str:
    init_repo(path)
    write(path / "components.toml", catalog_text)
    write(path / "scripts/stdlib-bundle.json", json.dumps(config, indent=2))
    for relative, sha in links.items():
        git(path, "update-index", "--add", "--cacheinfo", f"160000,{sha},{relative}")
    git(path, "add", "components.toml", "scripts/stdlib-bundle.json")
    git(path, "commit", "-q", "-m", "root fixture")
    return git(path, "rev-parse", "HEAD").stdout.decode().strip()


def base_config(version="0.1.0", aliases=None) -> dict:
    return {
        "name": "marqdo-stdlib",
        "version": version,
        "public_aliases": aliases or [],
    }


def std_component(component_id: str, path: str, state="active") -> dict:
    return {
        "id": component_id,
        "path": path,
        "kind": "stdlib",
        "state": state,
        "owner": "test",
        "scope": "fixture",
        "requires": ["cli"],
        "checks_with": [],
        "release": "base",
    } | ({"entry": "scripts/test.sh"} if state == "active" else {})


class ProducerTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name) / "root"
        self.root.mkdir()

    def tearDown(self):
        self.tmp.cleanup()

    def build(self, *extra):
        import contextlib
        import io

        out = Path(self.tmp.name) / (
            "out-" + str(len(list(Path(self.tmp.name).glob("out-*"))))
        )
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            status = producer.main(
                ["--out", str(out), "--root", str(self.root), *extra]
            )
        self.assertEqual(status, 0)
        return buffer.getvalue(), out

    def make_two_components(self):
        text = component(
            self.root / "stdlib/text",
            {
                "modules/text.mq.md": "# @exec text\n",
                "modules/text/utf8.mq.md": "# @exec utf8\n",
                "modules/README.md": "not published\n",
            },
        )
        io = component(
            self.root / "stdlib/io",
            {
                "modules/io.mq.md": "# @exec io\n",
                "modules/io/console.mq.md": "# @exec console\n",
            },
        )
        return text, io

    def two_component_root(self, text_sha, io_sha, version="0.1.0", aliases=None):
        cat = catalog(
            std_component("std-text", "stdlib/text"),
            std_component("std-io", "stdlib/io"),
        )
        return root_repo(
            self.root,
            cat,
            {"stdlib/text": text_sha, "stdlib/io": io_sha},
            base_config(version, aliases),
        )

    def test_builds_deterministic_bundle_and_excludes_non_modules(self):
        text_sha, io_sha = self.make_two_components()
        self.two_component_root(text_sha, io_sha)
        summary_one, out_one = self.build()
        payload = json.loads(summary_one)
        self.assertEqual(payload["files"], [
            "io.mq.md",
            "io/console.mq.md",
            "text.mq.md",
            "text/utf8.mq.md",
        ])
        self.assertNotIn("README.md", payload["files"])
        manifest = json.loads((out_one / "stdlib.json").read_text())
        self.assertEqual(manifest["name"], "marqdo-stdlib")
        self.assertEqual(manifest["version"], "0.1.0")
        self.assertEqual(manifest["digest"], payload["tree_digest"])
        self.assertEqual(
            manifest["aliases_digest"], producer.aliases_digest([])
        )
        summary_two, out_two = self.build()
        self.assertEqual(
            json.loads(summary_two)["tree_digest"], payload["tree_digest"]
        )
        for relative in payload["files"]:
            self.assertEqual(
                (out_one / relative).read_bytes(), (out_two / relative).read_bytes()
            )
        self.assertEqual(
            (out_one / "stdlib.json").read_bytes(),
            (out_two / "stdlib.json").read_bytes(),
        )

    def test_worktree_mode_requires_clean_components(self):
        self.make_two_components()
        write(
            self.root / "components.toml",
            catalog(
                std_component("std-text", "stdlib/text"),
                std_component("std-io", "stdlib/io"),
            ),
        )
        write(
            self.root / "scripts/stdlib-bundle.json",
            json.dumps(base_config(), indent=2),
        )
        out = Path(self.tmp.name) / "w"
        producer.main(["--out", str(out), "--root", str(self.root), "--worktree"])
        self.assertTrue((out / "text.mq.md").is_file())
        write(self.root / "stdlib/text/modules/dirty.mq.md", "# @exec dirty\n")
        with self.assertRaises(SystemExit):
            producer.main([
                "--out", str(Path(self.tmp.name) / "w2"),
                "--root", str(self.root),
                "--worktree",
            ])

    def test_planned_component_is_not_published(self):
        text_sha, _ = self.make_two_components()
        planned_sha = component(
            self.root / "stdlib/net", {"modules/net/tcp.mq.md": "# @exec tcp\n"}
        )
        cat = catalog(
            std_component("std-text", "stdlib/text"),
            std_component("std-net", "stdlib/net", state="planned"),
        )
        root_repo(
            self.root,
            cat,
            {"stdlib/text": text_sha, "stdlib/net": planned_sha},
            base_config(),
        )
        summary, out = self.build()
        self.assertEqual(json.loads(summary)["files"], ["text.mq.md", "text/utf8.mq.md"])
        self.assertFalse((out / "net/tcp.mq.md").exists())

    def test_duplicate_and_case_colliding_paths_are_rejected(self):
        text_sha, _ = self.make_two_components()
        clash_sha = component(
            self.root / "stdlib/other", {"modules/Text.mq.md": "# @exec clash\n"}
        )
        cat = catalog(
            std_component("std-text", "stdlib/text"),
            std_component("std-other", "stdlib/other"),
        )
        root_repo(
            self.root,
            cat,
            {"stdlib/text": text_sha, "stdlib/other": clash_sha},
            base_config(),
        )
        with self.assertRaises(SystemExit):
            self.build()

    def test_non_nfc_and_symlink_entries_are_rejected(self):
        import unicodedata

        decomposed = unicodedata.normalize("NFD", "text/café.mq.md")
        text_sha = component(
            self.root / "stdlib/text",
            {"modules/text.mq.md": "# @exec text\n", f"modules/{decomposed}": "# x\n"},
        )
        io_sha = component(
            self.root / "stdlib/io", {"modules/io.mq.md": "# @exec io\n"}
        )
        self.two_component_root(text_sha, io_sha)
        with self.assertRaises(SystemExit):
            self.build()

        tmp = tempfile.TemporaryDirectory()
        try:
            root = Path(tmp.name) / "sr"
            root.mkdir()
            text = root / "stdlib/text"
            init_repo(text)
            write(text / "modules/text.mq.md", "# @exec text\n")
            (text / "modules/link.mq.md").symlink_to("text.mq.md")
            sha = commit_all(text)
            io = component(root / "stdlib/io", {"modules/io.mq.md": "# @exec io\n"})
            cat = catalog(
                std_component("std-text", "stdlib/text"),
                std_component("std-io", "stdlib/io"),
            )
            root_repo(root, cat, {"stdlib/text": sha, "stdlib/io": io}, base_config())
            with self.assertRaises(SystemExit):
                producer.main(["--out", str(root / "bundle"), "--root", str(root)])
        finally:
            tmp.cleanup()

    def test_active_component_without_modules_is_rejected(self):
        text_sha, io_sha = self.make_two_components()
        empty_sha = component(self.root / "stdlib/empty", {"README.md": "x\n"})
        cat = catalog(
            std_component("std-text", "stdlib/text"),
            std_component("std-io", "stdlib/io"),
            std_component("std-empty", "stdlib/empty"),
        )
        root_repo(
            self.root,
            cat,
            {
                "stdlib/text": text_sha,
                "stdlib/io": io_sha,
                "stdlib/empty": empty_sha,
            },
            base_config(),
        )
        with self.assertRaises(SystemExit):
            self.build()

    def test_content_change_requires_explicit_version_bump(self):
        text_sha, io_sha = self.make_two_components()
        self.two_component_root(text_sha, io_sha)
        _, out = self.build()
        baseline = out / "stdlib.json"
        changed = component(
            self.root / "stdlib/io",
            {
                "modules/io.mq.md": "# @exec io\n# changed\n",
                "modules/io/console.mq.md": "# @exec console\n",
            },
        )
        git(self.root, "reset", "-q", "--hard", "HEAD")
        self.two_component_root(text_sha, changed)
        with self.assertRaises(SystemExit):
            self.build("--baseline-manifest", str(baseline))
        new_root = self.two_component_root(text_sha, changed, version="0.2.0")
        self.assertIsNotNone(new_root)
        summary, out_two = self.build("--baseline-manifest", str(baseline))
        self.assertEqual(json.loads(summary)["version"], "0.2.0")

    def test_public_alias_must_target_a_published_module(self):
        text_sha, io_sha = self.make_two_components()
        alias = {
            "public_names": ["print", "打印"],
            "target_module": "io.mq.md",
            "target_function": "写值",
            "expected_signature": "params(值)",
            "effects": ["host"],
        }
        self.two_component_root(text_sha, io_sha, aliases=[alias])
        summary, out = self.build()
        manifest = json.loads((out / "stdlib.json").read_text())
        self.assertEqual(
            manifest["aliases_digest"], json.loads(summary)["aliases_digest"]
        )
        bad = dict(alias, target_module="missing.mq.md")
        git(self.root, "reset", "-q", "--hard", "HEAD")
        self.two_component_root(text_sha, io_sha, aliases=[bad])
        with self.assertRaises(SystemExit):
            self.build()

    def test_digest_algorithm_vector(self):
        import hashlib

        files = {"a.mq.md": b"content"}
        expected = "sha256:" + hashlib.sha256(b"a.mq.md\0content\0").hexdigest()
        self.assertEqual(producer.tree_digest(files), expected)


if __name__ == "__main__":
    unittest.main()
