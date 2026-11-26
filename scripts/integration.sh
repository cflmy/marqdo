#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
root="$PWD"
active="$(python3 scripts/check-layout.py --checkouts --active-paths)"
expected=$'cli\ntests/conformance\ntools/knowledge\ntools/lint\ntools/prove\ntools/view'
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

unset MARQDO_STDLIB_DIR
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
mkdir "$work/frozen"
frozen="$root/tools/view/scripts/frozen-beta.sh"
metadata="$("$frozen" metadata)"
artifact="$(sed -n 's/^artifact=//p' <<<"$metadata")"
if [[ -n "${MARQDO_INTEGRATION_ARCHIVE:-}" ]]; then
    archive="$(realpath "$MARQDO_INTEGRATION_ARCHIVE")"
    archive_sha="$(sed -n 's/^archive_sha=//p' <<<"$metadata")"
    printf '%s  %s\n' "$archive_sha" "$archive" | sha256sum -c -
    tar -xzf "$archive" -C "$work/frozen"
    MARQDO_BIN="$work/frozen/$artifact/marqdo"
    "$frozen" verify "$MARQDO_BIN"
else
    MARQDO_BIN="$("$frozen" fetch "$work/frozen")"
    archive="$work/frozen/$artifact.tar.gz"
fi
export MARQDO_BIN
source_sha="$(git -C cli rev-parse HEAD)"
test "$(cat "$(dirname "$MARQDO_BIN")/SOURCE_SHA")" = "$source_sha"
sha256sum "$archive" "$MARQDO_BIN" "$(dirname "$MARQDO_BIN")/marqdo-provider-worker"

python3 cli/tools/release/smoke.py --archive "$archive"
python3 cli/tools/ci/verify_extension_contracts.py \
    --source-repo cli --tested-sha "$source_sha" --cli "$MARQDO_BIN" \
    --release-info "$(dirname "$MARQDO_BIN")/RELEASE_INFO.json" \
    --fixtures tests/conformance --out "$work/conformance.json"

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

echo "PASS: ecosystem RC.4 candidate checks"
