#!/usr/bin/env bash
# Build Marqdo web plugin (Go → C shared library).
# Output: plugins/web/build/libweb.so (Linux) / libweb.dylib / web.dll
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
# Do NOT copy over target/{debug,release}/libweb.so — rust archive crate still
# builds there during migration. Canonical Go artifact is plugins/web/build/.
echo "ok: $ROOT/plugins/web/$OUT"
echo "hint: export MARQDO_WEB_PLUGIN=$ROOT/plugins/web/$OUT"
echo "      (or use plugins/web/build before ~/.marqdo/ext/native in resolution)"
