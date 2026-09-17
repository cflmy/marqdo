#!/usr/bin/env bash
# One-shot: install packaging tools → build signed source package → dput to Launchpad PPA.
#
# Run in YOUR terminal (needs sudo for apt + GPG passphrase for signing):
#   ./scripts/ppa-ship.sh              # default series: noble
#   ./scripts/ppa-ship.sh resolute     # match host /etc/os-release
#   ./scripts/ppa-ship.sh noble --no-upload
#   sudo ./scripts/ppa-ship.sh noble   # also OK — build/sign/upload drop back to $SUDO_USER
#
# PPA: ppa:cflmy/marqdo  |  key: 505943294D04C803
set -euo pipefail

SERIES="${1:-noble}"
NO_UPLOAD=0
for arg in "${@:2}"; do
  case "$arg" in
    --no-upload|-n) NO_UPLOAD=1 ;;
    *) echo "unknown arg: $arg" >&2; exit 2 ;;
  esac
done

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Never cargo/debuild/dput as root — GPG lives in the human user's ~/.gnupg
if [ "$(id -u)" -eq 0 ]; then
  if [ -z "${SUDO_USER:-}" ] || [ "$SUDO_USER" = root ]; then
    echo "error: run as a normal user, or: sudo -u \$USER ./scripts/ppa-ship.sh" >&2
    exit 1
  fi
  REAL_USER="$SUDO_USER"
  REAL_HOME="$(getent passwd "$REAL_USER" | cut -d: -f6)"
  echo "==> root detected; apt as root, build/upload as ${REAL_USER}"
  run_user() {
    local upath="${REAL_HOME}/.cargo/bin:/usr/local/bin:/usr/bin:/bin"
    sudo -u "$REAL_USER" -H --preserve-env=MARQDO_PPA_GPG_KEY \
      env HOME="$REAL_HOME" PATH="$upath" "$@"
  }
  APT=(apt-get)
else
  REAL_USER="$(id -un)"
  REAL_HOME="$HOME"
  run_user() { "$@"; }
  APT=(sudo apt-get)
fi

PPA_TARGET="${MARQDO_PPA:-ppa:cflmy/marqdo}"
PKGS=(build-essential debhelper devscripts dput lintian
      dpkg-dev quilt cargo rustc pkg-config libssl-dev ca-certificates
      python3 xz-utils)

need_apt=0
for bin in debuild dput dpkg-parsechangelog cargo gpg; do
  if ! command -v "$bin" >/dev/null 2>&1; then
    need_apt=1
    break
  fi
done
# dh_auto_* comes from debhelper even if debuild exists from a partial install
if ! dpkg -s debhelper >/dev/null 2>&1; then
  need_apt=1
fi

if [ "$need_apt" -eq 1 ]; then
  echo "==> installing packaging toolchain (sudo)…"
  "${APT[@]}" update -qq
  DEBIAN_FRONTEND=noninteractive "${APT[@]}" install -y "${PKGS[@]}"
else
  echo "==> packaging tools already present"
fi

echo "==> building source package for series=${SERIES}"
run_user "$ROOT/scripts/ppa-build-source.sh" "$SERIES"

VER="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -1)"
DEB_VER="${VER}-1ppa1~${SERIES}"
BUILD_AREA="$(cd "$ROOT/.." && pwd)/build-area"
CHANGES="${BUILD_AREA}/marqdo_${DEB_VER}_source.changes"

# ppa-build may leave artifacts next to /tmp/marqdo-ppa — copy already done; also scan
if [ ! -f "$CHANGES" ]; then
  alt="$(ls -1 "$BUILD_AREA"/marqdo_*_source.changes 2>/dev/null | tail -1 || true)"
  if [ -n "${alt:-}" ]; then
    CHANGES="$alt"
  else
    echo "error: no *_source.changes under $BUILD_AREA" >&2
    ls -la "$BUILD_AREA" 2>/dev/null || true
    exit 1
  fi
fi

echo "==> artifact: $CHANGES"
if command -v lintian >/dev/null 2>&1; then
  echo "==> lintian (non-fatal)"
  run_user lintian -i "$CHANGES" || true
fi

if [ "$NO_UPLOAD" -eq 1 ]; then
  echo
  echo "Skip upload (--no-upload). When ready:"
  echo "  dput ${PPA_TARGET} ${CHANGES}"
  exit 0
fi

echo "==> dput ${PPA_TARGET}"
run_user dput --unchecked "${PPA_TARGET}" "$CHANGES"

echo
echo "Done. Watch builds:"
echo "  https://launchpad.net/~cflmy/+archive/ubuntu/marqdo/+builds"
echo "Enable the Ubuntu series (${SERIES}) in PPA → Change details if Launchpad rejects the upload."
