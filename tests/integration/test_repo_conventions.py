"""Repository development conventions gate tests (docs/REPOSITORY-CONVENTIONS.md)."""

import importlib.util
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "repo_conventions", ROOT / "scripts" / "check-repo-conventions.py"
)
conventions = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(conventions)

MARKER = conventions.MARKER
URL = conventions.CONVENTIONS_URL


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


def agents(role: str) -> str:
    return f"""{MARKER}
# AGENTS.md — {role}

全局规范唯一来源：{URL}
本文件只写本仓边界，不复制全局规范或合同正文。

## 本仓定位

catalog 角色 `{role}`；只在本仓职责范围内修改，先开本仓工单。
本仓拥有自己的实现、测试与制品；不拥有其它仓的实现、合同或调度。

## 必做 / 禁止

- 必做：真实 CLI/制品验证、固定 SHA/摘要、证据记录、未运行写 UNVERIFIED。
- 禁止：复制 parser/实现/合同；依赖父仓或兄弟仓 checkout；第二 catalog/manifest；伪造 PASS；提交密钥。
- 禁止：自动 tag/release/合并；跨仓变更必须走 C2 → B2 → A2 晋升。

## 构建与证据

CI 入口非交互、失败非零、不打印秘密；证据含 SHA、制品摘要、平台、命令、退出码、日志。
"""


class RepoConventionsTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        write(self.root / "AGENTS.md", f"# 根\n\n防膨胀规则见 docs/REPOSITORY-CONVENTIONS.md\n")
        write(self.root / "docs/REPOSITORY-CONVENTIONS.md", "规则")
        write(self.root / ".gitmodules", "")
        write(
            self.root / "components.toml",
            """[[component]]
id = "cli"
path = "cli"
kind = "core"
state = "active"
owner = "t"
scope = "s"
requires = []
checks_with = []
release = "base"
entry = "scripts/ci.sh"
""",
        )
        write(self.root / "cli/README.md", "x" * 200)
        write(self.root / "cli/AGENTS.md", agents("cli"))
        write(self.root / "cli/scripts/ci.sh", "#!/bin/sh\n")
        write(
            self.root / ".gitmodules",
            '[submodule "cli"]\n\tpath = cli\n\turl = https://git.cflmy.cn/cflmy/marqdo-cli.git\n',
        )

    def tearDown(self):
        self.tmp.cleanup()

    def test_active_repository_with_required_files_passes(self):
        self.assertEqual(conventions.check(self.root), [])

    def test_missing_agents_or_marker_fails(self):
        (self.root / "cli/AGENTS.md").unlink()
        self.assertTrue(any("AGENTS.md missing" in item for item in conventions.check(self.root)))
        write(self.root / "cli/AGENTS.md", agents("cli").replace(MARKER, ""))
        self.assertTrue(
            any("lacks `<!-- marqdo:repo-conventions v1 -->`" in item for item in conventions.check(self.root))
        )

    def test_active_entry_and_readme_are_required(self):
        (self.root / "cli/README.md").unlink()
        (self.root / "cli/scripts/ci.sh").unlink()
        errors = conventions.check(self.root)
        self.assertTrue(any("README.md missing" in item for item in errors))
        self.assertTrue(any("entry file missing" in item for item in errors))

    def test_planned_component_must_not_declare_entry(self):
        text = (self.root / "components.toml").read_text().replace(
            'state = "active"', 'state = "planned"'
        )
        text = text.replace('entry = "scripts/ci.sh"', 'entry = "scripts/ci.sh"')
        write(self.root / "components.toml", text)
        self.assertTrue(
            any("planned component must not declare entry" in item for item in conventions.check(self.root))
        )

    def test_leaf_repository_must_not_own_second_catalog_or_submodules(self):
        write(self.root / "cli/components.toml", "[[component]]\n")
        write(self.root / "cli/.gitmodules", "")
        errors = conventions.check(self.root)
        self.assertTrue(any("leaf component must not own components.toml" in item for item in errors))
        self.assertTrue(any("leaf component must not own .gitmodules" in item for item in errors))

    def test_domain_child_must_not_be_registered_in_root_catalog(self):
        write(
            self.root / "components.toml",
            (self.root / "components.toml").read_text()
            + """
[[component]]
id = "web"
path = "extensions/web"
kind = "domain"
state = "active"
owner = "t"
scope = "d"
requires = []
checks_with = []
release = "none"
entry = "scripts/domain.sh"
""",
        )
        write(self.root / "extensions/web/README.md", "x" * 200)
        write(self.root / "extensions/web/AGENTS.md", agents("web"))
        write(self.root / "extensions/web/scripts/domain.sh", "#!/bin/sh\n")
        write(
            self.root / "extensions/web/components.toml",
            """[[component]]
id = "web-design"
path = "design"
kind = "extension"
state = "active"
owner = "t"
scope = "d"
requires = ["cli"]
checks_with = []
release = "optional"
entry = "scripts/ci.sh"
""",
        )
        write(self.root / "extensions/web/design/README.md", "x" * 200)
        write(self.root / "extensions/web/design/AGENTS.md", agents("web-design"))
        write(self.root / "extensions/web/design/scripts/ci.sh", "#!/bin/sh\n")
        write(
            self.root / ".gitmodules",
            (self.root / ".gitmodules").read_text()
            + '[submodule "web"]\n\tpath = extensions/web\n\turl = https://git.cflmy.cn/cflmy/marqdo-ext-web.git\n',
        )
        self.assertEqual(conventions.check(self.root), [])
        write(
            self.root / "components.toml",
            (self.root / "components.toml").read_text()
            + """
[[component]]
id = "web-design-dup"
path = "extensions/web/design"
kind = "extension"
state = "planned"
owner = "t"
scope = "d"
requires = ["cli"]
checks_with = []
release = "optional"
""",
        )
        self.assertTrue(
            any("also registered in the root catalog" in item for item in conventions.check(self.root))
        )


if __name__ == "__main__":
    unittest.main()
