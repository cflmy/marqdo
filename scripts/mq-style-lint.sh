#!/usr/bin/env bash
# O4 — style gate: flag json.set dict chains and mixed EN/ZH API hints in .mq.md.
# Exit 0 when clean; 1 when findings (unless MQ_STYLE_LINT_SOFT=1).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
TARGETS=("$@")
if [[ ${#TARGETS[@]} -eq 0 ]]; then
  TARGETS=(ext/ai/sandbox.mq.md ext/ai/policies examples/agent-sandbox-okf)
fi

fail=0
findings=0

scan_file() {
  local f="$1"
  [[ -f "$f" ]] || return 0
  # Long json.set pipelines (3+ consecutive) — style bug per marqdo-dev.
  if awk '
    /json\.set/ { c++; if (c >= 3) { found=1 } next }
    { c=0 }
    END { exit found ? 0 : 1 }
  ' "$f"; then
    echo "STYLE json.set-chain: $f" >&2
    findings=$((findings + 1))
    fail=1
  fi
  # Same file imports both EN agent and ZH 智能体 (or web/网页) — mixed surface.
  if grep -qE 'import[[:space:]]+agent:|import[[:space:]]+web:' "$f" \
    && grep -qE '导入[[:space:]]+(智能体|网页):' "$f"; then
    echo "STYLE mixed-EN/ZH-import: $f" >&2
    findings=$((findings + 1))
    fail=1
  fi
}

while IFS= read -r -d '' f; do
  scan_file "$f"
done < <(find "${TARGETS[@]}" -type f -name '*.mq.md' -print0 2>/dev/null)

echo "mq-style-lint: findings=$findings"
if [[ "${MQ_STYLE_LINT_SOFT:-}" == "1" ]]; then
  exit 0
fi
exit "$fail"
