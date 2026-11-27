import importlib.util
import shutil
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

CORE_URL = "https://git.cflmy.cn/cflmy/marqdo-cli.git"

WEB_DOMAIN = '''
[[component]]
id = "web"
path = "extensions/web"
kind = "domain"
state = "active"
owner = "maintainer"
scope = "web prototype domain governance"
requires = []
checks_with = []
release = "none"
entry = "scripts/integration.sh"
'''

STDLIB_DOMAIN = '''
[[component]]
id = "stdlib"
path = "stdlib"
kind = "domain"
state = "active"
owner = "maintainer"
scope = "standard library bundle assembly"
requires = []
checks_with = []
release = "base"
entry = "scripts/integration.sh"
'''

DOMAIN = '''
[[component]]
id = "web-design"
path = "design"
kind = "extension"
state = "active"
owner = "maintainer"
scope = "design document interpretation"
requires = ["cli"]
checks_with = []
release = "optional"
entry = "scripts/ci.sh"

[[component]]
id = "web-preview"
path = "preview"
kind = "tool"
state = "active"
owner = "maintainer"
scope = "prototype rendering"
requires = ["cli", "web-design"]
checks_with = []
release = "optional"
entry = "scripts/ci.sh"

[[component]]
id = "web-components"
path = "components"
kind = "extension"
state = "planned"
owner = "maintainer"
scope = "design templates"
requires = ["web-design"]
checks_with = ["web-preview"]
release = "optional"
'''

APP = '''
[[component]]
id = "app-harness"
path = "apps/harness"
kind = "app"
state = "planned"
owner = "maintainer"
scope = "official single-user agent application"
requires = ["cli"]
checks_with = []
release = "optional"
'''

APP_ACTIVE = APP.replace('state = "planned"', 'state = "active"').replace(
    'release = "optional"', 'release = "optional"\nentry = "scripts/ci.sh"')

APP_URL = "https://git.cflmy.cn/cflmy/marqdo-app-harness.git"


def catalog(*blocks):
    return "".join(blocks)


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

    def candidate(self, metadata=CLI, url=CORE_URL, extra=None, file=None, extra_url=None):
        (self.root / "components.toml").write_text(metadata)
        modules = f'[submodule "cli"]\npath = cli\nurl = {url}\n'
        if extra_url:
            modules += f'[submodule "{extra}"]\npath = {extra}\nurl = {extra_url}\n'
        (self.root / ".gitmodules").write_text(modules)
        self.git("add", ".gitmodules", "components.toml")
        self.git("update-index", "--add", "--cacheinfo", f"160000,{self.sha},cli")
        if extra:
            self.git("update-index", "--add", "--cacheinfo", f"160000,{self.sha},{extra}")
        if file:
            path = self.root / file
            path.parent.mkdir(parents=True)
            path.write_text("Fixture")
            self.git("add", file)
        self.git("-c", "user.name=Test", "-c", "user.email=test@example.invalid", "commit", "-qm", "candidate")

    def test_accepts_active_gitlink_and_planned_component(self):
        self.candidate(CLI + PLANNED)
        layout.validate("HEAD", False)

    def test_accepts_a_planned_component_directory_readme(self):
        self.candidate(CLI + PLANNED, file="stdlib/core/README.md")
        layout.validate("HEAD", False)

    def test_rejects_source_under_a_planned_component(self):
        self.candidate(CLI + PLANNED, file="stdlib/core/result.mq.md")
        with self.assertRaisesRegex(ValueError, "unexpected tracked path or mode"):
            layout.validate("HEAD", False)

    def test_rejects_an_unregistered_component_directory(self):
        self.candidate(CLI + PLANNED, file="stdlib/missing/README.md")
        with self.assertRaisesRegex(ValueError, "unexpected tracked path or mode"):
            layout.validate("HEAD", False)

    def test_rejects_a_planned_gitlink_without_a_registered_remote(self):
        self.candidate(CLI + PLANNED, extra="stdlib/core")
        with self.assertRaisesRegex(ValueError, "missing or invalid submodule path: stdlib/core"):
            layout.validate("HEAD", False)

    def test_accepts_a_planned_gitlink_with_its_remote(self):
        self.candidate(CLI + PLANNED, extra="stdlib/core",
                       extra_url="https://git.cflmy.cn/cflmy/marqdo-std-core.git")
        root = layout.validate("HEAD", False)[0]
        self.assertEqual(root.linked["stdlib/core"]["state"], "planned")

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

    def root_fixture(self, metadata, modules, links, files=None):
        (self.root / "components.toml").write_text(metadata)
        if 'id = "cli"' in metadata and "path = cli" not in modules:
            modules += f'[submodule "cli"]\npath = cli\nurl = {CORE_URL}\n'
        if 'id = "cli"' in metadata and "cli" not in links:
            links = {**links, "cli": self.sha}
        (self.root / ".gitmodules").write_text(modules)
        for relative, content in (files or {}).items():
            path = self.root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content)
        self.git("add", "-A")
        for path, sha in links.items():
            self.git("update-index", "--add", "--cacheinfo", f"160000,{sha},{path}")
        self.git("-c", "user.name=Test", "-c", "user.email=test@example.invalid",
                 "commit", "-qm", "app fixture")

    def test_accepts_a_planned_app_gitlink(self):
        self.root_fixture(
            CLI + APP,
            f'[submodule "cli"]\npath = cli\nurl = {CORE_URL}\n'
            f'[submodule "apps/harness"]\npath = apps/harness\nurl = {APP_URL}\n',
            {"cli": self.sha, "apps/harness": self.sha})
        root = layout.validate("HEAD", False)[0]
        self.assertEqual(root.linked["apps/harness"]["kind"], "app")
        self.assertEqual(root.linked["apps/harness"]["state"], "planned")

    def test_rejects_an_app_outside_the_apps_path(self):
        self.root_fixture(
            CLI + APP.replace('path = "apps/harness"', 'path = "apps/other"'),
            f'[submodule "other"]\npath = apps/other\nurl = {APP_URL}\n',
            {"apps/other": self.sha})
        with self.assertRaisesRegex(ValueError, "invalid path: app-harness: apps/other"):
            layout.validate("HEAD", False)

    def test_rejects_an_app_id_without_the_app_prefix(self):
        self.root_fixture(
            CLI + APP.replace('id = "app-harness"', 'id = "harness"'),
            f'[submodule "harness"]\npath = apps/harness\nurl = {APP_URL}\n',
            {"apps/harness": self.sha})
        with self.assertRaisesRegex(ValueError, "app id must start with 'app-'"):
            layout.validate("HEAD", False)

    def test_rejects_a_wrong_app_remote(self):
        self.root_fixture(
            CLI + APP,
            f'[submodule "harness"]\npath = apps/harness\nurl = '
            f'https://git.cflmy.cn/cflmy/marqdo-harness.git\n',
            {"apps/harness": self.sha})
        with self.assertRaisesRegex(ValueError, "invalid submodule URL: apps/harness"):
            layout.validate("HEAD", False)

    def test_rejects_an_app_with_a_base_release_role(self):
        self.root_fixture(
            CLI + APP.replace('release = "optional"', 'release = "base"'),
            f'[submodule "harness"]\npath = apps/harness\nurl = {APP_URL}\n',
            {"apps/harness": self.sha})
        with self.assertRaisesRegex(ValueError, "invalid release role: app-harness"):
            layout.validate("HEAD", False)

    def test_rejects_an_app_dependency_on_a_domain(self):
        domain = WEB_DOMAIN.replace('state = "active"', 'state = "planned"').replace(
            'entry = "scripts/integration.sh"\n', '')
        self.root_fixture(
            CLI + domain + APP.replace('requires = ["cli"]', 'requires = ["cli", "web"]'),
            f'[submodule "harness"]\npath = apps/harness\nurl = {APP_URL}\n',
            {"apps/harness": self.sha})
        with self.assertRaisesRegex(ValueError, "requires must not target a domain: app-harness -> web"):
            layout.validate("HEAD", False)

    def test_rejects_an_app_dependency_on_another_app(self):
        second = APP.replace('id = "app-harness"', 'id = "app-other"').replace(
            'path = "apps/harness"', 'path = "apps/other"').replace(
            'requires = ["cli"]', 'requires = ["cli", "app-harness"]')
        self.root_fixture(
            CLI + APP + second,
            f'[submodule "harness"]\npath = apps/harness\nurl = {APP_URL}\n'
            f'[submodule "other"]\npath = apps/other\nurl = '
            f'https://git.cflmy.cn/cflmy/marqdo-app-other.git\n',
            {"apps/harness": self.sha, "apps/other": self.sha})
        with self.assertRaisesRegex(ValueError, "app components cannot be dependencies: app-other -> app-harness"):
            layout.validate("HEAD", False)

    def test_rejects_a_component_dependency_on_an_app(self):
        view = '''
[[component]]
id = "view"
path = "tools/view"
kind = "tool"
state = "active"
owner = "maintainer"
scope = "read and observe"
requires = ["cli", "app-harness"]
checks_with = []
release = "optional"
entry = "scripts/acceptance.sh"
'''
        self.root_fixture(
            CLI + APP_ACTIVE + view,
            f'[submodule "cli"]\npath = cli\nurl = {CORE_URL}\n'
            f'[submodule "harness"]\npath = apps/harness\nurl = {APP_URL}\n'
            f'[submodule "view"]\npath = tools/view\nurl = https://git.cflmy.cn/cflmy/marqdo-view.git\n',
            {"cli": self.sha, "apps/harness": self.sha, "tools/view": self.sha})
        with self.assertRaisesRegex(ValueError, "app components cannot be dependencies: view -> app-harness"):
            layout.validate("HEAD", False)

    def test_rejects_an_app_inside_a_domain(self):
        nested = {"id": "app-nested", "path": "harness", "kind": "app", "state": "planned",
                  "owner": "maintainer", "scope": "nested", "requires": [], "checks_with": [],
                  "release": "optional"}
        with self.assertRaisesRegex(ValueError, "app kind is only allowed at the root layer"):
            layout.check_component(nested, 1, "fixture domain", {}, set(), set())

    def test_rejects_unstaged_checkout(self):
        self.candidate()
        with self.assertRaisesRegex(ValueError, "missing submodule checkout"):
            layout.validate("HEAD", True)


class HierarchicalLayoutTests(unittest.TestCase):
    """End-to-end fixtures with real local Git repositories.

    The fixture builds root -> domain -> two leaves using only local clones:
    no network remote is contacted, no ``git submodule update`` is executed.
    """

    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        sources = tempfile.TemporaryDirectory()
        self.addCleanup(sources.cleanup)
        self.sources = Path(sources.name)
        self.enterContext(patch.object(layout, "ROOT", self.root))
        self.enterContext(patch.object(layout, "git", side_effect=self.git))
        self.git("init", "-q", "-b", "main")
        self.git("commit", "--allow-empty", "-qm", "fixture")

    def git(self, *args, cwd=None):
        return subprocess.check_output([
            "git",
            "-c", "user.name=Test",
            "-c", "user.email=test@example.invalid",
            "-C", str(cwd or self.root),
            *args,
        ], stderr=subprocess.DEVNULL).decode()

    # -- fixture primitives -------------------------------------------------

    def repo(self, name):
        path = self.sources / name
        path.mkdir(parents=True)
        self.git("init", "-q", "-b", "main", cwd=path)
        return path

    def write(self, repo, relative, content):
        path = Path(repo) / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)

    def stage(self, repo):
        self.git("add", "-A", cwd=repo)

    def commit(self, repo, message):
        self.git("commit", "-qm", message, cwd=repo)
        return self.git("rev-parse", "HEAD", cwd=repo).strip()

    def gitlink(self, repo, path, sha):
        self.git("update-index", "--add", "--cacheinfo", f"160000,{sha},{path}", cwd=repo)

    def modules(self, repo, entries):
        self.write(repo, ".gitmodules", "".join(
            f'[submodule "{path}"]\n\tpath = {path}\n\turl = {url}\n' for path, url in entries.items()))

    def leaf(self, name, entry="scripts/ci.sh", symlink_entry=False):
        repo = self.repo(name)
        if symlink_entry:
            self.write(repo, "scripts/real.sh", "#!/bin/sh\nexit 0\n")
            link = repo / entry
            link.parent.mkdir(parents=True, exist_ok=True)
            link.symlink_to("real.sh")
        else:
            self.write(repo, entry, "#!/bin/sh\nexit 0\n")
        self.stage(repo)
        return repo, self.commit(repo, f"leaf {name}")

    def leaf_with_gitlink(self, name):
        inner = self.repo(f"{name}-inner")
        self.write(inner, "scripts/ci.sh", "#!/bin/sh\nexit 0\n")
        self.stage(inner)
        inner_sha = self.commit(inner, "inner")
        repo = self.repo(name)
        self.write(repo, "scripts/ci.sh", "#!/bin/sh\nexit 0\n")
        self.stage(repo)
        self.gitlink(repo, "nested/inner", inner_sha)
        return repo, self.commit(repo, f"leaf {name}")

    def domain(self, catalog_text, links, files=None, modules=None):
        repo = self.repo("domain-source")
        if catalog_text is not None:
            self.write(repo, "components.toml", catalog_text)
        self.write(repo, "scripts/integration.sh", "#!/bin/sh\nexit 0\n")
        for relative, content in (files or {}).items():
            self.write(repo, relative, content)
        entries = {
            "design": "https://git.cflmy.cn/cflmy/marqdo-web-design.git",
            "preview": "https://git.cflmy.cn/cflmy/marqdo-web-preview.git",
        }
        if modules is not None:
            entries = modules
        self.modules(repo, entries)
        self.stage(repo)
        for path, sha in links.items():
            self.gitlink(repo, path, sha)
        return repo, self.commit(repo, "domain")

    def root_repo(self, catalog_text, links, modules):
        self.write(self.root, "components.toml", catalog_text)
        self.modules(self.root, modules)
        self.stage(self.root)
        for path, sha in links.items():
            self.gitlink(self.root, path, sha)
        return self.commit(self.root, "root")

    def clone(self, source, target, sha):
        target.parent.mkdir(parents=True, exist_ok=True)
        self.git("clone", "-q", str(source), str(target), cwd=self.sources)
        self.git("checkout", "-q", sha, cwd=target)

    def assemble(self, domain_catalog=DOMAIN, domain_files=None, domain_modules=None,
                 omit_domain_link=None, extra_domain_link=None, root_catalog=None,
                 root_modules=None, checkout=True, core=None, design=None, preview=None):
        core = core or self.leaf("core", entry="scripts/test.sh")
        design = design or self.leaf("design")
        preview = preview or self.leaf("preview")
        links = {"design": design[1], "preview": preview[1]}
        if omit_domain_link:
            links.pop(omit_domain_link)
        if extra_domain_link:
            links[extra_domain_link[0]] = extra_domain_link[1]
        domain_repo, domain_sha = self.domain(domain_catalog, links, domain_files, domain_modules)
        modules = root_modules if root_modules is not None else {
            "cli": CORE_URL,
            "extensions/web": "https://git.cflmy.cn/cflmy/marqdo-ext-web.git",
        }
        self.root_repo(root_catalog if root_catalog is not None else catalog(CLI, WEB_DOMAIN),
                       {"cli": core[1], "extensions/web": domain_sha}, modules)
        if checkout:
            self.clone(core[0], self.root / "cli", core[1])
            self.clone(domain_repo, self.root / "extensions/web", domain_sha)
            sources = {"design": design, "preview": preview}
            for path, sha in links.items():
                self.clone(sources.get(path, design)[0], self.root / "extensions/web" / path, sha)
        return {"core": core, "design": design, "preview": preview, "domain": (domain_repo, domain_sha)}

    # -- positive cases -----------------------------------------------------

    def test_accepts_a_two_level_domain_tree_with_checkouts(self):
        self.assemble(domain_files={"docs/WEB-DESIGN.md": "# contract\n", "examples/research/README.md": "example\n"})
        layout.validate("HEAD", True)
        layout.validate("HEAD", False)

    def test_active_entries_use_the_dependency_order_and_include_a_new_leaf(self):
        design = self.leaf("design")
        catalog_text = DOMAIN.replace('state = "planned"', 'state = "active"').replace(
            'checks_with = ["web-preview"]\nrelease = "optional"',
            'checks_with = ["web-preview"]\nrelease = "optional"\nentry = "scripts/ci.sh"',
        )
        self.assemble(domain_catalog=catalog_text, design=design,
                      extra_domain_link=("components", design[1]), domain_modules={
                          "design": "https://git.cflmy.cn/cflmy/marqdo-web-design.git",
                          "preview": "https://git.cflmy.cn/cflmy/marqdo-web-preview.git",
                          "components": "https://git.cflmy.cn/cflmy/marqdo-web-components.git",
                      })
        layers = layout.validate("HEAD", True)
        entries = list(layout.active_entries(layers))
        ids = [entry[0] for entry in entries]
        self.assertIn(("web-components", "extension", "extensions/web/components", "scripts/ci.sh"), entries)
        self.assertLess(ids.index("cli"), ids.index("web-design"))
        self.assertLess(ids.index("web-design"), ids.index("web-preview"))
        self.assertLess(ids.index("web-components"), ids.index("web"))
        self.assertEqual(set(ids), {"cli", "web", "web-design", "web-preview", "web-components"})

    def test_accepts_a_planned_domain_and_planned_child_gitlinks(self):
        web = WEB_DOMAIN.replace('state = "active"', 'state = "planned"').replace(
            'entry = "scripts/integration.sh"\n', '')
        children = DOMAIN.replace('state = "active"', 'state = "planned"').replace(
            'entry = "scripts/ci.sh"\n', '')
        self.assemble(root_catalog=CLI + web, domain_catalog=children)
        root, domain = layout.validate("HEAD", True)
        self.assertEqual(root.linked["extensions/web"]["state"], "planned")
        self.assertEqual(set(domain.linked), {"design", "preview"})
        self.assertTrue(all(child["state"] == "planned" for child in domain.linked.values()))

    def test_non_checkout_mode_only_validates_the_root_layer(self):
        self.assemble()
        shutil.rmtree(self.root / "extensions" / "web")
        layout.validate("HEAD", False)
        with self.assertRaisesRegex(ValueError, "missing submodule checkout: extensions/web"):
            layout.validate("HEAD", True)

    def test_accepts_a_stdlib_domain_with_base_release(self):
        self.root_repo(catalog(CLI, STDLIB_DOMAIN),
                       {"cli": self.leaf("core", entry="scripts/test.sh")[1], "stdlib": "a" * 40},
                       {"cli": CORE_URL, "stdlib": "https://git.cflmy.cn/cflmy/marqdo-stdlib.git"})
        layout.validate("HEAD", False)

    def test_allows_checks_with_cycles(self):
        cycle = CLI.replace('checks_with = []', 'checks_with = ["view"]') + '''
[[component]]
id = "view"
path = "tools/view"
kind = "tool"
state = "active"
owner = "maintainer"
scope = "read and observe"
requires = ["cli"]
checks_with = ["cli"]
release = "optional"
entry = "scripts/acceptance.sh"
'''
        sha = self.leaf("core", entry="scripts/test.sh")[1]
        self.root_repo(cycle, {"cli": sha, "tools/view": sha},
                       {"cli": CORE_URL, "tools/view": "https://git.cflmy.cn/cflmy/marqdo-view.git"})
        layout.validate("HEAD", False)

    # -- checkout failures --------------------------------------------------

    def test_rejects_a_dirty_domain_child(self):
        self.assemble()
        (self.root / "extensions/web/design/scratch.txt").write_text("dirty")
        with self.assertRaisesRegex(ValueError, r"domain web \(extensions/web\): dirty submodule: design"):
            layout.validate("HEAD", True)

    def test_rejects_a_wrong_checked_out_commit(self):
        self.assemble()
        self.git("commit", "--allow-empty", "-qm", "drift", cwd=self.root / "extensions/web/design")
        with self.assertRaisesRegex(ValueError, r"domain web \(extensions/web\): wrong checked-out commit: design"):
            layout.validate("HEAD", True)

    def test_rejects_a_missing_domain_child_checkout(self):
        self.assemble()
        shutil.rmtree(self.root / "extensions/web/preview")
        with self.assertRaisesRegex(ValueError, r"domain web \(extensions/web\): missing submodule checkout: preview"):
            layout.validate("HEAD", True)

    def test_rejects_a_missing_committed_object(self):
        self.assemble()
        self.git("update-index", "--add", "--cacheinfo", f"160000,{'f' * 40},extensions/web", cwd=self.root)
        self.git("commit", "-qm", "bad gitlink")
        with self.assertRaisesRegex(ValueError, "missing committed object: extensions/web"):
            layout.validate("HEAD", True)

    def test_rejects_a_symbolic_entry(self):
        design = self.leaf("design", symlink_entry=True)
        self.assemble(design=design)
        with self.assertRaisesRegex(ValueError, "missing or symbolic entry: design/scripts/ci.sh"):
            layout.validate("HEAD", True)

    def test_rejects_a_third_level_gitlink(self):
        design = self.leaf_with_gitlink("design")
        self.assemble(design=design)
        with self.assertRaisesRegex(ValueError, r"domain web \(extensions/web\) leaf web-design \(design\): leaf must not register gitlinks"):
            layout.validate("HEAD", True)

    def test_accepts_an_active_app_leaf_checkout(self):
        core = self.leaf("core", entry="scripts/test.sh")
        app = self.leaf("harness")
        self.root_repo(catalog(CLI, APP_ACTIVE), {"cli": core[1], "apps/harness": app[1]},
                       {"cli": CORE_URL, "apps/harness": APP_URL})
        self.clone(core[0], self.root / "cli", core[1])
        self.clone(app[0], self.root / "apps/harness", app[1])
        layers = layout.validate("HEAD", True)
        self.assertEqual(layers[0].linked["apps/harness"]["kind"], "app")
        entries = list(layout.active_entries(layers))
        self.assertIn(("app-harness", "app", "apps/harness", "scripts/ci.sh"), entries)
        ids = [entry[0] for entry in entries]
        self.assertLess(ids.index("cli"), ids.index("app-harness"))

    def test_rejects_an_app_with_a_nested_gitlink(self):
        core = self.leaf("core", entry="scripts/test.sh")
        app = self.leaf_with_gitlink("harness")
        self.root_repo(catalog(CLI, APP_ACTIVE), {"cli": core[1], "apps/harness": app[1]},
                       {"cli": CORE_URL, "apps/harness": APP_URL})
        self.clone(core[0], self.root / "cli", core[1])
        self.clone(app[0], self.root / "apps/harness", app[1])
        with self.assertRaisesRegex(
                ValueError, r"leaf app-harness \(apps/harness\): leaf must not register gitlinks"):
            layout.validate("HEAD", True)

    def test_rejects_an_app_registered_by_a_domain_catalog(self):
        nested = '''
[[component]]
id = "app-nested"
path = "nested-app"
kind = "app"
state = "planned"
owner = "maintainer"
scope = "not allowed inside a domain"
requires = []
checks_with = []
release = "optional"
'''
        core = self.leaf("core", entry="scripts/test.sh")
        design = self.leaf("design")
        preview = self.leaf("preview")
        nested_leaf = self.leaf("nested-app")
        domain_repo, domain_sha = self.domain(
            DOMAIN + nested,
            {"design": design[1], "preview": preview[1], "nested-app": nested_leaf[1]},
            modules={
                "design": "https://git.cflmy.cn/cflmy/marqdo-web-design.git",
                "preview": "https://git.cflmy.cn/cflmy/marqdo-web-preview.git",
                "nested-app": APP_URL,
            })
        self.root_repo(catalog(CLI, WEB_DOMAIN), {"cli": core[1], "extensions/web": domain_sha},
                       {"cli": CORE_URL,
                        "extensions/web": "https://git.cflmy.cn/cflmy/marqdo-ext-web.git"})
        self.clone(domain_repo, self.root / "extensions/web", domain_sha)
        with self.assertRaisesRegex(ValueError, "app kind is only allowed at the root layer: app-nested"):
            layout.validate("HEAD", False, expand_domains=True)

    # -- catalog and layer failures ----------------------------------------

    def test_rejects_a_missing_domain_catalog(self):
        self.assemble(domain_catalog=None)
        layout.validate("HEAD", False)
        with self.assertRaisesRegex(ValueError, "missing components.toml at"):
            layout.validate("HEAD", True)

    def test_rejects_a_domain_catalog_missing_a_gitlink(self):
        extra = DOMAIN + '''
[[component]]
id = "web-extra"
path = "extra"
kind = "extension"
state = "active"
owner = "maintainer"
scope = "extra component"
requires = ["cli"]
checks_with = []
release = "optional"
entry = "scripts/ci.sh"
'''
        self.assemble(domain_catalog=extra)
        with self.assertRaisesRegex(ValueError, r"domain web \(extensions/web\): gitlink/catalog mismatch"):
            layout.validate("HEAD", True)

    def test_rejects_an_unregistered_domain_gitlink(self):
        info = self.assemble()
        extra = self.leaf("extra")
        domain_repo = self.sources / "domain-source"
        self.git("update-index", "--add", "--cacheinfo", f"160000,{extra[1]},extra", cwd=domain_repo)
        new_domain_sha = self.commit(domain_repo, "extra gitlink")
        self.git("update-index", "--add", "--cacheinfo", f"160000,{new_domain_sha},extensions/web", cwd=self.root)
        self.git("commit", "-qm", "advance domain")
        shutil.rmtree(self.root / "extensions/web")
        self.clone(domain_repo, self.root / "extensions/web", new_domain_sha)
        for name in ("design", "preview"):
            self.clone(info[name][0], self.root / "extensions/web" / name, info[name][1])
        self.clone(extra[0], self.root / "extensions/web" / "extra", extra[1])
        with self.assertRaisesRegex(ValueError, r"domain web \(extensions/web\): gitlink/catalog mismatch"):
            layout.validate("HEAD", True)

    def test_rejects_a_nested_domain(self):
        nested = '''
[[component]]
id = "web-inner"
path = "inner"
kind = "domain"
state = "planned"
owner = "maintainer"
scope = "nested aggregate"
requires = []
checks_with = []
release = "none"
'''
        self.assemble(domain_catalog=nested)
        with self.assertRaisesRegex(ValueError, "domain kind is only allowed at the root layer: web-inner"):
            layout.validate("HEAD", True)

    def test_rejects_a_core_component_inside_a_domain(self):
        nested = '''
[[component]]
id = "web-core"
path = "core"
kind = "core"
state = "planned"
owner = "maintainer"
scope = "not allowed here"
requires = []
checks_with = []
release = "base"
'''
        self.assemble(domain_catalog=nested)
        with self.assertRaisesRegex(ValueError, "component kind is not allowed inside a domain: web-core"):
            layout.validate("HEAD", True)

    def test_rejects_a_production_file_in_the_domain_tree(self):
        self.assemble(domain_files={"Cargo.toml": '[package]\nname = "web-domain"\n'})
        with self.assertRaisesRegex(ValueError, "unexpected aggregate tracked path or mode: Cargo.toml"):
            layout.validate("HEAD", True)

    def test_rejects_a_planned_child_implementation_file(self):
        self.assemble(domain_files={"components/renderer.py": "print('not allowed')\n"})
        with self.assertRaisesRegex(ValueError, "unexpected aggregate tracked path or mode: components/renderer.py"):
            layout.validate("HEAD", True)

    def test_rejects_a_wrong_domain_child_url(self):
        self.assemble(domain_modules={
            "design": "https://git.cflmy.cn/cflmy/marqdo-ext-design.git",
            "preview": "https://git.cflmy.cn/cflmy/marqdo-web-preview.git",
        })
        shutil.rmtree(self.root / "extensions/web/design")
        with self.assertRaisesRegex(ValueError, "invalid submodule URL: design"):
            layout.validate("HEAD", False, expand_domains=True)

    # -- global graph failures ---------------------------------------------

    def test_rejects_a_duplicate_global_id(self):
        self.assemble(domain_catalog=DOMAIN.replace('id = "web-components"', 'id = "cli"'))
        with self.assertRaisesRegex(ValueError, "duplicate global id: cli"):
            layout.validate("HEAD", True)

    def test_rejects_an_unknown_domain_dependency(self):
        self.assemble(domain_catalog=DOMAIN.replace('requires = ["cli"]', 'requires = ["cli", "missing"]', 1))
        with self.assertRaisesRegex(ValueError, "unknown dependency: web-design -> missing"):
            layout.validate("HEAD", True)

    def test_rejects_an_active_domain_child_dependency_on_planned(self):
        self.assemble(domain_catalog=DOMAIN.replace(
            'requires = ["cli", "web-design"]', 'requires = ["cli", "web-design", "web-components"]', 1))
        with self.assertRaisesRegex(ValueError, "active depends on planned: web-preview -> web-components"):
            layout.validate("HEAD", True)

    # -- root catalog rules -------------------------------------------------

    def test_rejects_domain_requires(self):
        self.root_repo(CLI + WEB_DOMAIN.replace("requires = []", 'requires = ["cli"]'),
                       {"cli": "a" * 40, "extensions/web": "b" * 40},
                       {"cli": CORE_URL, "extensions/web": "https://git.cflmy.cn/cflmy/marqdo-ext-web.git"})
        with self.assertRaisesRegex(ValueError, "domain requires must be empty: web"):
            layout.validate("HEAD", False)

    def test_rejects_an_invalid_domain_release_role(self):
        self.root_repo(CLI + WEB_DOMAIN.replace('release = "none"', 'release = "optional"'),
                       {"cli": "a" * 40, "extensions/web": "b" * 40},
                       {"cli": CORE_URL, "extensions/web": "https://git.cflmy.cn/cflmy/marqdo-ext-web.git"})
        with self.assertRaisesRegex(ValueError, "invalid release role: web"):
            layout.validate("HEAD", False)

    def test_rejects_an_invalid_domain_path(self):
        self.root_repo(CLI + WEB_DOMAIN.replace('path = "extensions/web"', 'path = "extensions/other"'),
                       {"cli": "a" * 40, "extensions/other": "b" * 40},
                       {"cli": CORE_URL, "extensions/other": "https://git.cflmy.cn/cflmy/marqdo-ext-web.git"})
        with self.assertRaisesRegex(ValueError, "invalid path: web: extensions/other"):
            layout.validate("HEAD", False)

    def test_rejects_a_requires_edge_to_a_domain(self):
        view = '''
[[component]]
id = "view"
path = "tools/view"
kind = "tool"
state = "active"
owner = "maintainer"
scope = "read and observe"
requires = ["web"]
checks_with = []
release = "optional"
entry = "scripts/acceptance.sh"
'''
        self.root_repo(CLI + WEB_DOMAIN + view,
                       {"cli": "a" * 40, "extensions/web": "b" * 40, "tools/view": "a" * 40},
                       {"cli": CORE_URL,
                        "extensions/web": "https://git.cflmy.cn/cflmy/marqdo-ext-web.git",
                        "tools/view": "https://git.cflmy.cn/cflmy/marqdo-view.git"})
        with self.assertRaisesRegex(ValueError, "requires must not target a domain: view -> web"):
            layout.validate("HEAD", False)

    def test_rejects_stdlib_dependency_on_extension(self):
        stdlib_extra = '''
[[component]]
id = "std-extra"
path = "stdlib/extra"
kind = "stdlib"
state = "planned"
owner = "maintainer"
scope = "not allowed to depend on extensions"
requires = ["math"]
checks_with = []
release = "base"
'''
        math = '''
[[component]]
id = "math"
path = "extensions/math"
kind = "extension"
state = "planned"
owner = "maintainer"
scope = "numeric algorithms"
requires = ["cli"]
checks_with = []
release = "optional"
'''
        self.root_repo(CLI + stdlib_extra + math, {"cli": "a" * 40}, {"cli": CORE_URL})
        with self.assertRaisesRegex(ValueError, "stdlib dependency violation: std-extra -> math"):
            layout.validate("HEAD", False)

    def test_rejects_core_build_dependency_on_other_component(self):
        conformance = '''
[[component]]
id = "conformance"
path = "tests/conformance"
kind = "test"
state = "active"
owner = "maintainer"
scope = "independent protocol tests"
requires = []
checks_with = []
release = "none"
entry = "scripts/build_all.sh"
'''
        cli = CLI.replace("requires = []", 'requires = ["conformance"]')
        self.root_repo(cli + conformance,
                       {"cli": "a" * 40, "tests/conformance": "a" * 40},
                       {"cli": CORE_URL,
                        "tests/conformance": "https://git.cflmy.cn/cflmy/marqdo-extension-conformance.git"})
        with self.assertRaisesRegex(ValueError, "dependency violation: cli must not require conformance"):
            layout.validate("HEAD", False)

    def test_rejects_a_requires_cycle(self):
        math = '''
[[component]]
id = "math"
path = "extensions/math"
kind = "extension"
state = "planned"
owner = "maintainer"
scope = "numeric algorithms"
requires = ["linalg"]
checks_with = []
release = "optional"
'''
        linalg = '''
[[component]]
id = "linalg"
path = "extensions/linalg"
kind = "extension"
state = "planned"
owner = "maintainer"
scope = "linear algebra"
requires = ["math"]
checks_with = []
release = "optional"
'''
        self.root_repo(CLI + math + linalg, {"cli": "a" * 40}, {"cli": CORE_URL})
        with self.assertRaisesRegex(ValueError, "requires DAG cycle"):
            layout.validate("HEAD", False)

    def test_rejects_overlapping_component_paths(self):
        stdlib_domain = STDLIB_DOMAIN.replace("state = \"active\"", "state = \"planned\"").replace(
            'entry = "scripts/integration.sh"\n', "")
        planned = PLANNED
        self.root_repo(CLI + stdlib_domain + planned, {"cli": "a" * 40}, {"cli": CORE_URL})
        with self.assertRaisesRegex(ValueError, "overlapping component path: stdlib/core / stdlib"):
            layout.validate("HEAD", False)

    def test_rejects_a_submodule_update_command(self):
        self.root_repo(CLI, {"cli": "a" * 40}, {"cli": CORE_URL})
        self.write(self.root, ".gitmodules",
                   f'[submodule "cli"]\n\tpath = cli\n\turl = {CORE_URL}\n\tupdate = !false\n')
        self.stage(self.root)
        self.gitlink(self.root, "cli", "a" * 40)
        self.git("commit", "-qm", "update command")
        with self.assertRaisesRegex(ValueError, "unexpected submodule settings"):
            layout.validate("HEAD", False)

    def test_rejects_forbidden_submodule_urls(self):
        for url in ("file:///tmp/marqdo-cli.git", "../marqdo-cli.git", "git@git.cflmy.cn:cflmy/marqdo-cli.git"):
            with self.subTest(url=url):
                self.root_repo(CLI, {"cli": "a" * 40}, {"cli": url})
                with self.assertRaisesRegex(ValueError, "invalid submodule URL: cli"):
                    layout.validate("HEAD", False)

    def test_accepts_a_submodule_section_name_that_differs_from_its_path(self):
        self.root_repo(CLI, {"cli": "a" * 40}, {"cli": CORE_URL})
        self.write(self.root, ".gitmodules",
                   f'[submodule "core-runtime"]\n\tpath = cli\n\turl = {CORE_URL}\n')
        self.stage(self.root)
        self.gitlink(self.root, "cli", "a" * 40)
        self.git("commit", "-qm", "named section")
        layout.validate("HEAD", False)

    def test_rejects_a_wrong_domain_root_url(self):
        self.root_repo(CLI + WEB_DOMAIN,
                       {"cli": "a" * 40, "extensions/web": "b" * 40},
                       {"cli": CORE_URL,
                        "extensions/web": "https://git.cflmy.cn/cflmy/ext-web.git"})
        with self.assertRaisesRegex(ValueError, "invalid submodule URL: extensions/web"):
            layout.validate("HEAD", False)

    def test_rejects_a_domain_without_submodule_gitmodules(self):
        self.assemble(domain_modules={})
        with self.assertRaisesRegex(ValueError, "missing or invalid submodule path: design"):
            layout.validate("HEAD", True)

    def test_rejects_an_unknown_root_dependency(self):
        self.root_repo(CLI.replace('requires = []', 'requires = ["missing"]'), {"cli": "a" * 40}, {"cli": CORE_URL})
        with self.assertRaisesRegex(ValueError, "unknown dependency: cli -> missing"):
            layout.validate("HEAD", False)

    def test_rejects_an_absolute_or_parent_path(self):
        for path in ("/extensions/web", "extensions/../web", "extensions//web"):
            with self.subTest(path=path):
                self.root_repo(
                    CLI + WEB_DOMAIN.replace('path = "extensions/web"', f'path = "{path}"'),
                    {"cli": "a" * 40},
                    {"cli": CORE_URL})
                with self.assertRaisesRegex(ValueError, "invalid path for web"):
                    layout.validate("HEAD", False)


if __name__ == "__main__":
    unittest.main()
