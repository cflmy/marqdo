# P4 性能验证设计（实验协议与判据）

| | |
|---|---|
| 状态 | **Accepted · P4 执行依据** |
| 日期 | 2026-09-23 |
| 授权 | 用户授权**自行设计**本性能验证设计（P4 不再等待外部文档导入；`doc/next/002.md` 与性能验证无关） |
| 上位判据 | [three-problems.md](three-problems.md) §5.1 判据矩阵 + §5.2 实验 (a)–(e)；「判据矩阵全绿 + 实验数字可复现之前，三大问题一律视为未解决」 |
| 工程任务 | [three-problems-plan.md](three-problems-plan.md) Phase 1–3（已完成）；本文只负责**验证它们** |

> **本文定位**：把 three-problems.md §5.2 的五个实验操作化——每个实验给出**材料、协议、指标、判据**，全部机器可执行、数字可复现。实验产物 = 报告 [perf-validation-report-2026-09-23.md](perf-validation-report-2026-09-23.md)。**只认行为与数字，不认宣言。**

## 0. 总览

| 实验 | 验证对象 | 判据（全过 = 该问题的实证成立） | 执行方式 | 状态 |
|------|----------|-------------------------------|----------|------|
| (a) 锚点修复消融 | 问题三 · 可靠反馈 | 一轮修复率 ≥ 80%（anchor 臂），护栏 0 越界 | 模型在环 `tests/repair_experiment.rs` | ✅ 已完成（三轮双模型） |
| (b) 盲测 | 问题三 · 不必学语法 | 查询组错误率 ≤ 全文档组；token 显著更少；修复轮次不升 | 模型在环 `tests/blind_experiment.rs` | 见报告 |
| (c) 契约消融 | 问题二 · 语义校验 | 拦截率 = 100%；误报率 = 0%；无契约路径零回归 | 确定性 `tests/contract_ablation.rs` | 见报告 |
| (d) 防漂移演练 | 问题二 · 契约可信 | 故意写错契约必报率 = 100% | 确定性 `tests/drift_drill.rs` | 见报告 |
| (e) 核心守卫演练 | 问题一 · 核心要小 | 三种变异注入各必红 → 还原必绿 | 变异脚本（见 §4） | 见报告 |

判定口径（[three-problems.md](three-problems.md) §5.1）：**任何一项判据不成立，对应问题即「未解决」**——不许用文档宣言充数。

---

## 1. 实验 (a)：锚点修复消融（问题三 · 可靠反馈）

**假设**：诊断带 `doc_anchor`/`doc_quote`/`suggestion`（锚点诊断）比裸诊断显著提高一轮修复率。

- **对照**：`anchor` 臂（全量诊断字段）vs `plain` 臂（剥掉 doc_anchor/doc_quote/suggestion）。
- **材料**：golden 20 例（10 骨架 × 2 变体：arg/ret/field/key/typo/drift/dup），易锚定错误。
- **协议**：`validate` → `repair_targets` → 模型行编辑 → `repair_apply`（越界必拒/abstain）→ 复验；≤2 轮。
- **指标**：一轮修复率、最终修复率、平均尝试、越界拒、弃权。
- **判据**：anchor 臂一轮修复率 ≥ 80%；护栏 0 次越界应用（越界必拒由 `tests/repair_loop.rs` 硬断言兜底）。
- **执行**：`OPENAI_MODEL=… cargo test --test repair_experiment -- --ignored --nocapture`。
- **结果**：三轮双模型全过（cflmy anchor 100% / mimo 90%→100%，plain 85%–100%，护栏 0 越界）——
  详见 [repair-experiment-2026-09-22.md](repair-experiment-2026-09-22.md) 与
  [repair-experiment-2026-09-22-mimo-v2.6-flash.md](repair-experiment-2026-09-22-mimo-v2.6-flash.md)。**✅ PASS**。
- **诚实注记**：样本为易锚定错误（两臂近天花板），锚点增量的区分性证据留待难样本扩展；不影响本判据达标。

## 2. 实验 (b)：盲测——查询式 vs 全文档背诵（问题三 · 超级核心）

**假设**：「零语法文档 + MLSP 查询」组（Q）成绩**不劣于**「全文档背诵」组（D），且 prompt token 显著更少、修复轮次不升（[three-problems.md](three-problems.md) §3.7 最终判据）。

### 2.1 对照（唯一变量 = 语法知识的获取方式）

| 臂 | 系统提示词内容 | 语法知识来源 |
|----|----------------|--------------|
| **Q 查询组** | 硬规则（12 条）+ 契约表三形状 + MLSP 查询通道说明 | 按需 `QUERY: <关键字>`（≤3 次，harness 代答 `mlsp syntax` 卡片） |
| **D 全文档组** | 硬规则（12 条）+ 契约表三形状 + **完整 markup 参考**（`tests/blind/docs-bundle.md`：v0.3 全语法表 + 中文内建面） | 上下文预载（背诵） |

两臂共同点：硬规则与契约表形状**同文**（受控变量，不测契约知识差异）；修复回路相同（`mlsp validate` 诊断回喂，≤2 轮）；模型 / 温度 / 任务 / 提示词模板相同。

### 2.2 材料：任务集（6 例，业务 3 + 算法 3）

| id | 类型 | 任务 | 功能 oracle（stdout 精确匹配） |
|----|------|------|-------------------------------|
| `claims_reserve_settle` | 业务 | 保险准备金（免赔/共保/分项限额/追偿/地板） | `28000` |
| `contract_milestone_pay` | 业务 | 工程里程碑应付款（保留金/逾期罚/变更/税/地板） | `37354` |
| `vendor_po_gate` | 业务 | 采购 PO 准付闸（折扣/加急/预算帽/合规费/地板） | `814` |
| `qb_gcd` | 算法 | 最大公约数；打印 gcd(48,18) gcd(1071,462) gcd(0,5) gcd(17,5) 各一行 | `6` `21` `5` `1` |
| `qb_max_sublist_sum` | 算法 | 最大子段和；三组输入各打印一行 | `6` `6` `-1` |
| `qb_get_factors` | 算法 | 质因数分解，因数以 `*` 连接；12 / 13 / 60 各一行 | `2*2*3` `13` `2*2*3*5` |

业务任务书复用 MarqdoThesis `formal/exp3-bizreq/data/tasks/`（权威规则文本原样进入提示词）；算法任务书为 QuixBugs 改写（从「修 bug」改「实现 + 固定输出」，消除参考源码变量）。

### 2.3 协议（每例）

1. 提示词 = 任务书 + 「输出完整 `.mq.md` 源码于单个 ```markdown 围栏」；
2. **查询阶段**（仅 Q 臂）：模型可输出 `QUERY: <关键字>` 行，harness 以 `mlsp syntax` 卡片作答并继续；累计 ≤3 次；
3. **执行阶段**：`marqdo run` → stdout == oracle 即通过；失败则回喂 `mlsp validate` 结构化诊断 + stderr，允许 ≤2 次修复；
4. 记录每轮 token（API `usage.prompt_tokens + completion_tokens` 累计）。

### 2.4 指标与判据

| 指标 | 口径 | 判据 |
|------|------|------|
| 功能通过率（一轮 / 最终≤3 轮） | stdout 与 oracle 精确匹配 | Q 最终通过率**不劣于** D（允许 −0 例） |
| 首轮语法错误率 | 首轮 `validate`/run 出现 parse 类失败的例数占比 | Q ≤ D |
| 首轮契约诊断率 | 首轮出现 `contract.*` 诊断的例数占比 | Q ≤ D（观测项） |
| 平均修复轮次 | 通过前的修复次数（含首轮生成记 0） | Q ≤ D |
| token 用量 | 逐例累计 prompt+completion | Q **显著更少**（≤ 70% of D） |

**判定**：四条判据全成立 ⇒ 「AI 不必学语法」实证成立；否则按 §3.7「继续改协议」，不得以改进文档充数。
- **执行**：`OPENAI_MODEL=… cargo test --test blind_experiment -- --ignored --nocapture`（每模型一轮）。

## 3. 实验 (c)：契约消融（问题二 · 语义校验）

**假设**：契约只在**有契约处**拦截错误值（P1：无契约 = 全动态），且不误报。

- **材料**：`tests/contract_ablation.rs` 内联四边界用例（调用实参 / 返回 / 表绑定 / 字段访问）×三形态：
  ① 带契约 + 错值 → **必拦**（`contract.*` 诊断）；② 带契约 + 对值 → **零误报**（正常产出）；③ 同程序**删契约表** + 错值 → **零契约诊断**（动态语义原样）。
- **指标与判据**：
  - 拦截率 = ①被拦 / ①总数 = **100%**；
  - 误报率 = ②误报 / ②总数 = **0%**；
  - 无契约零回归 = ③零 `contract.*` 诊断且输出 = 动态语义基线；且全量 `cargo test` 绿（441+ 既有测试是无契约路径的主回归面）。
- **执行**：`cargo test --test contract_ablation`。

## 4. 实验 (d)：防漂移演练（问题二 · 契约可信）

**假设**：契约写错**必**被机器抓（不静默信任契约）——three-problems.md §2.8「故意写错契约的金样必红」。

- **材料**：`tests/drift_drill.rs`——故意写错契约的 N 例（类型 typo `nubmer`、未知对象类型、契约参数名 ≠ 升参名 ×2、返回契约明显不符、重复契约表、错位契约表）+ **负对照**（被绑定的 `字段|类型` 数据表 = web_db_migration 式，必须**不**被当契约吃掉）。
- **指标与判据**：
  - 必报率 = 漂移例被诊断（`check` 报错/警告或运行时 `contract.*`）/ 漂移例总数 = **100%**；
  - 负对照误报 = **0**。
- **执行**：`cargo test --test drift_drill`。

## 5. 实验 (e)：核心守卫演练（问题一 · 核心要小）

**假设**：核心表面任何**单侧**变更（代码/文档/金样不同步）→ CI 必红——three-problems.md §1.5「新增标记不更新清单时 CI 必须红」。

- **材料**：`tests/core_surface.rs` 守卫 + 三种变异注入（脚本化，`git checkout` 还原）：
  1. `src/parse/mod.rs` `CORE_CONSTRUCTS` 增一行、不改文档 → **必红**；
  2. `doc/design/core-surface.md` 表增一行、不改代码 → **必红**；
  3. 金样文件改名（路径失效）→ **必红**。
- **指标与判据**：3/3 变异注入后守卫测试红；还原后全绿。**必红率 = 100%**。
- **执行**：报告内嵌演练记录（命令 + 测试输出摘录）。

## 6. 报告与复现

- 报告：[perf-validation-report-2026-09-23.md](perf-validation-report-2026-09-23.md)（(a) 引用既有两份实验报告；(b)–(e) 本次产出）。
- 复现命令全部写入报告各节；模型在环实验记录模型名与端点，判据数字允许 ±1 例波动（模型非确定性），**判据线本身不放宽**。
- 汇总结论回填 [three-problems-plan.md](three-problems-plan.md) 落地记录；§5.1 判据矩阵逐行标注 ✅/❌。
