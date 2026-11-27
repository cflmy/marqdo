#!/usr/bin/env bash
# Build and optionally publish the shared Marqdo CI images.
#
# Usage:
#   MARQDO_GIT_TOKEN=... ci/images/build.sh [--push]
#
# The images are immutable per released CLI version. They are the only shared
# CI runtime images: shell/source components use `marqdo-ci-shell`, Rust
# components use `marqdo-ci-rust`, the Web domain uses `marqdo-ci-web`.
# Core CLI/root keep their own specialized gates and images.
set -euo pipefail
if ! command -v cargo >/dev/null 2>&1; then
  export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
fi
root="$(cd "$(dirname "$0")/../.." && pwd)"
toolchain="${MARQDO_CI_RUST_TOOLCHAIN:-$(rustc --print sysroot)}"
cargo_cache="${MARQDO_CI_CARGO_CACHE:-${CARGO_HOME:-$HOME/.cargo}}"
preview_checkout="${MARQDO_CI_PREVIEW_CHECKOUT:-$root/extensions/web/preview}"
version="${MARQDO_CLI_VERSION:-3.0.0-rc.5}"
registry="${MARQDO_CI_REGISTRY:-git.cflmy.cn/cflmy}"
commit="${MARQDO_CI_IMAGE_COMMIT:-$(git -C "$root" rev-parse --short HEAD)}"
context="$root/.build/ci-images/$commit-$version"
push=0
[[ "${1:-}" == "--push" ]] && push=1

archive_url="https://git.cflmy.cn/cflmy/marqdo-cli/releases/download/v${version}/marqdo-${version}-x86_64-unknown-linux-gnu.tar.gz"
archive_sha="fd7a313e0ddaf38a9b1901ff2a9aa7b9a0a153e65522a9c0c60f01e2549f583b"

rm -rf "$context"; mkdir -p "$context"
echo "==> download published CLI/Bundle"
curl -fsSL --retry 3 -o "$context/marqdo.tar.gz" "$archive_url"
echo "$archive_sha  $context/marqdo.tar.gz" | sha256sum -c -

echo "==> package the preview web Tool from the pinned preview checkout"
(
  cd "$preview_checkout"
  scripts/package.sh >/dev/null
)
tar -czf "$context/marqdo-web-preview.tgz" -C "$preview_checkout/dist" marqdo-web-preview

echo "==> stage the pinned Rust toolchain and cargo cache"
cp -a "$toolchain" "$context/rust-toolchain"
cp -a "$cargo_cache" "$context/cargo-home"
cp "$root/ci/images/LICENSE" "$context/marqdo-ci-shell-license" 2>/dev/null || echo "internal CI image" > "$context/marqdo-ci-shell-license"
cp "$root/ci/images/Dockerfile.shell" "$root/ci/images/Dockerfile.rust" "$root/ci/images/Dockerfile.web" "$context/"

build() {
  local tag="$1" file="$2"
  echo "==> docker build $tag"
  DOCKER_CONFIG="${DOCKER_CONFIG:-$root/.build/docker-config}" \
    docker build -f "$context/$file" -t "$registry/$tag" "$context"
}
build "marqdo-ci-shell:$version" Dockerfile.shell
docker tag "$registry/marqdo-ci-shell:$version" "marqdo-ci-shell:$version"
build "marqdo-ci-rust:$version" Dockerfile.rust
build "marqdo-ci-web:$version" Dockerfile.web

if [[ "$push" == 1 ]]; then
  : "${MARQDO_GIT_TOKEN:?set MARQDO_GIT_TOKEN to publish images}"
  export DOCKER_CONFIG="${DOCKER_CONFIG:-$root/.build/docker-config}"
  echo "$MARQDO_GIT_TOKEN" | docker login "${registry%%/*}" -u "${MARQDO_GIT_USERNAME:-cflmy}" --password-stdin
  docker push "$registry/marqdo-ci-shell:$version"
  docker push "$registry/marqdo-ci-rust:$version"
  docker push "$registry/marqdo-ci-web:$version"
  echo "published immutable CI images for $version"
fi
