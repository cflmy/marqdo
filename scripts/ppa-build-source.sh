#!/usr/bin/env bash
# Build a signed Ubuntu *source* package for Launchpad PPA.
# Usage: ./scripts/ppa-build-source.sh [series]
# Example: ./scripts/ppa-build-source.sh noble
#
# Launchpad builders have no network and noble's cargo (1.75) cannot read
# Cargo.lock v4. This script therefore:
#   1) cargo vendor → vendor/
#   2) installs a modern Rust toolchain into third_party/rust (cached)
#   3) packs both into the .orig.tar.xz for offline debuild on Launchpad
set -euo pipefail

SERIES="${1:-noble}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

need() { command -v "$1" >/dev/null || { echo "missing tool: $1 (sudo apt install $2)"; exit 1; }; }
need cargo cargo
need debuild devscripts
need dpkg-parsechangelog dpkg-dev
need curl curl
need tar tar
need xz xz-utils

VER="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
# Debian revision comes from debian/changelog (e.g. 1.0.2-1ppa2); we append ~series.
CHG_VER="$(dpkg-parsechangelog -l debian/changelog -S Version)"
# Strip any previous ~series suffix from changelog version
BASE_DEB_VER="${CHG_VER%%~*}"
# If changelog still has old upstream, prefer Cargo VER for the upstream part
if [[ "$BASE_DEB_VER" != "$VER"-* ]]; then
  # keep debian revision suffix after first '-' following VER if possible
  REV="${BASE_DEB_VER#*-}"
  if [[ "$BASE_DEB_VER" == *"-"* ]]; then
    BASE_DEB_VER="${VER}-${REV}"
  else
    BASE_DEB_VER="${VER}-1ppa1"
  fi
fi
DEB_VER="${BASE_DEB_VER}~${SERIES}"
PKG_ROOT="${TMPDIR:-/tmp}/marqdo-ppa/${VER}"
BUILD_AREA="${ROOT}/../build-area"
GPG_KEY="${MARQDO_PPA_GPG_KEY:-505943294D04C803}"
RUST_VER="${MARQDO_PPA_RUST_VERSION:-1.85.0}"
RUST_TRIPLE="${MARQDO_PPA_RUST_TRIPLE:-x86_64-unknown-linux-gnu}"
RUST_NAME="rust-${RUST_VER}-${RUST_TRIPLE}"
RUST_CACHE="${MARQDO_PPA_RUST_CACHE:-${HOME}/.cache/marqdo-ppa}"
RUST_TARBALL="${RUST_CACHE}/${RUST_NAME}.tar.gz"
RUST_URL="${MARQDO_PPA_RUST_URL:-https://static.rust-lang.org/dist/${RUST_NAME}.tar.gz}"

echo "==> upstream ${VER} → debian ${DEB_VER} (${SERIES})"
rm -rf "$PKG_ROOT"
mkdir -p "$PKG_ROOT" "$BUILD_AREA" "$RUST_CACHE"

# Clean export of sources (no target/, .git)
if command -v git >/dev/null && git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  git archive --format=tar HEAD | tar -x -C "$PKG_ROOT"
else
  rsync -a --exclude target --exclude .git --exclude build-area "$ROOT"/ "$PKG_ROOT"/
fi

# Ensure debian/ from working tree (may be newer than last commit)
rsync -a "$ROOT/debian/" "$PKG_ROOT/debian/"

cd "$PKG_ROOT"
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
cat > .cargo/config.toml <<'EOF'
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF

echo "==> bundle Rust ${RUST_VER} (${RUST_TRIPLE}) for Launchpad"
if [ ! -f "$RUST_TARBALL" ]; then
  echo "    downloading ${RUST_URL}"
  curl -fL --retry 3 --retry-delay 2 -o "${RUST_TARBALL}.partial" "$RUST_URL"
  mv "${RUST_TARBALL}.partial" "$RUST_TARBALL"
else
  echo "    cache hit ${RUST_TARBALL}"
fi
rm -rf third_party/rust
mkdir -p third_party
RUST_EXTRACT="$(mktemp -d "${TMPDIR:-/tmp}/marqdo-rust.XXXXXX")"
tar -xzf "$RUST_TARBALL" -C "$RUST_EXTRACT"
# Official dist tarball contains install.sh
INSTALL_SH="$(find "$RUST_EXTRACT" -maxdepth 2 -name install.sh | head -1)"
if [ -z "$INSTALL_SH" ]; then
  echo "error: install.sh not found in ${RUST_TARBALL}" >&2
  exit 1
fi
# --disable-ldconfig avoids needing root; strip docs to save space
bash "$INSTALL_SH" \
  --prefix="$PKG_ROOT/third_party/rust" \
  --without=rust-docs \
  --disable-ldconfig
rm -rf "$RUST_EXTRACT"
"$PKG_ROOT/third_party/rust/bin/rustc" --version
"$PKG_ROOT/third_party/rust/bin/cargo" --version

# Always rebuild orig (vendor + rust toolchain change every run)
cd "$(dirname "$PKG_ROOT")"
ORIG="marqdo_${VER}.orig.tar.xz"
echo "==> packing ${ORIG} (vendor + third_party/rust)"
rm -f "$BUILD_AREA/$ORIG" "marqdo_${VER}.orig.tar.xz"
tar -cJf "$BUILD_AREA/$ORIG" \
  --exclude=debian \
  -C "$(dirname "$PKG_ROOT")" "$(basename "$PKG_ROOT")"
ln -sfn "$BUILD_AREA/$ORIG" "marqdo_${VER}.orig.tar.xz"
ls -lh "$BUILD_AREA/$ORIG"

cd "$PKG_ROOT"
echo "==> debuild -S (key ${GPG_KEY})"
debuild -S -sa -k"${GPG_KEY}" -d

# debuild writes next to package parent (/tmp/marqdo-ppa/); collect into build-area
mkdir -p "$BUILD_AREA"
shopt -s nullglob
for f in ../marqdo_"${DEB_VER}"* ../marqdo_"${VER}".orig.tar.*; do
  [ -e "$f" ] || continue
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
