#!/usr/bin/env bash
# O5 — offline standing / policy rewrite hint (GEPA-like, products stay repo files).
# Does NOT change the runtime agent path. Reads harness failure log + current files,
# writes a suggested patch under .marqdo/optimize/ for human review.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
LOG="${1:-}"
STANDING="${2:-ext/ai/agent.mq.md}"
POLICY="${3:-ext/ai/policies/step-route.mq.md}"
OUT_DIR="${OUT_DIR:-.marqdo/optimize}"
mkdir -p "$OUT_DIR"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
REPORT="$OUT_DIR/standing-policy-$STAMP.md"

{
  echo "# Offline standing / policy optimize suggestion"
  echo
  echo "Generated: $STAMP"
  echo
  echo "This is a **review artifact**, not an automatic runtime rewrite."
  echo "Strong models may rewrite \`standing\` / \`policies/*.mq.md\` offline;"
  echo "commit only after harness green."
  echo
  echo "## Inputs"
  echo
  echo "- standing source: \`$STANDING\`"
  echo "- policy source: \`$POLICY\`"
  if [[ -n "$LOG" && -f "$LOG" ]]; then
    echo "- harness log: \`$LOG\`"
    echo
    echo "## Harness failure excerpt"
    echo
    echo '```'
    tail -n 80 "$LOG" || true
    echo '```'
  else
    echo "- harness log: _(none — pass path as \$1)_"
  fi
  echo
  echo "## Current policy table (copy)"
  echo
  if [[ -f "$POLICY" ]]; then
    echo '```markdown'
    cat "$POLICY"
    echo '```'
  else
    echo "_missing $POLICY_"
  fi
  echo
  echo "## Suggested next edits"
  echo
  echo "1. Keep step-class → backend as a **GFM table** (no json glue)."
  echo "2. Quarantine failing skills (\`status: quarantine\`) before promoting again."
  echo "3. Re-run \`scripts/agent-harness.sh\` and confirm Skill Lift summary."
} > "$REPORT"

echo "wrote $REPORT"
