# 修复实验（§5.2(a) 锚点消融）—— 2026-09-22

| | |
|---|---|
| 模型 | `cflmy` @ `https://llm.cflmy.cn/v1` |
| 样本 | golden 20 例（易锚定错误） |
| 协议 | `validate` → `repair_targets` → 模型行编辑 → `repair_apply`（越界必拒/abstain）→ 复验；≤2 轮 |
| 对照 | `anchor` = 诊断带 `doc_anchor`/`doc_quote`/`suggestion`；`plain` = 全部剥掉 |

## 汇总（设计目标：一轮修复率 ≥ 80%）

| 臂 | 一轮修复率 | 最终修复率(≤2轮) | 平均尝试 | 越界拒（护栏） | 弃权 |
|---|---|---|---|---|---|
| anchor | 20/20（100%） | 20/20（100%） | 1.00 | 0 | 0 |
| plain | 17/20（85%） | 17/20（85%） | 0.85 | 0 | 0 |

## 逐例

| 例 | 臂 | 结果 | 轮次 | 越界拒 | 备注 |
|---|---|---|---|---|---|
| arg-num-a | anchor | 一轮修复 | 1 | 0 |  |
| arg-num-a | plain | 一轮修复 | 1 | 0 |  |
| arg-num-b | anchor | 一轮修复 | 1 | 0 |  |
| arg-num-b | plain | 一轮修复 | 1 | 0 |  |
| arg-text-a | anchor | 一轮修复 | 1 | 0 |  |
| arg-text-a | plain | 一轮修复 | 1 | 0 |  |
| arg-text-b | anchor | 一轮修复 | 1 | 0 |  |
| arg-text-b | plain | 一轮修复 | 1 | 0 |  |
| drift-cover-a | anchor | 一轮修复 | 1 | 0 |  |
| drift-cover-a | plain | 一轮修复 | 1 | 0 |  |
| drift-cover-b | anchor | 一轮修复 | 1 | 0 |  |
| drift-cover-b | plain | 一轮修复 | 1 | 0 |  |
| drift-rename-a | anchor | 一轮修复 | 1 | 0 |  |
| drift-rename-a | plain | 一轮修复 | 1 | 0 |  |
| drift-rename-b | anchor | 一轮修复 | 1 | 0 |  |
| drift-rename-b | plain | 一轮修复 | 1 | 0 |  |
| dup-delete-a | anchor | 一轮修复 | 1 | 0 |  |
| dup-delete-a | plain | 一轮修复 | 1 | 0 |  |
| dup-delete-b | anchor | 一轮修复 | 1 | 0 |  |
| dup-delete-b | plain | 一轮修复 | 1 | 0 |  |
| field-nullable-a | anchor | 一轮修复 | 1 | 0 |  |
| field-nullable-a | plain | 未修复 | 0 | 0 | LLM 调用失败（重试后）: 非 JSON / 空体:   |
| field-nullable-b | anchor | 一轮修复 | 1 | 0 |  |
| field-nullable-b | plain | 未修复 | 0 | 0 | LLM 调用失败（重试后）: 非 JSON / 空体:   |
| field-type-a | anchor | 一轮修复 | 1 | 0 |  |
| field-type-a | plain | 一轮修复 | 1 | 0 |  |
| field-type-b | anchor | 一轮修复 | 1 | 0 |  |
| field-type-b | plain | 一轮修复 | 1 | 0 |  |
| key-fix-a | anchor | 一轮修复 | 1 | 0 |  |
| key-fix-a | plain | 一轮修复 | 1 | 0 |  |
| key-fix-b | anchor | 一轮修复 | 1 | 0 |  |
| key-fix-b | plain | 未修复 | 0 | 0 | LLM 调用失败（重试后）: 非 JSON / 空体:   |
| ret-type-a | anchor | 一轮修复 | 1 | 0 |  |
| ret-type-a | plain | 一轮修复 | 1 | 0 |  |
| ret-type-b | anchor | 一轮修复 | 1 | 0 |  |
| ret-type-b | plain | 一轮修复 | 1 | 0 |  |
| type-typo-a | anchor | 一轮修复 | 1 | 0 |  |
| type-typo-a | plain | 一轮修复 | 1 | 0 |  |
| type-typo-b | anchor | 一轮修复 | 1 | 0 |  |
| type-typo-b | plain | 一轮修复 | 1 | 0 |  |

## 护栏核对

- 每次越界编辑都以 `mlsp.repair_out_of_scope` + `abstain` 被拒，且 `repair_apply` 全有或全无（源零字节不改）——由 `tests/repair_loop.rs` 的确定性断言覆盖；本实验计数仅作记录。
