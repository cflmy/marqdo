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
active="$(python3 scripts/check-layout.py --checkouts --active-paths)"
expected=$'cli\nstdlib/collections\nstdlib/data\nstdlib/fs\nstdlib/io\nstdlib/system\nstdlib/text\ntests/conformance\ntools/knowledge\ntools/lint\ntools/prove\ntools/view'
if [[ "$active" != "$expected" ]]; then
    echo "UNVERIFIED: update the integration plan for the changed active component set" >&2
    exit 1
fi

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
python3 cli/tools/release/smoke.py --archive "$archive"
mkdir -p "$work/install"
tar -xzf "$archive" -C "$work/install"
installed="$(find "$work/install" -mindepth 1 -maxdepth 1 -type d -print -quit)"
MARQDO_BIN="$installed/marqdo"
export MARQDO_BIN
test "$(cat "$installed/SOURCE_SHA")" = "$source_sha"
test "$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' "$installed/stdlib/stdlib.json")" = "$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' "$work/stdlib/stdlib.json")"
sha256sum "$archive" "$MARQDO_BIN" "$installed/marqdo-provider-worker"

# 4. Root stdlib acceptance: component entries, explicit Packages, composition
#    fixtures and fail-closed negatives with the same installed CLI/Bundle.
python3 scripts/acceptance-stdlib.py \
    --cli "$MARQDO_BIN" --bundle "$work/stdlib" \
    --work "$work/stdlib-acceptance" --evidence "$work/stdlib-acceptance.json"

# 5. Frozen independent public-contract fixtures against the same CLI.
python3 cli/tools/ci/verify_extension_contracts.py \
    --source-repo cli --tested-sha "$source_sha" --cli "$MARQDO_BIN" \
    --release-info "$installed/RELEASE_INFO.json" \
    --fixtures tests/conformance --out "$work/conformance.json"

# 6. Existing consumers, same installed binary and adjacent Bundle.
(
    cd tools/view
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
)
(
    cd tools/knowledge
    MARQDO_TEST_BIN="$MARQDO_BIN" cargo test --locked --test core_cli --test compile
)
(
    cd tools/prove
    cargo test --locked --test analysis_document --test document_contract
)
(
    cd tools/lint
    export MARQDO_TOOL_HOME="$work/lint-registry"
    scripts/package-tool.sh "$work/lint"
    "$MARQDO_BIN" tool install "$work/lint/tool.json"
    "$MARQDO_BIN" lint examples/nested/inventory.mq.md --format json
    "$MARQDO_BIN" tool remove lint
)

# 7. Evidence identity for the accepted candidate.
python3 - "$work" "$source_sha" "$bundle_digest" "$bundle_aliases" "$archive" <<'PY'
import hashlib, json, pathlib, sys
work = pathlib.Path(sys.argv[1])
archive = pathlib.Path(sys.argv[5])
evidence = {
    "schema": "marqdo.ecosystem-integration",
    "schema_version": 1,
    "root_commit": sys.argv[2],
    "cli_source_sha": sys.argv[2],
    "stdlib_tree_digest": sys.argv[3],
    "stdlib_aliases_digest": sys.argv[4],
    "archive": archive.name,
    "archive_sha256": "sha256:" + hashlib.sha256(archive.read_bytes()).hexdigest(),
    "status": "PASS",
}
(work / "integration-evidence.json").write_text(json.dumps(evidence, indent=2) + "\n")
print(json.dumps(evidence, indent=2))
PY

echo "PASS: ecosystem candidate (single CLI/Worker/Bundle artifact, consumers green)"
