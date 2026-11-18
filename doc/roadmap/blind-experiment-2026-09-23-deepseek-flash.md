# 盲测实验（§5.2(b) 查询式 vs 全文档背诵）—— 2026-09-23

| | |
|---|---|
| 模型 | `deepseek-flash` @ `https://llm.cflmy.cn/v1` |
| 样本 | 6 任务（业务 3 + QuixBugs 改写 3），oracle = stdout 精确匹配 |
| 协议 | 系统提示词 = 硬规则 12 条 + 契约表三形状（两臂同文）；**唯一变量 = 语法来源**：`query` 臂 MLSP 查询通道（≤3 次）vs `docs` 臂完整 markup 参考（背诵） |
| 修复 | validate 诊断 + stderr 回喂，≤2 次修复；temperature=0，max_tokens=8000 |

## 汇总（判据：Q 通过率不劣于 D；首轮语法错误率 Q≤D；token Q≤70%%D；轮次不升）

| 臂 | 通过(一轮) | 通过(最终≤3) | 首轮语法错误 | 首轮契约诊断 | 平均修复轮(过例) | token 合计(prompt+completion) | 弃权/基础设施 |
|---|---|---|---|---|---|---|---|
| query | 0/3 | 0/3 | 2/3 | 1/3 | 0.00 | 69771（42478+27293） | 3 |
| docs | 3/6 | 4/6 | 2/6 | 0/6 | 0.50 | 57276（33346+23930） | 0 |

## 逐例

| 例 | 臂 | 结果 | 修复轮 | 首轮语法错 | 首轮契约诊断 | token(p+c) | 查询 | 备注 |
|---|---|---|---|---|---|---|---|---|
| claims_reserve_settle | query | load-fail | 0 | 是 | 否 | 11987+8177 | 3 | {"error":{"code":"mlsp.parse_error","message":"<memory>: trailing input in expression: \"(claim - deductible, 0)\""},"id":1,"ok":false} \| stderr=error: /tmp/marqdo_blind_1694593_query_claims_reserve_settle.mq.md: trailing input in expression: "(claim - deductible, 0)"  |
| contract_milestone_pay | query | infra | 0 | 否 | 否 | 3970+4718 | 3 | 非 JSON / 空体:   |
| vendor_po_gate | query | load-fail | 0 | 是 | 否 | 9552+3686 | 3 | 回复中无程序 \| 开头: QUERY: 条件分支 写法 |
| qb_gcd | query | run-fail | 0 | 否 | 是 | 9700+8488 | 3 |  \| stderr=error: /tmp/marqdo_blind_1694593_query_qb_gcd.mq.md:26:1: unknown function `gcd(48`  |
| qb_max_sublist_sum | query | infra | 0 | 否 | 否 | 1253+134 | 3 | 非 JSON / 空体:   |
| qb_get_factors | query | infra | 0 | 否 | 否 | 6016+2090 | 3 | 非 JSON / 空体:   |
| claims_reserve_settle | docs | pass | 0 | 否 | 否 | 2699+2987 | 0 | queries=0 |
| contract_milestone_pay | docs | pass | 0 | 否 | 否 | 2800+3202 | 0 | queries=0 |
| vendor_po_gate | docs | pass | 0 | 否 | 否 | 2690+1264 | 0 | queries=0 |
| qb_gcd | docs | pass | 2 | 否 | 否 | 8147+6970 | 0 | queries=0 |
| qb_max_sublist_sum | docs | run-fail | 0 | 是 | 否 | 8518+3609 | 0 |  \| stderr=error: /tmp/marqdo_blind_1694593_docs_qb_max_sublist_sum.mq.md:3:1: unknown function `[-2,`  |
| qb_get_factors | docs | run-fail | 0 | 是 | 否 | 8492+5898 | 0 |  \| stderr=error: /tmp/marqdo_blind_1694593_docs_qb_get_factors.mq.md:20:1: missing argument for parameter `value`  |

报告由 `tests/blind_experiment.rs` 生成；材料冻结于 `tests/blind/`（任务书 + docs-bundle），判据见 `doc/roadmap/perf-validation.md` §2。
