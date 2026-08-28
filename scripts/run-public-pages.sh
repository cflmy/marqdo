#!/usr/bin/env bash
# Run every public/*.mq.md that defines `# main`, then export the site.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# CLI + native plugins used by public demos (ext/quantum).
cargo build --release -p marqdo -p marqdo_plugin_quantum

BIN=./target/release/marqdo
while IFS= read -r -d '' f; do
  if grep -q '^# main' "$f"; then
    echo "+ marqdo run $f"
    "$BIN" run "$f"
  else
    echo "- skip (no # main): $f"
  fi
done < <(find public -name '*.mq.md' -print0)

"$BIN" view output public -o public
