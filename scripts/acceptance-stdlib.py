#!/usr/bin/env python3
"""Accept the stdlib migration candidate with explicit, verified inputs.

This is the root-layer composition gate: it builds (or accepts) the one Bundle
produced by ``scripts/build-stdlib.py`` from the pinned component gitlinks, runs
each active stdlib component's own entry point against that Bundle, runs the
cross-repository composition fixtures, exercises the negative inputs that must
fail closed, and optionally packages the Bundle with the CLI's own packer twice
to prove determinism.

Every input is explicit: a real ``marqdo`` executable (``--cli``) and either a
Bundle (``--bundle``) or a checkout that can build one at its pinned gitlinks.
Nothing is downloaded and no sibling component is discovered implicitly.

Usage:

    python3 scripts/acceptance-stdlib.py --cli build/marqdo --work build/accept
    python3 scripts/acceptance-stdlib.py --cli build/marqdo --work build/accept \
        --pack-tool /path/to/cli/tools/pack-stdlib.py --evidence build/accept/evidence.json
"""

from __future__ import annotations

import argparse
import base64
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CASES = ROOT / "tests" / "integration" / "composition" / "cases"
CASES_SCHEMA = "marqdo.stdlib-test-case"
FORBIDDEN = {
    "net/http.mq.md",
    "regex.mq.md",
    "compress.mq.md",
    "demo.mq.md",
    "tests",
}


class AcceptanceError(Exception):
    pass


def run(command, cwd=None, env=None, input_bytes=b""):
    result = subprocess.run(
        command, cwd=cwd, env=env, input=input_bytes, capture_output=True
    )
    return result


def sha256(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def channel_text(events: list[dict], channel: str) -> str:
    chunks = []
    for event in events:
        if event.get("kind") != channel:
            continue
        payload = event["payload"]
        data = base64.b64decode(payload["data_base64"])
        if len(data) != payload["byte_count"]:
            raise AcceptanceError(f"{channel} byte_count mismatch")
        chunks.append(data)
    return b"".join(chunks).decode("utf-8")


def tree_digest(bundle: Path) -> str:
    digest = hashlib.sha256()
    for path in sorted(bundle.rglob("*.mq.md"), key=lambda p: p.relative_to(bundle).as_posix()):
        digest.update(path.relative_to(bundle).as_posix().encode())
        digest.update(b"\0")
        digest.update(path.read_bytes())
        digest.update(b"\0")
    return "sha256:" + digest.hexdigest()


def aliases_digest(aliases: list[dict]) -> str:
    digest = hashlib.sha256()
    for alias in sorted(aliases, key=lambda a: (a["target_module"], a["target_function"])):
        digest.update(alias["target_module"].encode()); digest.update(b"\0")
        digest.update(alias["target_function"].encode()); digest.update(b"\0")
        for name in sorted(alias["public_names"]):
            digest.update(name.encode()); digest.update(b"\0")
    return "sha256:" + digest.hexdigest()


def verify_bundle(bundle: Path) -> dict:
    manifest_path = bundle / "stdlib.json"
    if not manifest_path.is_file():
        raise AcceptanceError(f"missing Bundle manifest: {manifest_path}")
    manifest = json.loads(manifest_path.read_text())
    for field in ("name", "version", "digest", "aliases_digest"):
        if not isinstance(manifest.get(field), str) or not manifest[field]:
            raise AcceptanceError(f"manifest has no {field}")
    computed = tree_digest(bundle)
    if computed != manifest["digest"]:
        raise AcceptanceError(
            f"manifest digest mismatch: {manifest['digest']} != {computed}"
        )
    aliases = manifest.get("public_aliases", [])
    expected_alias = aliases_digest(aliases)
    if expected_alias != manifest["aliases_digest"]:
        raise AcceptanceError("aliases digest mismatch")
    files = sorted(
        path.relative_to(bundle).as_posix()
        for path in bundle.rglob("*.mq.md")
    )
    for relative in files:
        parts = relative.split("/")
        if "tests" in parts or relative in FORBIDDEN:
            raise AcceptanceError(f"forbidden module in Bundle: {relative}")
    for alias in aliases:
        if alias["target_module"] not in files:
            raise AcceptanceError(
                f"public alias targets missing module {alias['target_module']}"
            )
    if "io.mq.md" not in files:
        raise AcceptanceError("Bundle lost the io.mq.md alias target")
    return {
        "manifest": manifest,
        "files": files,
        "tree_digest": computed,
    }


def run_case(cli: str, bundle: Path, case: dict) -> dict:
    source = ROOT / "tests" / "integration" / "composition" / case["file"]
    command = [cli, "run", str(source), *case.get("args", []), "--output", "json"]
    env = os.environ.copy()
    env["MARQDO_STDLIB_DIR"] = str(bundle)
    for key, value in case.get("env", {}).items():
        env[key] = value.replace("$MARQDO_BIN", cli)
    result = run(command, cwd=ROOT, env=env, input_bytes=case.get("stdin", "").encode())
    if result.returncode != 0:
        raise AcceptanceError(
            f"composition case {case['name']} failed: "
            f"{result.stderr.decode(errors='replace')}"
        )
    report = json.loads(result.stdout)
    if report.get("run", {}).get("status") != "ok":
        raise AcceptanceError(f"composition case {case['name']} run status not ok")
    terminal = report["events"][-1]["payload"]
    if terminal.get("value") != case["expected_terminal"]:
        raise AcceptanceError(
            f"composition case {case['name']} terminal value changed"
        )
    stdout = channel_text(report["events"], "stdout.chunk")
    if stdout != case.get("expected_stdout", ""):
        raise AcceptanceError(
            f"composition case {case['name']} stdout changed: {stdout!r}"
        )
    return {"name": case["name"], "status": "PASS"}


def load_components() -> list[dict]:
    catalog = tomllib.loads((ROOT / "components.toml").read_text())
    return [
        component
        for component in catalog["component"]
        if component.get("kind") == "stdlib" and component.get("state") == "active"
    ]


def load_explicit_packages() -> list[dict]:
    """Planned extensions that already publish an explicit Package entry.

    A component is gated as an explicit package only when its pinned checkout
    carries the current package layout (``marqdo.yaml`` + ``index.mq.md`` +
    ``scripts/test.sh``); it is never silently loaded into the default Bundle.
    """
    catalog = tomllib.loads((ROOT / "components.toml").read_text())
    packages = []
    for component in catalog["component"]:
        if component.get("kind") != "extension":
            continue
        checkout = ROOT / component["path"]
        if (checkout / "marqdo.yaml").is_file() and (
            checkout / "index.mq.md"
        ).is_file() and (checkout / "scripts" / "test.sh").is_file():
            packages.append(component)
    return packages


def component_gates(cli: str, bundle: Path) -> list[dict]:
    results = []
    for component in load_components():
        entry = ROOT / component["path"] / component["entry"]
        if not entry.is_file():
            raise AcceptanceError(
                f"active component {component['id']} has no entry {component['entry']}"
            )
        env = os.environ.copy()
        env["MARQDO_BIN"] = cli
        env["MARQDO_STDLIB_DIR"] = str(bundle)
        result = run([str(entry)], cwd=entry.parent.parent, env=env)
        output = (result.stdout + result.stderr).decode(errors="replace").strip()
        if result.returncode != 0:
            raise AcceptanceError(
                f"component {component['id']} failed entry {component['entry']}:\n{output}"
            )
        results.append(
            {
                "id": component["id"],
                "entry": component["entry"],
                "status": "PASS",
                "output": output,
            }
        )
    for component in load_explicit_packages():
        entry = ROOT / component["path"] / "scripts" / "test.sh"
        env = os.environ.copy()
        env["MARQDO_BIN"] = cli
        env["MARQDO_STDLIB_DIR"] = str(bundle)
        result = run([str(entry)], cwd=ROOT / component["path"], env=env)
        output = (result.stdout + result.stderr).decode(errors="replace").strip()
        if result.returncode != 0:
            raise AcceptanceError(
                f"explicit package {component['id']} failed: {output}"
            )
        results.append(
            {
                "id": component["id"],
                "entry": "scripts/test.sh",
                "kind": "explicit-package",
                "status": "PASS",
                "output": output,
            }
        )
    return results


def negative_gates(cli: str, bundle: Path, work: Path) -> list[dict]:
    results = []

    def expect_failure(name, command, env_for_gate):
        result = run(command, env=env_for_gate)
        if result.returncode == 0:
            raise AcceptanceError(f"negative gate {name} unexpectedly passed")
        results.append(
            {
                "name": name,
                "status": "PASS",
                "exit": result.returncode,
                "stderr": result.stderr.decode(errors="replace").strip()[:400],
            }
        )

    missing = work / "missing-bundle"
    env = os.environ.copy()
    env["MARQDO_STDLIB_DIR"] = str(missing)
    expect_failure(
        "missing-explicit-bundle",
        [cli, "run", str(ROOT / "tests/integration/composition/default-bundle.mq.md")],
        env,
    )

    tampered = work / "tampered-bundle"
    if tampered.exists():
        shutil.rmtree(tampered)
    shutil.copytree(bundle, tampered)
    target = next(iter(sorted(tampered.rglob("*.mq.md"))))
    target.write_bytes(target.read_bytes() + b"\n")
    env["MARQDO_STDLIB_DIR"] = str(tampered)
    expect_failure(
        "tampered-module-digest",
        [cli, "run", str(ROOT / "tests/integration/composition/default-bundle.mq.md")],
        env,
    )
    return results


def packaging_gates(pack_tool: str, bundle: Path, work: Path) -> list[dict]:
    results = []
    archives = []
    for attempt in (1, 2):
        out_dir = work / f"pack-{attempt}"
        if out_dir.exists():
            shutil.rmtree(out_dir)
        result = run(
            [
                sys.executable,
                pack_tool,
                "--stdlib-dir",
                str(bundle),
                "--out-dir",
                str(out_dir),
            ]
        )
        if result.returncode != 0:
            raise AcceptanceError(
                "CLI packer rejected the Bundle: "
                + result.stderr.decode(errors="replace")
            )
        archives.append(sorted(out_dir.glob("*.tar.gz"))[0])
    if archives[0].read_bytes() != archives[1].read_bytes():
        raise AcceptanceError("two packaging runs are not byte-identical")
    descriptor = json.loads(
        sorted(archives[0].parent.glob("*.json"))[0].read_text()
    )
    results.append(
        {
            "status": "PASS",
            "archive": archives[0].name,
            "archive_sha256": sha256(archives[0]),
            "tree_digest": descriptor.get("tree_digest"),
            "aliases_digest": descriptor.get("aliases_digest"),
            "deterministic": True,
        }
    )
    return results


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", required=True)
    parser.add_argument("--work", required=True)
    parser.add_argument("--bundle")
    parser.add_argument("--pack-tool")
    parser.add_argument("--evidence")
    parser.add_argument("--root-commit")
    args = parser.parse_args(argv)
    cli = str(Path(args.cli).resolve())
    if not Path(cli).is_file():
        raise SystemExit(f"acceptance-stdlib: error: no CLI executable at {cli}")
    work = Path(args.work).resolve()
    work.mkdir(parents=True, exist_ok=True)
    try:
        if args.bundle:
            bundle = Path(args.bundle).resolve()
        else:
            bundle = work / "bundle"
            if bundle.exists():
                shutil.rmtree(bundle)
            command = [
                sys.executable,
                str(ROOT / "scripts" / "build-stdlib.py"),
                "--out",
                str(bundle),
            ]
            if args.root_commit:
                command += ["--root-commit", args.root_commit]
            result = run(command, cwd=ROOT)
            if result.returncode != 0:
                raise AcceptanceError(
                    "Bundle producer failed: "
                    + result.stderr.decode(errors="replace")
                )
        bundle_record = verify_bundle(bundle)
        version = run([cli, "--version"]).stdout.decode().strip()
        cases = []
        for path in sorted(CASES.glob("*.json")):
            case = json.loads(path.read_text())
            if case.get("schema") != CASES_SCHEMA:
                raise AcceptanceError(f"unsupported case schema: {path}")
            cases.append(run_case(cli, bundle, case))
        components = component_gates(cli, bundle)
        negatives = negative_gates(cli, bundle, work)
        packaging = (
            packaging_gates(args.pack_tool, bundle, work)
            if args.pack_tool
            else []
        )
        evidence = {
            "schema": "marqdo.stdlib-acceptance",
            "schema_version": 1,
            "cli": {"path": cli, "version": version, "sha256": sha256(Path(cli))},
            "root_commit": run(
                ["git", "-C", str(ROOT), "rev-parse", "HEAD"]
            ).stdout.decode().strip(),
            "bundle": {
                "name": bundle_record["manifest"]["name"],
                "version": bundle_record["manifest"]["version"],
                "tree_digest": bundle_record["tree_digest"],
                "aliases_digest": bundle_record["manifest"]["aliases_digest"],
                "files": bundle_record["files"],
            },
            "components": components,
            "composition": cases,
            "negatives": negatives,
            "packaging": packaging,
            "status": "PASS",
        }
    except AcceptanceError as error:
        print(f"acceptance-stdlib: FAIL: {error}", file=sys.stderr)
        return 1
    text = json.dumps(evidence, ensure_ascii=False, indent=2) + "\n"
    if args.evidence:
        Path(args.evidence).write_text(text)
    print(text)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
