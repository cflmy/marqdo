# 修复实验（§5.2(a) 锚点消融）—— 2026-09-22

| | |
|---|---|
| 模型 | `mimo-v2.6-flash` @ `https://llm.cflmy.cn/v1` |
| 样本 | golden 20 例（易锚定错误） |
| 协议 | `validate` → `repair_targets` → 模型行编辑 → `repair_apply`（越界必拒/abstain）→ 复验；≤2 轮 |
| 对照 | `anchor` = 诊断带 `doc_anchor`/`doc_quote`/`suggestion`；`plain` = 全部剥掉 |

## 汇总（设计目标：一轮修复率 ≥ 80%）

| 臂 | 一轮修复率 | 最终修复率(≤2轮) | 平均尝试 | 越界拒（护栏） | 弃权 |
|---|---|---|---|---|---|
| anchor | 20/20（100%） | 20/20（100%） | 1.00 | 0 | 0 |
| plain | 18/20（90%） | 19/20（95%） | 1.05 | 0 | 1 |

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
| field-nullable-a | plain | 一轮修复 | 1 | 0 |  |
| field-nullable-b | anchor | 一轮修复 | 1 | 0 |  |
| field-nullable-b | plain | 一轮修复 | 1 | 0 |  |
| field-type-a | anchor | 一轮修复 | 1 | 0 |  |
| field-type-a | plain | 一轮修复 | 1 | 0 |  |
| field-type-b | anchor | 一轮修复 | 1 | 0 |  |
| field-type-b | plain | 一轮修复 | 1 | 0 |  |
| key-fix-a | anchor | 一轮修复 | 1 | 0 |  |
| key-fix-a | plain | 未修复 | 1 | 0 | 模型弃权/输出不可解析: [{"op":"replace","line":13","text":"*`用户`[^name]*"}] |
| key-fix-b | anchor | 一轮修复 | 1 | 0 |  |
| key-fix-b | plain | 二轮修复 | 2 | 0 | 修复后仍有诊断: {"code":"marqdo.error","contract_ref":null,"doc_anchor":null,"doc_quote":null,"message":"missing map key `phone`","severity":"error","span":{"col":1,"file":"/tmp/marqdo_repair_1666049_plain_key-fix-b.mq.md","line":14},"suggestion":null} |
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
