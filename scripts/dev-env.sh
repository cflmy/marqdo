#!/usr/bin/env bash
# Marqdo dev environment for this sandbox (glibc 2.36 host, no system gcc/python3/go).
#
# Usage:  source scripts/dev-env.sh
#
# Everything lives under target/toolchain/ (bundled): rust 1.81, zig 0.13 (as cc),
# go 1.25, python 3.12. Zig also solves the "release binary needs GLIBC_2.39"
# problem by targeting x86_64-linux-gnu.2.36.
#
# Two non-obvious constraints this script encodes:
#   1. $HOME is not writable -> zig/rust caches must be redirected explicitly.
#   2. /tmp is a noexec tmpfs -> anything a build *executes* from a temp dir
#      fails with "AccessDenied" (zig link step) or "signal: aborted" (go link
#      step via gcc). TMPDIR must point at a writable+executable dir.

_marqdo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
_tc="$_marqdo_root/target/toolchain"
_zshim="/workspace/qdagent/.tools/zigcc"

export RUSTUP_HOME="$_tc/rustup"
export CARGO_HOME="$_tc/cargo"
export GOCACHE="$_tc/gocache"
export GOPATH="$_tc/gopath"
export TMPDIR="$_tc/gotmp"
export ZIG_GLOBAL_CACHE_DIR="/workspace/qdagent/.tools/zigcache"
export ZIG_LOCAL_CACHE_DIR="/workspace/qdagent/.tools/zigcache"

export PATH="$_zshim/bin:$_tc/python/bin:$_tc/go/bin:$_tc/cargo/bin:$PATH"

# Rust: link through zig cc pinned to glibc 2.36 so the binary runs on this host.
export CC="$_zshim/cc"
export CXX="$_zshim/cc"
export AR="$_zshim/ar"
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="$_zshim/cc"

mkdir -p "$TMPDIR" "$GOCACHE" "$GOPATH" "$ZIG_GLOBAL_CACHE_DIR"

echo "marqdo dev-env: rust=$(rustc --version 2>/dev/null | cut -d' ' -f2) go=$(go version 2>/dev/null | awk '{print $3}') python=$(python3 --version 2>&1 | awk '{print $2}') cc=zig-glibc2.36"
