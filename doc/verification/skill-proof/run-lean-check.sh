#!/usr/bin/env bash
# run-lean-check.sh — 用真实 Lean 4.35.0 校验 `marqdo prove --emit` 的产物。
# 用法: run-lean-check.sh <file.mq.md>...
# 每个文件: prove 生成 → lake env lean 编译 → 按退出状态分类。
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"          # /workspace/marqdo
LEAN_BIN="$ROOT/target/toolchain/lean-4.35.0-linux/bin"
PROOF_DIR="$ROOT/target/proof/skill-proof"
MARQDO="$ROOT/target/node/debug/marqdo"
export PATH="$LEAN_BIN:$PATH"

[ -x "$MARQDO" ] || { echo "marqdo binary missing: $MARQDO" >&2; exit 2; }
[ -f "$PROOF_DIR/.lake/build/lib/lean/MarqdoProof.olean" ] || \
  { (cd "$PROOF_DIR" && lake build) >&2 || exit 2; }

declare -A RESULT
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
for src in "$@"; do
  name="$(basename "$src" .mq.md)"
  outdir="$work/$name"
  mkdir -p "$outdir"
  if ! "$MARQDO" prove "$src" --emit "$outdir" >/dev/null 2>&1; then
    RESULT[$src]="emit-failed"
    continue
  fi
  lean_file="$(find "$outdir" -name '*.lean' | head -1)"
  [ -n "$lean_file" ] || { RESULT[$src]="no-lean-emitted"; continue; }
  cp "$lean_file" "$PROOF_DIR/Generated/_check.lean"
  log="$work/$name.leanlog"
  (cd "$PROOF_DIR" && lake env lean Generated/_check.lean) >"$log" 2>&1
  rc=$?
  rm -f "$PROOF_DIR/Generated/_check.lean"
  if [ $rc -eq 0 ] && ! grep -q "error" "$log"; then
    RESULT[$src]="proved"
  elif grep -q "unsolved goals" "$log"; then
    RESULT[$src]="unproved"
  else
    RESULT[$src]="lean-error"
  fi
done

fail=0
for src in "$@"; do
  status="${RESULT[$src]}"
  printf '%-12s %s\n' "$status" "$src"
  [ "$status" = "proved" ] || fail=1
done
exit $fail
