#!/usr/bin/env bash
# Build a signed Ubuntu *source* package for Launchpad PPA.
# Usage: ./scripts/ppa-build-source.sh [series]
# Example: ./scripts/ppa-build-source.sh noble
#
# Launchpad builders have no network and noble's cargo (1.75) cannot read
# Cargo.lock v4. For a *new* upstream SemVer this script:
#   1) cargo vendor → vendor/
#   2) installs a modern Rust toolchain into third_party/rust (cached)
#   3) packs both into the .orig.tar.xz for offline debuild on Launchpad
#
# For debian-only bumps (same upstream X.Y.Z already in the PPA):
#   - Reuse the published marqdo_X.Y.Z.orig.tar.xz (never rewrite it)
#   - debuild -S -sd (do not re-upload .orig)
# Override: MARQDO_PPA_FORCE_ORIG=1 rebuilds orig; MARQDO_PPA_ORIG=/path uses that file.
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
PPA_OWNER="${MARQDO_PPA_OWNER:-cflmy}"
PPA_NAME="${MARQDO_PPA_NAME:-marqdo}"
ORIG_NAME="marqdo_${VER}.orig.tar.xz"
FORCE_ORIG="${MARQDO_PPA_FORCE_ORIG:-0}"

resolve_existing_orig() {
  local candidate
  if [ -n "${MARQDO_PPA_ORIG:-}" ]; then
    candidate="${MARQDO_PPA_ORIG}"
    if [ ! -f "$candidate" ]; then
      echo "error: MARQDO_PPA_ORIG not a file: $candidate" >&2
      exit 1
    fi
    echo "$candidate"
    return 0
  fi
  for candidate in \
    "$BUILD_AREA/$ORIG_NAME" \
    "$RUST_CACHE/$ORIG_NAME" \
    "$HOME/.cache/marqdo-ppa/$ORIG_NAME"
  do
    if [ -f "$candidate" ]; then
      echo "$candidate"
      return 0
    fi
  done
  return 1
}

download_published_orig() {
  local dest="$1"
  local urls=(
    "https://ppa.launchpadcontent.net/${PPA_OWNER}/${PPA_NAME}/ubuntu/pool/main/m/marqdo/${ORIG_NAME}"
    "https://launchpad.net/~${PPA_OWNER}/+archive/ubuntu/${PPA_NAME}/+files/${ORIG_NAME}"
  )
  local url
  mkdir -p "$(dirname "$dest")"
  for url in "${urls[@]}"; do
    echo "    trying ${url}"
    if curl -fL --retry 3 --retry-delay 2 --connect-timeout 30 \
      -o "${dest}.partial" "$url"; then
      mv "${dest}.partial" "$dest"
      return 0
    fi
    rm -f "${dest}.partial"
  done
  return 1
}

echo "==> upstream ${VER} → debian ${DEB_VER} (${SERIES})"
rm -rf "$PKG_ROOT"
mkdir -p "$PKG_ROOT" "$BUILD_AREA" "$RUST_CACHE"

REUSE_ORIG=0
ORIG_SRC=""
if [ "$FORCE_ORIG" != "1" ]; then
  # Prefer the bytes already accepted by Launchpad (rewriting .orig is rejected).
  if download_published_orig "$RUST_CACHE/$ORIG_NAME"; then
    ORIG_SRC="$RUST_CACHE/$ORIG_NAME"
  elif ORIG_SRC="$(resolve_existing_orig)"; then
    echo "warning: could not fetch PPA ${ORIG_NAME}; using local ${ORIG_SRC}" >&2
    echo "warning: if its sha256 differs from the archive, Launchpad will reject the upload" >&2
  fi
  if [ -n "$ORIG_SRC" ]; then
    REUSE_ORIG=1
  fi
fi

if [ "$REUSE_ORIG" -eq 1 ]; then
  echo "==> reusing published ${ORIG_NAME} (debian-only bump; do not rewrite orig)"
  echo "    source: ${ORIG_SRC}"
  sha256sum "$ORIG_SRC"
  cp -f "$ORIG_SRC" "$BUILD_AREA/$ORIG_NAME"
  # Unpack orig (top dir is usually ${VER} or marqdo-${VER})
  tar -xJf "$BUILD_AREA/$ORIG_NAME" -C "$(dirname "$PKG_ROOT")"
  if [ ! -d "$PKG_ROOT" ]; then
    # Some tarballs use marqdo-${VER}/
    if [ -d "$(dirname "$PKG_ROOT")/marqdo-${VER}" ]; then
      mv "$(dirname "$PKG_ROOT")/marqdo-${VER}" "$PKG_ROOT"
    else
      echo "error: unpack did not create ${PKG_ROOT}" >&2
      tar -tJf "$BUILD_AREA/$ORIG_NAME" | head -5 >&2
      exit 1
    fi
  fi
  rsync -a "$ROOT/debian/" "$PKG_ROOT/debian/"
else
  echo "==> building fresh ${ORIG_NAME} (new upstream or MARQDO_PPA_FORCE_ORIG=1)"
  # Clean export of sources (no target/, .git)
  if command -v git >/dev/null && git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    git archive --format=tar HEAD | tar -x -C "$PKG_ROOT"
  else
    rsync -a --exclude target --exclude .git --exclude build-area "$ROOT"/ "$PKG_ROOT"/
  fi
  rsync -a "$ROOT/debian/" "$PKG_ROOT/debian/"

  cd "$PKG_ROOT"
  echo "==> cargo vendor (Launchpad offline)"
  mkdir -p .cargo
  cargo vendor --locked vendor >/tmp/marqdo-cargo-vendor.conf
  cat > .cargo/config.toml <<'EOF'
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF
  # dh_clean removes *.orig; drop those paths from vendor checksums so a
  # stray clean (or older rules) cannot break `cargo --offline` on Launchpad.
  python3 - <<'PY'
from __future__ import annotations

import json
from pathlib import Path

root = Path("vendor")
if not root.is_dir():
    raise SystemExit("vendor/ missing after cargo vendor")
n_files = 0
n_json = 0
for checksum in root.rglob(".cargo-checksum.json"):
    data = json.loads(checksum.read_text(encoding="utf-8"))
    files = data.get("files") or {}
    drop = [name for name in files if name.endswith(".orig") or name.endswith(".rej")]
    if not drop:
        continue
    for name in drop:
        del files[name]
        path = checksum.parent / name
        if path.is_file():
            path.unlink()
            n_files += 1
    checksum.write_text(json.dumps(data, indent=4) + "\n", encoding="utf-8")
    n_json += 1
print(f"stripped {n_files} *.orig/*.rej from {n_json} vendor checksums")
PY

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
  INSTALL_SH="$(find "$RUST_EXTRACT" -maxdepth 2 -name install.sh | head -1)"
  if [ -z "$INSTALL_SH" ]; then
    echo "error: install.sh not found in ${RUST_TARBALL}" >&2
    exit 1
  fi
  bash "$INSTALL_SH" \
    --prefix="$PKG_ROOT/third_party/rust" \
    --without=rust-docs \
    --disable-ldconfig
  rm -rf "$RUST_EXTRACT"
  "$PKG_ROOT/third_party/rust/bin/rustc" --version
  "$PKG_ROOT/third_party/rust/bin/cargo" --version

  cd "$(dirname "$PKG_ROOT")"
  echo "==> packing ${ORIG_NAME} (vendor + third_party/rust)"
  rm -f "$BUILD_AREA/$ORIG_NAME"
  tar -cJf "$BUILD_AREA/$ORIG_NAME" \
    --exclude=debian \
    -C "$(dirname "$PKG_ROOT")" "$(basename "$PKG_ROOT")"
fi

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

ln -sfn "$BUILD_AREA/$ORIG_NAME" "$(dirname "$PKG_ROOT")/$ORIG_NAME"
ls -lh "$BUILD_AREA/$ORIG_NAME"

cd "$PKG_ROOT"
# -sd: debian-only upload (reuse published orig). -sa: include orig (new upstream).
if [ "$REUSE_ORIG" -eq 1 ]; then
  DEBUILD_ORIG_FLAG="-sd"
  echo "==> debuild -S ${DEBUILD_ORIG_FLAG} (key ${GPG_KEY}; not re-uploading .orig)"
else
  DEBUILD_ORIG_FLAG="-sa"
  echo "==> debuild -S ${DEBUILD_ORIG_FLAG} (key ${GPG_KEY}; including new .orig)"
fi
debuild -S ${DEBUILD_ORIG_FLAG} -k"${GPG_KEY}" -d

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
if [ "$REUSE_ORIG" -eq 1 ]; then
  echo "    sha256 of reused orig (must match PPA):"
  sha256sum "$BUILD_AREA/$ORIG_NAME"
fi
CHANGES="$BUILD_AREA/marqdo_${DEB_VER}_source.changes"
if [ -f "$CHANGES" ]; then
  echo
  echo "Upload with:"
  echo "  dput ppa:cflmy/marqdo $CHANGES"
  echo "  # or: ./scripts/ppa-ship.sh ${SERIES}"
fi
