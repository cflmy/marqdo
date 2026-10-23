#!/usr/bin/env bash
# Marqdo dev environment.
#
# Usage:  source scripts/dev-env.sh
#
# If a bundled toolchain is present under target/toolchain/, it is preferred;
# otherwise the script keeps using tools already installed on PATH. This makes
# the script work both in the maintained sandbox image and on a normal dev host.
#
# Two non-obvious constraints this script encodes:
#   1. $HOME is not writable -> zig/rust caches must be redirected explicitly.
#   2. /tmp is a noexec tmpfs -> anything a build *executes* from a temp dir
#      fails with "AccessDenied" (zig link step) or "signal: aborted" (go link
#      step via gcc). TMPDIR must point at a writable+executable dir.

_marqdo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
_tc="$_marqdo_root/target/toolchain"
_zshim="${MARQDO_ZIG_SHIM:-$_marqdo_root/.tools/zigcc}"

_prepend_existing() {
  [ -d "$1" ] && PATH="$1:$PATH"
}

export RUSTUP_HOME="${RUSTUP_HOME:-$_tc/rustup}"
export CARGO_HOME="${CARGO_HOME:-$_tc/cargo}"
export GOCACHE="${GOCACHE:-$_tc/gocache}"
export GOPATH="${GOPATH:-$_tc/gopath}"
export TMPDIR="${TMPDIR:-$_tc/tmp}"
export ZIG_GLOBAL_CACHE_DIR="${ZIG_GLOBAL_CACHE_DIR:-$_tc/zigcache}"
export ZIG_LOCAL_CACHE_DIR="${ZIG_LOCAL_CACHE_DIR:-$_tc/zigcache}"

_prepend_existing "$_zshim/bin"
_prepend_existing "$_tc/python/bin"
_prepend_existing "$_tc/go/bin"
_prepend_existing "$_tc/cargo/bin"

# Rust: link through zig cc pinned to glibc 2.36 so the binary runs on this host.
if [ -x "$_zshim/cc" ] && [ -x "$_zshim/ar" ]; then
  export CC="$_zshim/cc"
  export CXX="$_zshim/cc"
  export AR="$_zshim/ar"
  export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="$_zshim/cc"
fi

mkdir -p "$TMPDIR" "$GOCACHE" "$GOPATH" "$ZIG_GLOBAL_CACHE_DIR"

echo "marqdo dev-env: rust=$(rustc --version 2>/dev/null | cut -d' ' -f2 || echo missing) go=$(go version 2>/dev/null | awk '{print $3}' || echo missing) python=$(python3 --version 2>&1 | awk '{print $2}' || echo missing)"
