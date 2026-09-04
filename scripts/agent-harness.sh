#!/usr/bin/env bash
# Wave B1 — offline constitution / OKF harness (no network required).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
MQ="${MARQDO_BIN:-$ROOT/target/release/marqdo}"
export MARQDO_EXT="${MARQDO_EXT:-$ROOT/.marqdo-ext}"
export MARQDO_AGENT_PLUGIN="${MARQDO_AGENT_PLUGIN:-$ROOT/.marqdo-ext/native/libagent.so}"

if [[ ! -x "$MQ" ]]; then
  echo "missing $MQ — run: cargo build --release -p marqdo -p marqdo_plugin_agent" >&2
  exit 1
fi

fail=0
run() {
  local f="$1"
  echo "==> $f"
  if ! "$MQ" run "$f"; then
    echo "FAIL $f" >&2
    fail=1
  fi
}

run tests/ext/agent-constitution-b0.mq.md
run tests/ext/agent-kb-plan-hit.mq.md
run tests/ext/agent-context-budget-a3.mq.md
run tests/ext/agent-workbook-patch-a0.mq.md
run examples/agent-okf-flywheel/index.mq.md
run examples/agent-pong/index.mq.md

if [[ "${AGENT_HARNESS_LIVE:-}" == "1" ]]; then
  export AGENT_LIVE=1
  run examples/agent-pong/index.mq.md
  run tests/ext/agent-run-live.mq.md
fi

if [[ "$fail" -ne 0 ]]; then
  echo "agent harness: FAILED" >&2
  exit 1
fi
echo "agent harness: OK"
