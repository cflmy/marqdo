#!/bin/sh
# 校验内嵌语义底座（src/prove/driver.rs include_str!）与
# doc/verification/skill-proof/MarqdoProof.lean 一致，并输出其 sha256。
# 用法：scripts/check-proof-library.sh [LEAN_BIN]
#   LEAN_BIN 缺省用 PATH/MARQDO_PROOF_LEAN 里的 lean。
set -eu

cd "$(dirname "$0")/.."
LIB=doc/verification/skill-proof/MarqdoProof.lean
LEAN_BIN="${1:-${MARQDO_PROOF_LEAN:-lean}}"

# 1) 内嵌源必须与仓库里的底座逐字节一致（发布二进制自带它）。
#    driver.rs 的 include_str! 在编译期锁定该路径；这里再防人手改跑偏。
grep -q 'include_str!("../../doc/verification/skill-proof/MarqdoProof.lean")' src/prove/driver.rs

# 2) 底座 sha256（写进判定证书，判定可复核）。
if command -v sha256sum >/dev/null 2>&1; then
  SHA=$(sha256sum "$LIB" | cut -d' ' -f1)
else
  SHA=$(shasum -a 256 "$LIB" | cut -d' ' -f1)
fi
echo "MarqdoProof.lean sha256: $SHA"

# 3) （可选）真机验底座本身可编译。
if command -v "$LEAN_BIN" >/dev/null 2>&1; then
  TMP=$(mktemp -d)
  cp "$LIB" "$TMP/MarqdoProof.lean"
  (cd "$TMP" && "$LEAN_BIN" -o MarqdoProof.olean MarqdoProof.lean)
  rm -rf "$TMP"
  echo "MarqdoProof.lean compiles with $LEAN_BIN"
else
  echo "（未找到 $LEAN_BIN，跳过底座编译验证）"
fi
