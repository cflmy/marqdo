"""Recursive impact analysis contract tests (ECO-02 / issue #7).

Builds real temporary Git repositories for two root candidates and asserts the
ordering required by the development contract: changed leaves, reverse
`requires` closure over the union graph, `checks_with` fixed point, then
parent domains. Undetermined scope widens to the full candidate.
"""

import importlib.util
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / "scripts" / "impact.py"
SPEC = importlib.util.spec_from_file_location("impact", SCRIPT)
impact = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(impact)

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
    return result.stdout.decode()


def init(path: Path) -> Path:
    path.mkdir(parents=True, exist_ok=True)
    git(path, "init", "-q")
    return path


def write(path: Path, data) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data if isinstance(data, bytes) else data.encode())


def commit(repo: Path, message: str = "fixture") -> str:
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", message)
    return git(repo, "rev-parse", "HEAD").strip()


def component(**fields) -> str:
    lines = ["[[component]]"]
    for key, value in fields.items():
        if isinstance(value, list):
            rendered = "[" + ", ".join(f'"{item}"' for item in value) + "]"
        else:
            rendered = f'"{value}"'
        lines.append(f"{key} = {rendered}")
    return "\n".join(lines)


def catalog(*components: str) -> str:
    return "\n\n".join(components) + "\n"


def leaf(base: Path, name: str, files: dict[str, str]) -> str:
    repo = init(base / name)
    for relative, data in files.items():
        write(repo / relative, data)
    return commit(repo)


def root_repo(
    path: Path, catalog_text: str, links: dict[str, str], urls: dict[str, str] | None = None
) -> str:
    init(path)
    write(path / "components.toml", catalog_text)
    if urls:
        modules = ["[submodule \"x\"]"]
        lines = []
        for link_path, url in urls.items():
            lines.extend([f"\tpath = {link_path}", f"\turl = {url}"])
        write(path / ".gitmodules", "\n".join(modules + lines) + "\n")
        git(path, "add", ".gitmodules")
    git(path, "add", "components.toml")
    for link_path, sha in links.items():
        git(path, "update-index", "--add", "--cacheinfo", f"160000,{sha},{link_path}")
    git(path, "commit", "-q", "-m", "root fixture")
    return git(path, "rev-parse", "HEAD").strip()


COMPONENTS_V1 = catalog(
    component(id="cli", path="cli", kind="core", state="active", owner="t", scope="s",
              requires=[], checks_with=[], release="base"),
    component(id="std-core", path="stdlib/core", kind="stdlib", state="active", owner="t",
              scope="s", requires=["cli"], checks_with=[], release="base"),
    component(id="std-text", path="stdlib/text", kind="stdlib", state="active", owner="t",
              scope="s", requires=["cli", "std-core"], checks_with=["probe"], release="base"),
    component(id="probe", path="tools/probe", kind="tool", state="active", owner="t",
              scope="s", requires=["cli"], checks_with=["std-text"], release="optional"),
    component(id="tool", path="tools/tool", kind="tool", state="active", owner="t",
              scope="s", requires=["cli"], checks_with=[], release="optional"),
)


class ImpactTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.base = Path(self.tmp.name)
        self.root = self.base / "root"
        self.original_root = impact.ROOT

    def tearDown(self):
        impact.ROOT = self.original_root
        self.tmp.cleanup()

    def make_tree(self, catalog_text: str, leaves: dict[str, str]) -> str:
        return root_repo(self.root, catalog_text, {path: leaves[name] for name, path in {
            "cli": "cli", "std-core": "stdlib/core", "std-text": "stdlib/text",
            "probe": "tools/probe", "tool": "tools/tool",
        }.items() if name in leaves})

    def leaf_shas(self):
        return {
            "cli": leaf(self.base, "cli", {"README.md": "cli"}),
            "std-core": leaf(self.base, "core", {"modules/core.mq.md": "# core\n"}),
            "std-text": leaf(self.base, "text", {"modules/text.mq.md": "# text\n"}),
            "probe": leaf(self.base, "probe", {"README.md": "probe"}),
            "tool": leaf(self.base, "tool", {"README.md": "tool"}),
        }

    def analyze(self, first: str, second: str):
        impact.ROOT = self.root
        return impact.analyze(first, second)

    def test_changed_leaf_closes_over_requires_then_checks_with(self):
        leaves = self.leaf_shas()
        first = self.make_tree(COMPONENTS_V1, leaves)
        leaves["std-core"] = leaf(self.base, "core2", {"modules/core.mq.md": "# core v2\n"})
        second = root_repo(
            self.root,
            COMPONENTS_V1,
            {
                "cli": leaves["cli"],
                "stdlib/core": leaves["std-core"],
                "stdlib/text": leaves["std-text"],
                "tools/probe": leaves["probe"],
                "tools/tool": leaves["tool"],
            },
        )
        report = self.analyze(first, second)
        self.assertEqual(
            [(c["id"], c["reason"]) for c in report["changed"]],
            [("std-core", "sha")],
        )
        self.assertEqual(report["affected"], ["probe", "std-core", "std-text"])
        self.assertEqual(report["checks_with"], ["probe"])
        self.assertNotIn("tool", report["affected"])

    def test_removed_component_and_its_checks_with_partner_are_affected(self):
        leaves = self.leaf_shas()
        first = self.make_tree(COMPONENTS_V1, leaves)
        without_text = catalog(
            component(id="cli", path="cli", kind="core", state="active", owner="t",
                      scope="s", requires=[], checks_with=[], release="base"),
            component(id="std-core", path="stdlib/core", kind="stdlib", state="active",
                      owner="t", scope="s", requires=["cli"], checks_with=[], release="base"),
            component(id="probe", path="tools/probe", kind="tool", state="active", owner="t",
                      scope="s", requires=["cli"], checks_with=["std-text"], release="optional"),
            component(id="tool", path="tools/tool", kind="tool", state="active", owner="t",
                      scope="s", requires=["cli"], checks_with=[], release="optional"),
        )
        second = root_repo(
            self.root,
            without_text,
            {
                "cli": leaves["cli"],
                "stdlib/core": leaves["std-core"],
                "tools/probe": leaves["probe"],
                "tools/tool": leaves["tool"],
            },
        )
        report = self.analyze(first, second)
        self.assertIn({"id": "std-text", "reason": "removed"}, report["changed"])
        self.assertIn("probe", report["affected"])

    def test_dependency_change_is_reported_and_widens_scope(self):
        leaves = self.leaf_shas()
        first = self.make_tree(COMPONENTS_V1, leaves)
        changed_text = COMPONENTS_V1.replace(
            'requires = ["cli", "std-core"]', 'requires = ["cli", "std-core", "tool"]'
        )
        second = self.make_tree(changed_text, leaves)
        report = self.analyze(first, second)
        self.assertEqual(
            [(c["id"], c["reason"]) for c in report["changed"]],
            [("std-text", "dependencies")],
        )
        self.assertIn("std-text", report["affected"])

    def test_removed_requires_edge_remains_in_the_impact_scope(self):
        leaves = self.leaf_shas()
        first = self.make_tree(COMPONENTS_V1, leaves)
        without_dependency = COMPONENTS_V1.replace(
            'requires = ["cli", "std-core"]', 'requires = ["cli"]'
        )
        second = self.make_tree(without_dependency, leaves)
        before, _ = impact.load_graph(self.root, first, "before")
        after, _ = impact.load_graph(self.root, second, "after")
        affected, _, partners = impact.affected_set(before, after, [{"id": "std-core"}])
        self.assertIn("std-text", affected)
        self.assertIn("probe", partners)

    def test_domain_gitlink_change_expands_to_child_identity(self):
        child = init(self.root / "extensions/web")
        write(child / "components.toml", catalog(
            component(id="web-design", path="design", kind="stdlib", state="planned",
                      owner="t", scope="s", requires=["cli"], checks_with=[], release="base"),
        ))
        design = leaf(self.base, "design", {"index.mq.md": "# design v1\n"})
        git(child, "add", "components.toml")
        git(child, "update-index", "--add", "--cacheinfo", f"160000,{design},design")
        git(child, "commit", "-q", "-m", "domain v1")
        domain_v1 = git(child, "rev-parse", "HEAD").strip()
        root_catalog = catalog(
            component(id="cli", path="cli", kind="core", state="active", owner="t",
                      scope="s", requires=[], checks_with=[], release="base"),
            component(id="web", path="extensions/web", kind="domain", state="planned",
                      owner="t", scope="s", requires=[], checks_with=[], release="none"),
        )
        first = root_repo(
            self.root,
            root_catalog,
            {"cli": leaf(self.base, "cli2", {"README.md": "cli"}), "extensions/web": domain_v1},
            {"extensions/web": "https://git.example.invalid/web.git"},
        )
        design_v2 = leaf(self.base, "design2", {"index.mq.md": "# design v2\n"})
        git(child, "update-index", "--cacheinfo", f"160000,{design_v2},design")
        git(child, "commit", "-q", "-m", "domain v2")
        domain_v2 = git(child, "rev-parse", "HEAD").strip()
        second = root_repo(
            self.root,
            root_catalog,
            {"cli": leaf(self.base, "cli3", {"README.md": "cli"}), "extensions/web": domain_v2},
            {"extensions/web": "https://git.example.invalid/web.git"},
        )
        report = self.analyze(first, second)
        self.assertIn("web", [c["id"] for c in report["changed"]])
        self.assertIn("web-design", [c["id"] for c in report["changed"]])
        self.assertIn("web-design", report["affected"])

    def test_unexpandable_domain_widens_to_full_scope(self):
        root_catalog = catalog(
            component(id="cli", path="cli", kind="core", state="active", owner="t",
                      scope="s", requires=[], checks_with=[], release="base"),
            component(id="web", path="extensions/web", kind="domain", state="planned",
                      owner="t", scope="s", requires=[], checks_with=[], release="none"),
        )
        cli = leaf(self.base, "cli4", {"README.md": "cli"})
        first = root_repo(
            self.root, root_catalog,
            {"cli": cli, "extensions/web": leaf(self.base, "web1", {"README.md": "web"})},
            {"extensions/web": "https://git.example.invalid/web.git"},
        )
        second = root_repo(
            self.root, root_catalog,
            {"cli": cli, "extensions/web": leaf(self.base, "web2", {"README.md": "web2"})},
            {"extensions/web": "https://git.example.invalid/web.git"},
        )
        # The root fixture points at a plain repository without a components.toml.
        report = self.analyze(first, second)
        self.assertTrue(report["unverified"])
        self.assertEqual(report["fallback"], "full")
        self.assertEqual(report["affected"], ["cli", "web"])


if __name__ == "__main__":
    unittest.main()
