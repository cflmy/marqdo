# 调研：`ext/agent` 在 A0–A4 之后还差什么

| | |
|---|---|
| 状态 | **调研笔记 · 缺口盘点（A0–A4 闭环之后）** |
| 日期 | 2026-08-28 |
| 前置调研 | [agent-frameworks-and-marqdo.md](agent-frameworks-and-marqdo.md)（优势与优化原则） |
| 已落地路线 | [ext-agent-optimize.md](../roadmap/ext-agent-optimize.md)（**A0–A4 done**） |
| 设计 | [ext-agent.md](../design/ext-agent.md) · [ext-agent-plan.md](../design/ext-agent-plan.md) · [ext-agent-parent.md](../design/ext-agent-parent.md) · [okf.md](../design/okf.md) |
| 邻近规划 | [agent-streaming.md](../roadmap/agent-streaming.md) · [okf-near-match.md](../roadmap/okf-near-match.md) |
| 外部对照（2026） | LangGraph persistence / CrewAI+Flows / MCP 工具协议 / 业界 Agent 评测与记忆层讨论 |

---

## 0. 一句话结论

调研驱动的 **A0–A4** 已把 Marqdo 智能体的**差异化内核**补齐：工作簿补丁可靠、OKF 复用飞轮可读可用、过程事件可审、上下文有预算、外置检索/MCP 只作证据。

相对 2026 年主流框架（LangGraph、CrewAI、厂商 SDK、MCP 生态），Marqdo **不再缺「能跑的 Agent loop」**；缺口集中在：

1. **产品化与可发现性**（示例、文档新鲜度、上手路径）  
2. **生产配套**（评测 harness、真 MCP、断点续跑、HITL、飞轮指标）  
3. **可选扩展面**（并行 fan-out、多 Agent/A2A、trace 外发）  

**不要**用缺口清单去证明应回退到隐藏 TOOL: 循环或把向量库升为权威真相——那与宪法冲突，也抵消已落地的优势。

---

## 1. 已落地基线（A0–A4）

| 波次 | 主题 | 验收要点 |
|------|------|----------|
| **A0** | 工作簿补丁 / 固化 | 整文件 FIND 护栏；`soft` 仅 miss；`workbook_solidify`；金样 `agent-workbook-patch-a0` |
| **A1** | OKF 复用飞轮 | `list_tasks` 策展字段；`plan` 暴露 `match`/`score`；金样 `agent-kb-plan-hit` |
| **A2** | 过程可见 | `events` 默认落盘；view plan 过程卡；SSE 仍仅 `stream=True` |
| **A3** | 上下文预算 | `source_brief` / `skill_brief`；`READ:source|skill`；`step max_reads` |
| **A4** | RAG/MCP 证据工具 | `corpus_search`；fixture `mcp_*`；`authority=workbook` |

原则回顾（见前置调研 §9）：强化差异化；资源砸在工作簿 / 写回 / 固化 / OKF / 过程可视；外置 RAG/MCP **只作工具**。

---

## 2. 相对业界：已经「够格」的部分

下列能力若取消「可执行 Markdown 为权威」即消失——这是护城河，不是功能点攀比：

| 能力 | Marqdo 形态 | 业界常见替代 |
|------|-------------|--------------|
| 单一真相源 | 源码即提示 + 调用点注入 | Prompt Hub / 字符串 system |
| 过程可审 | 写回块 + `events` + view 过程卡 | Langfuse / LangSmith 外挂 |
| 成功可复用 | OKF 任务技能再跑 | transcript 再聊 / 向量片段召回 |
| 复杂任务边界 | 工作簿文件 + `subtask` | 无限加长 chat / 临时工具输出 |
| 确定步骤降本 | solidify → `llm_free` 快路径 | 每轮仍问模型 |

自评四问（前置调研 §9）粗判：

| 问 | 现状 |
|----|------|
| 新人能否只读一份 `.mq.md` 理解 Agent？ | **部分** — 机制在，缺 canonical 示例教程 |
| 失败后能否在 git + view 复盘、不强制外部 trace？ | **基本可以** |
| 成功任务下周能否以同一技能复用？ | **可以**（金样已锁二次命中） |
| 确定步骤是否越来越多变成普通 `##`、模型调用下降？ | **路径有**（solidify / llm_free），缺量化看板证明 |

---

## 3. 缺口总表（按优先级）

| 优先级 | 缺口 | 为何还缺 | 建议验收线索 |
|--------|------|----------|--------------|
| **高** | 官方端到端示例 + 文档新鲜度 | 无独立 `examples/agent-*`；Skill/参考里仍有「layout / roadmap」陈旧表述；外人难「照抄即跑通 plan→固化→二次命中」 | `examples/agent-pong`（或同类）+ README；Skill 专节与 A0–A4 对齐 |
| **高** | Agent 评测 / 回归 harness | 有离线金样，缺成本·恢复·命中率·补丁质量的系统评测；业界强调 evaluation 先于上线 | `tests/` 或脚本：命中率、llm_free 占比、失败恢复、可选 live 预算 |
| **高** | 真 MCP（HTTP/SSE）与凭证 | A4 仅 JSON fixture；缺 streamable HTTP、鉴权、resources/prompts | 金样打 mock MCP server；返回仍带 `authority=workbook` |
| **中** | Plan 断点续跑 | 工作簿在盘上，缺显式 checkpoint / resume（对标 LangGraph checkpointer） | `plan resume=` 从第 N 轮 DECISION 继续；进程崩溃可恢复 |
| **中** | 人机协同（HITL）加深 | 仅有 `confirm=True` 建簿即停；缺审批卡、拒绝重开、view 一键继续 | view 审批控件 + 金样 |
| **中** | 飞轮可观测指标 | `hits` / `improve_every` / `llm_free` 有字段，缺汇总与 CLI/view 报表 | `agent_kb_stats` 或 view 面板：命中率、探索浪费 |
| **中** | 受控并行 fan-out | `subtask` 能并行，plan 主路径偏串行 | 父协议允许有界并行调研再合并 |
| **低** | 多 Agent / A2A | dual skeleton 有；缺角色班组、跨进程 A2A | 多数场景可用「多工作簿 + 父 CALL」覆盖；有明确跨组织需求再开 |
| **低** | Trace 外发（OTLP / Langfuse） | 本地复盘已够；外发是集成题 | 可选 adapter；默认不依赖 |
| **刻意不做** | Dense embedding 默认化 | [okf-near-match.md](../roadmap/okf-near-match.md) **E** 仅在 n-gram 证据不足时评估 | 不宣传为默认向量库 |
| **刻意不做** | 私有 JSON TOOL 主路径 / 整文件 LLM 重写 | 与宪法及 parent 设计冲突 | 继续禁止 |

---

## 4. 分项说明

### 4.1 产品化（高）

**问题**  
内核能力已超过「能演示」门槛，但**可感知产品体验**仍依赖读设计文档与翻 `tests/ext/`。对照 quantum / web 已有 `examples/`，agent 侧空白最明显。

**建议**  
1. 一份最小可运行示例：`step` → `plan` → promote → 二次 `plan` 打印 `cache=hit`。  
2. 同步 Skill / `doc/README` / 公开页，删掉「仅 layout」类过时句。  
3. 示例强调「权威在 `.mq.md`」，corpus/mcp 仅作证据旁路。

### 4.2 评测 harness（高）

**问题**  
金样锁行为正确性；不锁「飞轮是否在变好」「一次失败后能否恢复」「live 成本是否失控」。2026 多 Agent 讨论普遍把 **evaluation** 列为上线前缺失步骤。

**建议**  
- 离线：命中路径、补丁护栏、预算截断（已有）再加 **stats 断言**。  
- 半自动 / live：固定小 goal 集，记录 rounds、cache 分布、token 粗估（若可从供应商响应取得）。  
- 明确：评测消费的是工作簿与 kb，而不是外部 transcript 库。

### 4.3 真 MCP（高）与「证据≠权威」（不变）

**问题**  
Fixture 证明协议形状与权威字段；生产要接真实 MCP server（工具、资源、鉴权）。

**约束**  
- 工具结果进入 observation / CALL 回执，**不得**静默写成 OKF 权威技能。  
- 晋升仍走 promote / solidify / 人审（既有路径）。

### 4.4 断点续跑与 HITL（中）

**问题**  
LangGraph 类框架的 checkpointer / interrupt 是生产差异点。Marqdo 已有文件化状态（工作簿、写回、events），差的是**一等恢复 API**与**可视审批**。

**建议方向**  
- Resume：序列化「当前 round、path、last observation、events」到约定槽或旁路文件，重启后跳过已完成子跑。  
- HITL：扩展 `confirm` 语义（建簿 / 危险 PATCH / promote）；view 展示 pending 卡。

### 4.5 飞轮指标与并行（中）

**问题**  
第四问（模型调用是否下降）需要数字。并行能缩短探索，但须有界，避免工作簿爆炸（见 [ext-agent.md](../design/ext-agent.md) §9）。

### 4.6 多 Agent / A2A / 外发 trace（低）

**问题**  
业界热度高（Crews、A2A、共享记忆层）。Marqdo 的「父 + 多工作簿」已覆盖多数单组织编排；跨组织 A2A 与共享记忆库是另一产品层，可用 MCP/证据工具接入，而不必把记忆权威搬出 `.mq.md`。

---

## 5. 建议的下一波分期（草案 B）

> 非正式锁定；落地时再写入 [ext-agent-optimize.md](../roadmap/ext-agent-optimize.md) 或独立 `ext-agent-productize.md`。

| 波次 | 主题 | 验收线索 |
|------|------|----------|
| **B0** | 官方示例 + Skill/文档去陈 | `examples/agent-*` 可 `marqdo run`；Skill 与 A0–A4 一致 |
| **B1** | 评测 harness | 命中率 / llm_free / 恢复用例可一键跑 |
| **B2** | Live MCP 客户端 | mock server 金样；`authority=workbook` 不变 |
| **B3** | Plan resume + HITL | 崩溃可续；view 审批 |
| **B4** | kb stats 面板 | CLI 或 view 展示飞轮数字 |
| **B5** | 有界并行（可选） | fan-out 上限 + 合并观察 |

---

## 6. 明确「不是缺口」的清单

避免把差异化优势误判为缺陷：

1. **没有默认 dense 向量库** — 有意；OKF near 是稀疏词法；corpus 是关键词证据。  
2. **没有隐藏 TOOL: JSON 主协议** — 有意；CALL/READ/DECISION 与文档同构。  
3. **子任务默认 quiet** — 有意；答案走返回值，过程走 events/stream。  
4. **ext 不直调 host_*** — 有意；Agent 域在插件，防核心膨胀。  
5. **不捆绑领域工单技能** — 有意；官方框架保持可教、可审、可扩展。

---

## 7. 结论与推荐下一刀

A0–A4 之后，Marqdo `ext/agent` 在「文档驱动智能体」叙事上已经**自洽且可演示**。相对业界，短板是 **被人用起来的路径** 与 **生产级配套**，不是再堆一条编排 DSL。

**推荐下一刀顺序：**

1. **B0 官方示例 + 文档去陈**（最低成本、最高感知）  
2. **B1 评测 harness**（锁住飞轮与质量，防止回退）  
3. 按真实用户需求选 **B2 live MCP** 或 **B3 resume/HITL**

若四问在 B0–B1 后均可回答「是」，再考虑 B4/B5 与低优先级项。

---

## 8. 参考

### 本仓

- [agent-frameworks-and-marqdo.md](agent-frameworks-and-marqdo.md)  
- [ext-agent-optimize.md](../roadmap/ext-agent-optimize.md)  
- [okf-near-match.md](../roadmap/okf-near-match.md)（E dense 仍非默认）  
- [agent-streaming.md](../roadmap/agent-streaming.md)  

### 外部（抽样，2026）

- LangGraph Persistence（checkpointer / store）  
- CrewAI Crews + Flows；MCP 作为工具连接层而非编排本体  
- 业界对 Agent evaluation、durable memory / A2A 状态层的讨论（共享记忆 ≠ 应取代可执行文档权威）
