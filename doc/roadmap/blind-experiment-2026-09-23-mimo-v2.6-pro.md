# 盲测实验（§5.2(b) 查询式 vs 全文档背诵）—— 2026-09-23

| | |
|---|---|
| 模型 | `mimo-v2.6-pro` @ `https://llm.cflmy.cn/v1` |
| 样本 | 6 任务（业务 3 + QuixBugs 改写 3），oracle = stdout 精确匹配 |
| 协议 | 系统提示词 = 硬规则 12 条 + 契约表三形状（两臂同文）；**唯一变量 = 语法来源**：`query` 臂 MLSP 查询通道（≤3 次）vs `docs` 臂完整 markup 参考（背诵） |
| 修复 | validate 诊断 + stderr 回喂，≤2 次修复；temperature=0，max_tokens=8000 |

## 汇总（判据：Q 通过率不劣于 D；首轮语法错误率 Q≤D；token Q≤70%%D；轮次不升）

| 臂 | 通过(一轮) | 通过(最终≤3) | 首轮语法错误 | 首轮契约诊断 | 平均修复轮(过例) | token 合计(prompt+completion) | 弃权/基础设施 |
|---|---|---|---|---|---|---|---|
| query | 0/0 | 0/0 | 0/0 | 0/0 | 0.00 | 8011（6566+1445） | 6 |
| docs | 0/0 | 0/0 | 0/0 | 0/0 | 0.00 | 0（0+0） | 6 |

## 逐例

| 例 | 臂 | 结果 | 修复轮 | 首轮语法错 | 首轮契约诊断 | token(p+c) | 查询 | 备注 |
|---|---|---|---|---|---|---|---|---|
| claims_reserve_settle | query | infra | 0 | 否 | 否 | 1424+366 | 3 | 非 JSON / 空体:   |
| contract_milestone_pay | query | infra | 0 | 否 | 否 | 1528+381 | 1 | 非 JSON / 空体:   |
| vendor_po_gate | query | infra | 0 | 否 | 否 | 0+0 | 0 | 非 JSON / 空体:   |
| qb_gcd | query | infra | 0 | 否 | 否 | 1211+252 | 3 | 非 JSON / 空体:   |
| qb_max_sublist_sum | query | infra | 0 | 否 | 否 | 1216+226 | 3 | 非 JSON / 空体:   |
| qb_get_factors | query | infra | 0 | 否 | 否 | 1187+220 | 3 | 非 JSON / 空体:   |
| claims_reserve_settle | docs | infra | 0 | 否 | 否 | 0+0 | 0 | 非 JSON / 空体:   |
| contract_milestone_pay | docs | infra | 0 | 否 | 否 | 0+0 | 0 | 非 JSON / 空体:   |
| vendor_po_gate | docs | infra | 0 | 否 | 否 | 0+0 | 0 | 非 JSON / 空体:   |
| qb_gcd | docs | infra | 0 | 否 | 否 | 0+0 | 0 | 非 JSON / 空体:   |
| qb_max_sublist_sum | docs | infra | 0 | 否 | 否 | 0+0 | 0 | 非 JSON / 空体:   |
| qb_get_factors | docs | infra | 0 | 否 | 否 | 0+0 | 0 | 非 JSON / 空体:   |

报告由 `tests/blind_experiment.rs` 生成；材料冻结于 `tests/blind/`（任务书 + docs-bundle），判据见 `doc/roadmap/perf-validation.md` §2。
