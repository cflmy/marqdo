#!/usr/bin/env bash
# Ecosystem integration for one exact candidate combination.
#
# Inputs are the pinned root gitlinks: the root-owned Bundle producer assembles
# the one stdlib Bundle, the CLI/Worker are built from the pinned `cli` commit,
# the release archive is packaged with that explicit Bundle, and every consumer
# runs against that same installed binary + adjacent Bundle. There is no
# implicit artifact from View, no floating branch and no network-selected CLI.
set -euo pipefail

cd "$(dirname "$0")/.."
root="$PWD"
plan="$(python3 scripts/check-layout.py --active-entries)"
declare -a gates=()
cli_smoke=
while IFS=$'\t' read -r id kind path entry; do
    case "$kind:$id:$entry" in
        core:cli:tools/release/smoke.py) cli_smoke="$path/$entry"; gate=core ;;
        stdlib:*:scripts/test.sh) gate=stdlib ;;
        test:conformance:scripts/build_all.sh) gate=conformance ;;
        domain:web:scripts/domain-acceptance.sh) gate=web ;;
        app:app-harness:scripts/ci.sh) gate=app ;;
        tool:view:scripts/acceptance.sh) gate=view ;;
        tool:knowledge:scripts/ci.sh) gate=knowledge ;;
        tool:prove:scripts/package-tool.sh) gate=prove ;;
        tool:lint:scripts/package-tool.sh) gate=lint ;;
        tool:*:scripts/ci.sh|tool:*:scripts/integration.sh|extension:*:scripts/ci.sh|extension:*:scripts/integration.sh|test:*:scripts/ci.sh|test:*:scripts/integration.sh)
            gate=entry
            ;;
        *)
            echo "UNVERIFIED: unsupported integration entry $id ($kind): $entry" >&2
            exit 1
            ;;
    esac
    gates+=("$gate")
done <<< "$plan"
test -n "$cli_smoke"

mkdir -p .integration-work
work="$(mktemp -d "$root/.integration-work/run.XXXXXX")"
echo "integration evidence: $work"
exec > >(tee "$work/run.log") 2>&1
set -x

git rev-parse HEAD
git submodule status --recursive
uname -m
ldd --version
cargo --version
python3 --version
python3 -c 'import jsonschema'
python3 scripts/check-layout.py --checkouts > "$work/layout.txt"
cat "$work/layout.txt"
printf '%s\n' "$plan" > "$work/active-entries.tsv"

# Recursive impact analysis against the accepted root commit: the change set
# comes from the recursively expanded (global_id, parent, remote, SHA, state)
# identities, then reverse `requires` closure, parent domains and `checks_with`
# fixed point. An undetermined scope widens to the full candidate.
accepted_ref="${MARQDO_ACCEPTED_REF:-}"
if [[ -z "$accepted_ref" && -n "${CI_COMMIT_TARGET_BRANCH:-}" ]] \
    && git rev-parse --verify "origin/$CI_COMMIT_TARGET_BRANCH" >/dev/null 2>&1; then
    accepted_ref="origin/$CI_COMMIT_TARGET_BRANCH"
elif [[ -z "$accepted_ref" ]] && git rev-parse --verify origin/main >/dev/null 2>&1; then
    accepted_ref="origin/main"
fi
if [[ -n "$accepted_ref" ]]; then
    python3 scripts/impact.py --accepted "$accepted_ref" --candidate HEAD \
        --out "$work/impact.json"
fi

# 1. One Bundle, produced by the root-owned producer from the pinned gitlinks.
python3 scripts/build-stdlib.py --out "$work/stdlib" | tee "$work/bundle.json"
bundle_digest="$(python3 -c 'import json,sys; print(json.load(sys.stdin)["tree_digest"])' < "$work/bundle.json")"
bundle_aliases="$(python3 -c 'import json,sys; print(json.load(sys.stdin)["aliases_digest"])' < "$work/bundle.json")"
test -f "$work/stdlib/stdlib.json"

# 2. CLI and Worker from the exact pinned CLI gitlink.
source_sha="$(git -C cli rev-parse HEAD)"
# The CLI release packager resolves its binaries from the pinned CLI checkout's
# own target directory; keep this build there so the package step consumes the
# same artifacts without a second build. Do not export CARGO_TARGET_DIR: the
# consumer repositories build into their own target directories.
CARGO_TARGET_DIR="$root/cli/target" \
    cargo build --manifest-path cli/Cargo.toml --locked --release \
    -p marqdo-cli -p marqdo-provider-worker
version="$(python3 -c '
import pathlib, tomllib
data = tomllib.loads(pathlib.Path("cli/Cargo.toml").read_text())
print(data["workspace"]["package"]["version"])
')"

# 3. The release archive takes the Bundle as one explicit input.
python3 cli/tools/release/package.py \
    --source-sha "$source_sha" --version "$version" \
    --stdlib-dir "$work/stdlib" --out-dir "$work/dist" | tee "$work/package.json"
archive="$(find "$work/dist" -maxdepth 1 -name 'marqdo-*.tar.gz' -print -quit)"
python3 "$cli_smoke" --archive "$archive"
mkdir -p "$work/install"
tar -xzf "$archive" -C "$work/install"
installed="$(find "$work/install" -mindepth 1 -maxdepth 1 -type d -print -quit)"
MARQDO_BIN="$installed/marqdo"
export MARQDO_BIN
test "$(cat "$installed/SOURCE_SHA")" = "$source_sha"
test "$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' "$installed/stdlib/stdlib.json")" = "$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' "$work/stdlib/stdlib.json")"
sha256sum "$archive" "$MARQDO_BIN" "$installed/marqdo-provider-worker"

# Fixed identity handoff for the Node application lane. The manifest is a
# consumer input, not a new lock: it names the exact archive, CLI source,
# Bundle digest and installed binary produced above.
python3 - "$work" "$archive" "$source_sha" "$bundle_digest" "$bundle_aliases" "$installed" "$root" <<'PY'
import hashlib, json, pathlib, subprocess, sys
work, archive, source_sha, bundle_digest, aliases, installed, root = map(pathlib.Path, sys.argv[1:])
manifest = {
    'schema': 'marqdo.app-candidate',
    'schema_version': 1,
    'candidate_root': str(root),
    'archive': str(archive),
    'archive_sha256': 'sha256:' + hashlib.sha256(archive.read_bytes()).hexdigest(),
    'installed_dir': str(installed),
    'cli_path': str(installed / 'marqdo'),
    'cli_source_sha': source_sha,
    'stdlib_tree_digest': bundle_digest,
    'stdlib_aliases_digest': aliases,
    'platform': 'x86_64-unknown-linux-gnu',
    'app_gitlink': subprocess.check_output(
        ['git', '-C', str(root / 'apps/harness'), 'rev-parse', 'HEAD'], text=True).strip(),
}
(work / 'app-candidate.json').write_text(json.dumps(manifest, indent=2) + '\n')
(work.parent / 'app-candidate.json').write_text(json.dumps(manifest, indent=2) + '\n')
print(json.dumps(manifest, indent=2))
PY

# Tool consumers use the same candidate installation and the catalog order.
# The candidate image uses a minimal Rust profile; component entries run
# fmt/clippy, so add those components once when the active toolchain lacks them.
if command -v rustup >/dev/null 2>&1 \
    && ! rustup component list --installed 2>/dev/null | grep -qx 'rustfmt.*'; then
    rustup component add rustfmt clippy
fi

run_consumer() (
    cd "$root/$path"
    export MARQDO_CI_OUT="$work/components/$id"
    mkdir -p "$MARQDO_CI_OUT"
    case "$gate" in
        entry)
            MARQDO_STDLIB_DIR="$installed/stdlib" "./$entry"
            ;;
        web)
            web_tool="$PWD/preview/dist/marqdo-web-preview/tool.json"
            export MARQDO_TOOL_HOME="$MARQDO_CI_OUT/tool-home"
            cli_before="$(sha256sum "$MARQDO_BIN")"
            "$MARQDO_BIN" tool install "$web_tool"
            example=examples/research-workspace
            "$MARQDO_BIN" web check "$example/index.mq.md" --unit 工作台 --root "$example"
            "$MARQDO_BIN" web preview "$example/index.mq.md" --unit 工作台 --root "$example" \
                --out "$MARQDO_CI_OUT/preview"
            test -s "$MARQDO_CI_OUT/preview/index.html"
            "$MARQDO_BIN" web handoff "$example/index.mq.md" --unit 工作台 --root "$example" \
                --out "$MARQDO_CI_OUT/handoff.md"
            test -s "$MARQDO_CI_OUT/handoff.md"
            "$MARQDO_BIN" tool remove web
            test "$(sha256sum "$MARQDO_BIN")" = "$cli_before"
            ;;
        app)
            # The Rust candidate lane builds the one CLI/Bundle artifact and
            # emits its identity. The dedicated Node lane runs scripts/ci.sh
            # against that exact artifact; this lane must not install Node.
            test -x "$PWD/$entry"
            echo "APP_LANE: candidate manifest $work/app-candidate.json"
            ;;
        view)
            export MARQDO_TOOL_HOME="$work/view-registry"
            scripts/package.sh
            "$MARQDO_BIN" tool install dist/marqdo-view/tool.json
            "$MARQDO_BIN" view verify examples/mixed-modes/index.mq.md
            "$MARQDO_BIN" view export examples/tables-math --out "$work/view-export"
            setsid "$MARQDO_BIN" view examples/control-flow --port 0 --no-open >"$work/view-server.log" 2>&1 &
            view_pid=$!
            stop_view() {
                kill -- "-$view_pid" 2>/dev/null || true
                wait "$view_pid" 2>/dev/null || true
            }
            trap stop_view EXIT
            view_url=""
            for attempt in {1..100}; do
                view_url="$(grep -m1 -o 'http://127\.0\.0\.1:[0-9]\+/' "$work/view-server.log" || true)"
                if [[ -n "$view_url" ]] || ! kill -0 "$view_pid" 2>/dev/null; then
                    break
                fi
                sleep 0.1
            done
            if [[ -z "$view_url" ]]; then
                cat "$work/view-server.log" >&2
                exit 1
            fi
            python3 scripts/acceptance-run.py "$view_url"
            stop_view
            trap - EXIT
            python3 scripts/check-observation.py stream-snapshot
            "$MARQDO_BIN" tool remove view
            if "$MARQDO_BIN" view --version; then
                echo "view still dispatches after removal" >&2
                exit 1
            fi
            ;;
        knowledge)
            MARQDO_TEST_BIN="$MARQDO_BIN" cargo test --locked --test core_cli --test compile
            ;;
        prove)
            cargo test --locked --test analysis_document --test document_contract
            ;;
        lint)
            export MARQDO_TOOL_HOME="$work/lint-registry"
            "./$entry" "$work/lint"
            "$MARQDO_BIN" tool install "$work/lint/tool.json"
            "$MARQDO_BIN" lint examples/nested/inventory.mq.md --format json
            "$MARQDO_BIN" tool remove lint
            ;;
    esac
)

stdlib_accepted=false
index=0
while IFS=$'\t' read -r -u 3 id kind path entry; do
    gate="${gates[index]}"
    index=$((index + 1))
    echo "candidate gate: $id ($path/$entry)"
    case "$gate" in
        core) ;;  # CLI/Worker packaging and smoke ran before consumer gates.
        stdlib)
            if [ "$stdlib_accepted" = false ]; then
                python3 scripts/acceptance-stdlib.py \
                    --cli "$MARQDO_BIN" --bundle "$work/stdlib" \
                    --work "$work/stdlib-acceptance" --evidence "$work/stdlib-acceptance.json"
                stdlib_accepted=true
            fi
            ;;
        conformance)
            python3 cli/tools/ci/verify_extension_contracts.py \
                --source-repo cli --tested-sha "$source_sha" --cli "$MARQDO_BIN" \
                --release-info "$installed/RELEASE_INFO.json" \
                --fixtures "$path" --out "$work/conformance.json"
            ;;
        *) run_consumer ;;
    esac
done 3<<< "$plan"

# 7. Evidence identity for the accepted candidate.
root_commit="$(git rev-parse HEAD)"
python3 - "$work" "$root_commit" "$source_sha" "$bundle_digest" "$bundle_aliases" "$archive" <<'PY'
import hashlib, json, pathlib, sys
work = pathlib.Path(sys.argv[1])
archive = pathlib.Path(sys.argv[6])
impact_path = work / "impact.json"
impact = json.loads(impact_path.read_text()) if impact_path.is_file() else None
evidence = {
    "schema": "marqdo.ecosystem-integration",
    "schema_version": 1,
    "root_commit": sys.argv[2],
    "cli_source_sha": sys.argv[3],
    "stdlib_tree_digest": sys.argv[4],
    "stdlib_aliases_digest": sys.argv[5],
    "archive": archive.name,
    "archive_sha256": "sha256:" + hashlib.sha256(archive.read_bytes()).hexdigest(),
    "active_entries": [line.split("\t") for line in (work / "active-entries.tsv").read_text().splitlines()],
    "impact": (
        {
            "changed": impact["changed"],
            "affected": impact["affected"],
            "checks_with": impact["checks_with"],
            "fallback": impact["fallback"],
            "unverified": impact["unverified"],
        }
        if impact
        else None
    ),
    "status": "PASS",
}
(work / "integration-evidence.json").write_text(json.dumps(evidence, indent=2) + "\n")
print(json.dumps(evidence, indent=2))
PY

echo "PASS: ecosystem candidate (single CLI/Worker/Bundle artifact, consumers green)"
