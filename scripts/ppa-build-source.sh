#!/usr/bin/env bash
# Build a signed Ubuntu *source* package for Launchpad PPA.
# Usage: ./scripts/ppa-build-source.sh [series]
# Example: ./scripts/ppa-build-source.sh noble
set -euo pipefail

SERIES="${1:-noble}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

need() { command -v "$1" >/dev/null || { echo "missing tool: $1 (sudo apt install $2)"; exit 1; }; }
need cargo cargo
need debuild devscripts
need dpkg-parsechangelog dpkg-dev

VER="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
DEB_VER="${VER}-1ppa1~${SERIES}"
PKG_ROOT="${TMPDIR:-/tmp}/marqdo-ppa/${VER}"
BUILD_AREA="${ROOT}/../build-area"
GPG_KEY="${MARQDO_PPA_GPG_KEY:-505943294D04C803}"

echo "==> upstream ${VER} → debian ${DEB_VER} (${SERIES})"
rm -rf "$PKG_ROOT"
mkdir -p "$PKG_ROOT" "$BUILD_AREA"

# Clean export of sources (no target/, .git)
if command -v git >/dev/null && git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  git archive --format=tar HEAD | tar -x -C "$PKG_ROOT"
else
  rsync -a --exclude target --exclude .git --exclude build-area "$ROOT"/ "$PKG_ROOT"/
fi

# Ensure debian/ from working tree (may be newer than last commit)
rsync -a "$ROOT/debian/" "$PKG_ROOT/debian/"

# Series-specific changelog entry (rewrite top distribution)
cd "$PKG_ROOT"
# Update changelog top line distribution / version if needed
python3 - <<PY
from pathlib import Path
import re
p = Path("debian/changelog")
text = p.read_text(encoding="utf-8")
text2 = re.sub(
    r"^marqdo \([^)]+\) [^;]+;",
    "marqdo (${DEB_VER}) ${SERIES};",
    text,
    count=1,
    flags=re.M,
)
p.write_text(text2, encoding="utf-8")
print("changelog:", p.read_text(encoding="utf-8").splitlines()[0])
PY

echo "==> cargo vendor (Launchpad offline)"
mkdir -p .cargo
cargo vendor --locked vendor >/tmp/marqdo-cargo-vendor.conf
# cargo vendor prints config to stdout; also write known-good offline config
cat > .cargo/config.toml <<'EOF'
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF

# Orig tarball: upstream version without debian/
cd "$(dirname "$PKG_ROOT")"
ORIG="marqdo_${VER}.orig.tar.xz"
if [ ! -f "$BUILD_AREA/$ORIG" ]; then
  tar -cJf "$BUILD_AREA/$ORIG" \
    --exclude=debian \
    -C "$(dirname "$PKG_ROOT")" "$(basename "$PKG_ROOT")"
fi
# debuild expects ../marqdo_VER.orig.tar.xz relative to package dir parent
ln -sfn "$BUILD_AREA/$ORIG" "marqdo_${VER}.orig.tar.xz"

cd "$PKG_ROOT"
echo "==> debuild -S (key ${GPG_KEY})"
debuild -S -sa -k"${GPG_KEY}" -d

# debuild writes next to package parent (/tmp/marqdo-ppa/); collect into build-area
mkdir -p "$BUILD_AREA"
shopt -s nullglob
for f in ../marqdo_"${DEB_VER}"* ../marqdo_"${VER}".orig.tar.*; do
  [ -e "$f" ] || continue
  # orig may already be a symlink into BUILD_AREA — skip same inode
  if [ "$(realpath -m "$f")" = "$(realpath -m "$BUILD_AREA/$(basename "$f")")" ]; then
    continue
  fi
  cp -f "$f" "$BUILD_AREA/"
done
shopt -u nullglob

echo "==> artifacts in $BUILD_AREA:"
ls -la "$BUILD_AREA"/marqdo_"${DEB_VER}"* "$BUILD_AREA"/marqdo_"${VER}".orig.tar.* 2>/dev/null || true
CHANGES="$BUILD_AREA/marqdo_${DEB_VER}_source.changes"
if [ -f "$CHANGES" ]; then
  echo
  echo "Upload with:"
  echo "  dput ppa:cflmy/marqdo $CHANGES"
  echo "  # or: ./scripts/ppa-ship.sh ${SERIES}"
fi
