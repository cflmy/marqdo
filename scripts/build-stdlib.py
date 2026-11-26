#!/usr/bin/env python3
"""Build the one official `marqdo-stdlib` Bundle from pinned component gitlinks.

The producer owns composition only: it collects the published `modules/` files
of every **active** `kind = "stdlib"` component at the exact commit pinned by
the root Gitlink, validates the published path set, generates the current
`stdlib.json` (tree digest + public aliases digest) and writes a deterministic
Bundle directory. Component repositories keep the module sources and tests; the
root repository does not copy or edit them. The CLI packages the resulting
Bundle with its existing `tools/pack-stdlib.py`.

Identity rules:

* release mode (default) reads `components.toml`, the Bundle config and the
  gitlinks from the ``--root-commit`` tree, never from the working tree;
* `--worktree` is a development-only mode that reads each component's
  ``HEAD`` and requires a clean component checkout;
* only ``modules/**/*.mq.md`` is published. Other files under ``modules/``
  (for example ``modules/README.md``) stay repository documentation and are
  not packed. Symlinks, submodules and non-regular entries are refused;
* published paths must be NFC, case-insensitively unique and free of
  ``tests/``, ``.git`` or traversal segments;
* a version bump is explicit: with ``--baseline-manifest`` content digest
  changes are refused unless ``version`` also changes.

Usage:

    python3 scripts/build-stdlib.py --out build/stdlib
    python3 scripts/build-stdlib.py --out build/stdlib --worktree
    python3 scripts/build-stdlib.py --out build/stdlib \
        --baseline-manifest previous/stdlib.json
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import tomllib
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_CONFIG = Path("scripts") / "stdlib-bundle.json"
MANIFEST = "stdlib.json"
MODULES = "modules"
PUBLISHED_SUFFIX = ".mq.md"


class BundleError(Exception):
    """An input cannot become the official Bundle."""


def _git(repo: Path, *args: str) -> bytes:
    result = subprocess.run(
        ["git", "-C", str(repo), *args], capture_output=True
    )
    if result.returncode != 0:
        raise BundleError(
            f"git failed in {repo}: {' '.join(args)}: "
            f"{result.stderr.decode(errors='replace').strip()}"
        )
    return result.stdout


def _nfc(value: str) -> str:
    return unicodedata.normalize("NFC", value)


def load_json(path: Path) -> dict:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise BundleError(f"cannot read {path}: {error}") from error
    if not isinstance(data, dict):
        raise BundleError(f"{path} must contain a JSON object")
    return data


def read_blob(repo: Path, revision: str, path: str) -> bytes | None:
    result = subprocess.run(
        ["git", "-C", str(repo), "show", f"{revision}:{path}"],
        capture_output=True,
    )
    return result.stdout if result.returncode == 0 else None


def read_catalog(root: Path, revision: str | None) -> list[dict]:
    if revision is None:
        raw = (root / "components.toml").read_bytes()
    else:
        raw = read_blob(root, revision, "components.toml")
        if raw is None:
            raise BundleError(f"root commit {revision} has no components.toml")
    try:
        catalog = tomllib.loads(raw.decode("utf-8"))
    except (UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
        raise BundleError(f"invalid components.toml: {error}") from error
    components = catalog.get("component")
    if not isinstance(components, list):
        raise BundleError("components.toml has no [[component]] entries")
    return components


def read_config(root: Path, revision: str | None, path: Path) -> dict:
    if not path.is_absolute():
        path = root / path
    if revision is not None:
        relative = path.resolve().relative_to(root.resolve()).as_posix()
        raw = read_blob(root, revision, relative)
        if raw is None:
            raise BundleError(f"root commit {revision} has no {relative}")
        try:
            config = json.loads(raw.decode("utf-8"))
        except (UnicodeDecodeError, json.JSONDecodeError) as error:
            raise BundleError(f"invalid Bundle config {relative}: {error}") from error
    else:
        config = load_json(path)
    for field in ("name", "version"):
        value = config.get(field)
        if not isinstance(value, str) or not value:
            raise BundleError(f"Bundle config has no non-empty `{field}`")
    aliases = config.get("public_aliases", [])
    if not isinstance(aliases, list):
        raise BundleError("Bundle config `public_aliases` must be a list")
    return config


def pinned_gitlink(root: Path, revision: str, path: str) -> str:
    result = _git(root, "ls-tree", revision, "--", path).decode().strip()
    if not result:
        raise BundleError(
            f"root commit {revision} has no gitlink at {path}; only published "
            "components may join the Bundle"
        )
    fields = result.split()
    if len(fields) != 4:
        raise BundleError(f"invalid gitlink record for {path}")
    mode, kind, sha, _ = fields
    if mode != "160000" or kind != "commit":
        raise BundleError(f"{path} is not a gitlink in root commit {revision}")
    return sha


def worktree_sha(component_dir: Path) -> str:
    if not (component_dir / ".git").exists():
        raise BundleError(f"missing component checkout: {component_dir}")
    status = _git(component_dir, "status", "--porcelain").decode().strip()
    if status:
        raise BundleError(f"dirty component checkout: {component_dir}")
    return _git(component_dir, "rev-parse", "HEAD").decode().strip()


def validate_published_path(relative: str, component_id: str) -> None:
    if not relative or relative.startswith("/"):
        raise BundleError(f"{component_id}: invalid published path {relative!r}")
    parts = relative.split("/")
    if any(part in ("", ".", "..") for part in parts):
        raise BundleError(f"{component_id}: unsafe published path {relative!r}")
    if any(part.startswith(".") for part in parts):
        raise BundleError(f"{component_id}: hidden path segment in {relative!r}")
    if "tests" in parts:
        raise BundleError(f"{component_id}: tests must not enter the Bundle: {relative!r}")
    if not relative.endswith(PUBLISHED_SUFFIX):
        raise BundleError(
            f"{component_id}: only {PUBLISHED_SUFFIX} files are published, got {relative!r}"
        )
    if _nfc(relative) != relative:
        raise BundleError(
            f"{component_id}: published path is not NFC normalised: {relative!r}"
        )


def component_modules(component_dir: Path, revision: str, component_id: str) -> list[tuple[str, str]]:
    output = _git(
        component_dir, "ls-tree", "-r", "-z", revision, "--", MODULES
    )
    records = [record for record in output.split(b"\0") if record]
    published: list[tuple[str, str]] = []
    for record in records:
        header, _, raw_path = record.partition(b"\t")
        fields = header.decode().split()
        if len(fields) != 3:
            raise BundleError(f"{component_id}: invalid tree record {record!r}")
        mode, kind, sha = fields
        path = raw_path.decode()
        if kind != "blob" or mode not in ("100644", "100755"):
            raise BundleError(
                f"{component_id}: only regular modules files are publishable, "
                f"found {mode} {kind} at {path}"
            )
        relative = path[len(MODULES) + 1 :]
        if not relative.endswith(PUBLISHED_SUFFIX):
            continue
        validate_published_path(relative, component_id)
        published.append((relative, sha))
    if not published:
        raise BundleError(
            f"{component_id}: active component publishes no {PUBLISHED_SUFFIX} modules"
        )
    return published


def collect(root: Path, components: list[dict], *, root_commit: str | None) -> tuple[dict[str, bytes], list[dict]]:
    active = [
        component
        for component in components
        if component.get("kind") == "stdlib" and component.get("state") == "active"
    ]
    if not active:
        raise BundleError("no active stdlib component; nothing to publish")
    for component in active:
        if component.get("release") != "base":
            raise BundleError(
                f'{component["id"]}: active stdlib component must have release = "base"'
            )
    files: dict[str, bytes] = {}
    identities: list[dict] = []
    owners: dict[str, str] = {}
    folded: dict[str, str] = {}
    for component in active:
        component_id = component["id"]
        path = component["path"]
        component_dir = (root / path).resolve()
        revision = (
            pinned_gitlink(root, root_commit, path)
            if root_commit is not None
            else worktree_sha(component_dir)
        )
        modules = component_modules(component_dir, revision, component_id)
        for relative, sha in modules:
            key = _nfc(relative).casefold()
            if relative in owners:
                raise BundleError(
                    f"duplicate published path {relative!r}: "
                    f"{owners[relative]} and {component_id}"
                )
            if key in folded:
                raise BundleError(
                    f"case/NFC collision on {relative!r}: "
                    f"{folded[key]} and {component_id}"
                )
            owners[relative] = component_id
            folded[key] = component_id
            files[relative] = _git(component_dir, "cat-file", "blob", sha)
        identities.append(
            {"id": component_id, "path": path, "revision": revision, "files": len(modules)}
        )
    return files, identities


def tree_digest(files: dict[str, bytes]) -> str:
    digest = hashlib.sha256()
    for relative in sorted(files):
        digest.update(relative.encode())
        digest.update(b"\0")
        digest.update(files[relative])
        digest.update(b"\0")
    return "sha256:" + digest.hexdigest()


def aliases_digest(aliases: list[dict]) -> str:
    digest = hashlib.sha256()
    for alias in sorted(
        aliases, key=lambda item: (item["target_module"], item["target_function"])
    ):
        digest.update(alias["target_module"].encode())
        digest.update(b"\0")
        digest.update(alias["target_function"].encode())
        digest.update(b"\0")
        for name in sorted(alias["public_names"]):
            digest.update(name.encode())
            digest.update(b"\0")
    return "sha256:" + digest.hexdigest()


def validate_aliases(aliases: list[dict], files: dict[str, bytes]) -> None:
    seen: set[str] = set()
    for alias in aliases:
        if not isinstance(alias, dict):
            raise BundleError("public alias entry must be an object")
        module = alias.get("target_module")
        function = alias.get("target_function")
        names = alias.get("public_names")
        if module not in files:
            raise BundleError(f"public alias targets missing module: {module!r}")
        if not isinstance(function, str) or not function:
            raise BundleError(f"public alias for {module} has no target_function")
        if not isinstance(names, list) or not names or not all(
            isinstance(name, str) and name for name in names
        ):
            raise BundleError(f"public alias for {module}.{function} has invalid names")
        for name in names:
            if name in seen:
                raise BundleError(f"public alias name declared twice: {name!r}")
            seen.add(name)
        if alias.get("expected_signature") != "params(值)":
            raise BundleError(
                f"public alias {module}.{function} must keep expected_signature params(值)"
            )
        if alias.get("effects") != ["host"]:
            raise BundleError(
                f"public alias {module}.{function} must keep effects [host]"
            )


def build_manifest(config: dict, files: dict[str, bytes]) -> dict:
    aliases = config.get("public_aliases", [])
    validate_aliases(aliases, files)
    return {
        "name": config["name"],
        "version": config["version"],
        "digest": tree_digest(files),
        "public_aliases": aliases,
        "aliases_digest": aliases_digest(aliases),
    }


def check_baseline(manifest: dict, baseline_path: Path) -> None:
    baseline = load_json(baseline_path)
    if baseline.get("version") == manifest["version"] and baseline.get(
        "digest"
    ) != manifest["digest"]:
        raise BundleError(
            f"Bundle content changed but version is still {manifest['version']}; "
            "bump the version explicitly"
        )


def write_bundle(out: Path, manifest: dict, files: dict[str, bytes], force: bool) -> None:
    if out.exists():
        if not force:
            raise BundleError(f"output directory already exists: {out}")
        shutil.rmtree(out)
    out.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(tempfile.mkdtemp(prefix=".stdlib-build-", dir=str(out.parent)))
    try:
        for relative, data in files.items():
            target = staging / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        (staging / MANIFEST).write_text(
            json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
        )
        os.replace(staging, out)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", required=True, help="Bundle output directory")
    parser.add_argument("--root-commit", default=None)
    parser.add_argument("--worktree", action="store_true")
    parser.add_argument("--config", default=None)
    parser.add_argument("--baseline-manifest", default=None)
    parser.add_argument("--force", action="store_true")
    parser.add_argument("--root", default=str(ROOT), help=argparse.SUPPRESS)
    args = parser.parse_args(argv)
    if args.worktree and args.root_commit:
        parser.error("--worktree and --root-commit are mutually exclusive")
    root = Path(args.root).resolve()
    revision = None
    if not args.worktree:
        revision = args.root_commit or _git(root, "rev-parse", "HEAD").decode().strip()
    try:
        components = read_catalog(root, revision)
        config = read_config(root, revision, Path(args.config) if args.config else DEFAULT_CONFIG)
        files, identities = collect(root, components, root_commit=revision)
        manifest = build_manifest(config, files)
        if args.baseline_manifest:
            check_baseline(manifest, Path(args.baseline_manifest))
    except BundleError as error:
        raise SystemExit(f"build-stdlib: error: {error}") from error
    write_bundle(Path(args.out), manifest, files, args.force)
    summary = {
        "name": manifest["name"],
        "version": manifest["version"],
        "tree_digest": manifest["digest"],
        "aliases_digest": manifest["aliases_digest"],
        "root_commit": revision,
        "components": identities,
        "files": sorted(files),
    }
    print(json.dumps(summary, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
