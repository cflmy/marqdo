#!/usr/bin/env bash
# Build Marqdo web plugin (Go → C shared library).
# Canonical output: plugins/web/build/libweb.so (Linux) / libweb.dylib / web.dll
# Also copies into target/{debug,release}/ for ext CLI and release packaging.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/plugins/web"
mkdir -p build
export PATH="${HOME}/.local/go/bin:/usr/local/go/bin:${PATH}"
if ! command -v go >/dev/null 2>&1; then
  echo "error: go not found (install Go 1.22+ or set PATH)" >&2
  exit 1
fi
OUT="build/libweb.so"
case "$(uname -s)" in
  Darwin) OUT="build/libweb.dylib" ;;
  MINGW*|MSYS*|CYGWIN*|Windows_NT) OUT="build/web.dll" ;;
esac
echo "building $OUT (go $(go version | awk '{print $3}'))…"
CGO_ENABLED=1 go build -buildmode=c-shared -o "$OUT" .
ABS="$ROOT/plugins/web/$OUT"
BASE="$(basename "$OUT")"
for dir in "$ROOT/target/debug" "$ROOT/target/release"; do
  mkdir -p "$dir"
  cp -f "$ABS" "$dir/$BASE"
done
echo "ok: $ABS"
echo "also: $ROOT/target/debug/$BASE  $ROOT/target/release/$BASE"
echo "hint: plugin.native_path name=web resolves these; override with MARQDO_WEB_PLUGIN=$ABS"
