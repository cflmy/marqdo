# P4 验证汇总报告 — 2026-09-23

> 执行 [perf-validation.md](perf-validation.md)（自设计性能验证协议）的结果汇总。
> 覆盖 three-problems.md 三大问题的**验证矩阵**（§5.2 a–e）。逐例明细见各实验原始报告（文末索引）。

| | |
|---|---|
| 协议 | [perf-validation.md](perf-validation.md)（实验 (a)–(e)） |
| 判据源 | [three-problems.md](three-problems.md) §3.7 / 各问题 §2.4 |
| 构建 | marqdo v1.0.7（本仓库工作树；金 20 + 全量回归通过后采样） |
| 端点 | `llm.cflmy.cn/v1`（OpenAI 兼容；temperature=0） |
| 模型 | 修复实验：`cflmy`、`mimo-v2.6-flash`；盲测：`deepseek-flash`、`mimo-v2.6-pro`（按用户指定） |
| 回归 | 全量干净回归（无 API key 基线）**零失败** |

## 1. 五实验结果与判据对照

| 实验 | 判据 | 结果 | 判定 |
|---|---|---|---|
| **(a) 锚点消融**（T3.3，双模型×20例×双臂，三轮） | anchor 臂一轮修复率 ≥ 80%；护栏 0 次越界应用 | cflmy anchor **20/20=100%**（plain 17/20=85%）；mimo-v2.6-flash anchor **20/20=100%**（plain 18/20=90%；容错加固中的第二轮 anchor 18/20=90% 亦 ≥80%）；越界拒/越界应用 0/0 | ✅ 过 |
| **(b) 盲测**（查询式 vs 全文档背诵，6任务×2臂） | 四判据：最终通过率 Q 不劣于 D（允许 −0 例）；首轮语法错误率 Q ≤ D；平均修复轮次 Q ≤ D；token ≤ 70% of D（首轮契约诊断率为观测项） | deepseek-flash：docs 4/6 vs query 0/3（3 例端点 infra 剔除）；首轮语法错 2/3 vs 2/6；token 69,771 vs 57,276。mimo-v2.6-pro：首跑 12/12 端点 infra，单独重跑见 §3 | ❌ 四判据未全立 ⇒ 按 §3.7 继续改协议 |
| **(c) 契约消融**（四边界×三情形） | 带契约错值拦截 = 100%；带契约对值误报 = 0%；无契约路径零回归 | 拦截 4/4=**100%**；误报 0/4=**0%**；无契约 4/4 动态语义保持（含 `missing map key` 基线） | ✅ 过 |
| **(d) 防漂移演练**（9 漂移 + 2 负对照） | 故意写错必报 = 100%；对照组误报 = 0% | 必报 9/9=**100%**（`unknown_type`×2、`param_drift`×2、`return_mismatch`、`arg_mismatch`、`unknown_key`、`contract.duplicate`、`contract.unknown_table`）；负对照误报 0/2 | ✅ 过 |
| **(e) 核心守卫演练**（3 种变异注入） | 单边改动必红；还原必绿 | 3/3 必红（`CORE_CONSTRUCTS` 私增、`core-surface.md` 私增、金样删除）；还原全绿 | ✅ 过 |

**可复制结论（a/c/d/e 全过）**：锚点臂一轮修复率双模型 100%（判据 ≥80%，增量 +15pp/+10pp——易锚定样本近天花板，区分性留难样本扩展）；契约系统"只在声明处拦截、零误报、不碰动态路径"成立；契约/实现互查防漂移 100% 必报；核心面单边膨胀被 CI 守卫机械阻断。

## 2. 实验 (c)(d)(e) 明细

### (c) 契约消融（tests/contract_ablation.rs，确定性）

四边界（调用实参 / 返回值 / 表绑定 / 键访问）各测三情形：

| 情形 | 口径 | 结果 |
|---|---|---|
| ① 带契约 + 错值 | 必须拦截（`contract.arg_mismatch` / `contract.return_mismatch` / `contract.field_mismatch` / `contract.unknown_key`） | 4/4 |
| ② 带契约 + 对值 | 不得报契约错（同路径同形状） | 0/4 误报 |
| ③ 无契约 + 错值 | 保持动态语义（不得出现 `contract.*`；走经典动态错误如 `missing map key`） | 4/4 |

取元一律用主推的 `[]()` 形（`[键](`集合`)`）验证；`集合[^键]` 过渡形保留兼容金样覆盖。

### (d) 防漂移演练（tests/drift_drill.rs，确定性）

9 个"故意写错"全部必报：类型词拼错、参数名漂移、升参未覆盖、返回越约、实参越约、未知键、重复契约表、错列表头契约表、运行时值越约。负对照 2 例（被绑定的数据表、纯动态程序）零误报。同时验证"契约表一经出现即为完整声明"（新升参未覆盖 ⇒ `contract.param_drift` warning）被按设计触发。

### (e) 核心守卫演练（人工变异，tests/core_surface.rs 必红验证）

| 变异 | 注入点 | 结果 |
|---|---|---|
| 私增 `ghost-construct` | `src/parse/mod.rs` `CORE_CONSTRUCTS` | 🔴 必红 |
| 私增 `ghost-construct` 行 | `doc/design/core-surface.md` | 🔴 必红 |
| 删除金样文件 | `tests/markup-v03/soft-emphasis.mq.md` | 🔴 必红 |

三次还原后 🟢 全绿。CI 守卫强制"清单与文档双改 + 全部金样存在"。

## 3. 实验 (b) 盲测：详况与诚实结论

### 协议

两臂系统提示词同文 = 硬规则 12 条 + 契约表三形状；**唯一变量 = 语法知识获取**：

- `query` 臂：MLSP 查询通道（`QUERY: <关键字>` ⇒ `mlsp syntax` 卡片，含戒律原文/样例/文档锚点；≤3 次）
- `docs` 臂：完整 markup 参考背诵（`tests/blind/docs-bundle.md`，背诵式全语法表 + 中文内建面）

任务 6 例（`tests/blind/tasks/`）：业务 3（保险理赔准备金 / 工程进度款 / 采购付款门槛，均出自律所实测任务集）+ QuixBugs 改写 3（gcd / 最大子段和 / 质因数分解）；oracle = stdout 精确匹配（任务书自带期望输出，把推理难度控制掉，失败归因于**语言能力**）。修复回路 ≤2 次（validate 结构化诊断 + stderr 回喂）。

### deepseek-flash（完成）

| 臂 | 通过(一轮) | 通过(最终) | 首轮语法错 | 平均修复轮(过例) | token(p+c) | infra 剔除 |
|---|---|---|---|---|---|---|
| query | 0/3 | 0/3 | 2/3 | – | 69,771（42,478+27,293） | 3/6 |
| docs | 3/6 | **4/6** | 2/6 | 0.50 | 57,276（33,346+23,930） | 0/6 |

- **docs 臂**：业务题 3/3 全部一轮通过（各 ~3k token）；`qb_gcd` 修复 2 轮过；`qb_max_sublist_sum`、`qb_get_factors` 暴露真语法滑倒（列表字面量被当调用 `unknown function [-2,`；参数名漂移 `missing argument for parameter value`）。
- **query 臂**：查询通道**被积极使用**（5/6 例用满 3 次，多词查询如 `条件分支 写法` 也能正确命中卡片——已单独验证查询匹配无缺陷）；但 3 例被端点空体吃掉，其余 3 例败于真语法错（Python 式元组 `(a, b)` ⇒ `trailing input in expression`；Python 式调用 `gcd(48, 18)` ⇒ `unknown function gcd(48`）与 1 例查询后未交程序。

### mimo-v2.6-pro（重跑中）

首跑 12/12 全部"非 JSON / 空体"（与 deepseek 并行时段；同参数小冒烟正常）。三向探针（小提示词×12000/4000、大提示词×12000）证实**网关随机空体**、与 `max_tokens`/提示词体量无稳定关系。已将重试 8→12 次、退避 10→15 秒，并**单独**重跑（不与其它模型抢并发）。结果落地后补入本节。

### 判据对照（截至 deepseek-flash）

| 判据（协议 §2.4 原文口径） | 目标 | 实测（deepseek-flash，非 infra 口径） | 判定 |
|---|---|---|---|
| 最终通过率（≤3 轮）Q 不劣于 D（允许 −0 例） | ≥ D | 0/3 vs 4/6 | ❌ |
| 首轮语法错误率 Q ≤ D | ≤ | 2/3 vs 2/6 | ❌ |
| 平均修复轮次 Q ≤ D | ≤ | query 臂无通过例，不可比 | ⚠️ |
| token 用量 Q ≤ 70% of D | ≤ 40,093 | 69,771（D 的 121.8%） | ❌ |
| （观测项）首轮契约诊断率 Q ≤ D | – | 1/3 vs 0/6 | ➖ 观测 |

**诚实结论：在 deepseek-flash 上，"查询式"没有跑赢"全背诵"**——三项可判量化判据未达，按协议 §3.7 判定：**四判据未全成立 ⇒「AI 不必学语法」本轮未获实证，继续改协议**（不是证伪——infra 吞掉 query 臂一半样本，非 infra n=3）。要点：

1. **统计力被端点削弱**：query 臂 6 例中 3 例被空体吃掉（协议无从执行），非 infra 样本 n=3；mimo-v2.6-pro 整组不可用。判据是"有方向的强声明"，当前证据**不足以支持也未证伪**其普适性，只能定格为"deepseek-flash × 本任务集 × 本协议版本"下的一次未达标观测。
2. **失败模式是真发现**：查询臂的败例是**跨语言习惯泄漏**（元组表达式、Python 调用形）——查询卡片能回答"条件分支怎么写"，但模型不查询"这里能不能用元组"。说明"AI 不必学语法"的瓶颈不在**查询机制**，而在**模型的自查纪律**（不知道自己不知道）。
3. **token 判据方向存疑**：查询式省的是**系统提示词常驻**（背诵包），花的是**多轮往返**。6 例短程序任务里往返开销 > 常驻节省；该判据更适用于**长会话多文件**场景（一次背诵包 × N 轮 vs 每轮均摊查询），本轮任务形态低估了查询式的收益面。
4. **docs 臂的 4/6 说明任务集有区分度**（不是全过/全不过的退化测量）。

## 4. 测量端基础设施记录（方法学附注）

- **LLM 端点不稳定**是本轮最大的干扰源：随机空体、reasoning 模型 `finish=length` 截断（思考吃满额度、正文为空）。对策：空体重试 5→12 次 + 退避 5→15 秒、截断自适应（`max_tokens` 加倍重试，封顶 32k）、infra 例从判据比率中剔除并单列。
- **harness 缺陷两处**（已修，未污染最终数据）：查询额度用满后的续篇被当最终回复（⇒ 假"无程序"）；程序提取失败不进修复回路（⇒ 少给模型一次标准修复机会）。修复后的协议对两臂完全一致。
- **模型口径变化留痕**：原定 `cflmy`/`mimo-v2.6-flash` 组合按用户指示弃用，改 `mimo-v2.6-pro`/`deepseek-flash`；修复实验 (a) 的双模型数据仍是原组合（金 20 协议已完成，不重跑）。

## 5. 材料与复现

```bash
# (a) 锚点消融（需要 API key）
cargo test --test repair_experiment -- --ignored --nocapture      # anchor/plain 双臂×20例
# (b) 盲测（每模型一次；建议单独跑避免并发）
OPENAI_MODEL=deepseek-flash MARQDO_EXP_DATE=2026-09-23 cargo test --test blind_experiment -- --ignored --nocapture
# (c)(d) 确定性
cargo test --test contract_ablation --test drift_drill
# (e) 核心守卫（变异注入→必红→还原必绿，按 perf-validation.md §5 手法）
cargo test --test core_surface
```

- 任务书 / 背诵包（冻结，勿改）：`tests/blind/`
- 原始报告：[blind-experiment-2026-09-23-deepseek-flash.md](blind-experiment-2026-09-23-deepseek-flash.md) · [blind-experiment-2026-09-23-mimo-v2.6-pro.md](blind-experiment-2026-09-23-mimo-v2.6-pro.md)（首跑全 infra）· [repair-experiment-2026-09-22.md](repair-experiment-2026-09-22.md) · [repair-experiment-2026-09-22-mimo-v2.6-flash.md](repair-experiment-2026-09-22-mimo-v2.6-flash.md)
- 判据协议：[perf-validation.md](perf-validation.md)（版本化于本仓库，实验前冻结）
