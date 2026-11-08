#!/usr/bin/env bash
# Generate HTML next to public/*.mq.md via the Marqdo interpreter.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release -q
./target/release/marqdo view output public -o public
echo "OK → public/index.html + public/pages/  (sources: public/**/*.mq.md)"
