# 调研：代码即文档 × OKF × 沙盒 下的智能体 / 大模型优化（2026-10）

| | |
|---|---|
| 状态 | **调研 + 问题分析 · 供方案 [ext-ai-optimize.md](../design/ext-ai-optimize.md) 参考** |
| 日期 | 2026-10-16 |
| 前置 | [agent-frameworks-and-marqdo.md](agent-frameworks-and-marqdo.md) · [agent-framework-gaps-after-a4.md](agent-framework-gaps-after-a4.md) · [agent-framework-2026-09.md](agent-framework-2026-09.md) · [ai-agent-sandbox.md](ai-agent-sandbox.md) · [okf-and-marqdo.md](okf-and-marqdo.md) |
| 本仓能力 | [ext-agent.md](../design/ext-agent.md) · [okf.md](../design/okf.md) · [engineering-knowledge.md](../design/engineering-knowledge.md) · [ext-ai-sandbox.md](../design/ext-ai-sandbox.md) · [ai-stack.md](../roadmap/ai-stack.md) · [ext-agent-optimize.md](../roadmap/ext-agent-optimize.md) |
| 外部抽样 | Anthropic Agent Skills · SoK Agentic Skills (arXiv 2602.20867) · ForgeAgent · ACES / Skill Lift · Switchcraft 路由 · DSPy/GEPA ·「工作流编译进权重」· MalSkillBench |

---

## 0. 一句话

业界正把 Agent 优化从「更好的 chat loop」转向 **可验证、可复用、可隔离执行的技能层**；Marqdo 已具备 **可执行 Markdown（代码即文档）+ OKF/EKC 复用 + CoW 沙盒** 三件套，优化目标应是把这三件套焊成闭环：**试跑 → 证据 → 晋升 → 零/少 LLM 复用**，而不是追平 LangChain 功能清单。

---

## 1. 调研方法

| 动作 | 说明 |
|------|------|
| 扫业界技能层 | Anthropic Skills（渐进披露 `SKILL.md`）；SoK Agentic Skills 生命周期；ForgeAgent「能力铸造」；ACES Skill Lift |
| 扫成本/路由 | Switchcraft 工具调用路由；DSPy+GEPA 提示优化；工作流编译进小模型权重 |
| 扫安全 | MalSkillBench 运行时验证恶意技能；沙盒粒度讨论（per-skill / per-session） |
| 对照本仓 | A0–A4 / Wave B 已落地；沙盒 S0–S4 已落地；OKF + EKC + adaptive routing 现状 |

**不修改** `doc/next/*`（手写真相源）；本文件只进 `doc/research/`。

---

## 2. 业界态势（与 Marqdo 相关的收敛点）

### 2.1 技能层取代「纯工具调用叙事」

| 来源 | 要点 | 对 Marqdo |
|------|------|-----------|
| Anthropic Agent Skills | 目录 + `SKILL.md`；元数据预载、正文按需；脚本可执行 | 形态近 OKF 任务包，但 Marqdo 技能 **本身可跑** |
| SoK Agentic Skills | 发现→练习→蒸馏→存储→组合→评测→更新；代码技能需沙盒 | 生命周期与 OKF promote / sandbox.run 对齐 |
| ForgeAgent | 生成能力 → 沙盒证明 → 信任复用；失败进回归隔离 | **验证后晋升** 正是 solidify / `llm_free` 缺的硬闸 |
| ACES | 有/无技能配对试验 → Skill Lift | 可作 Marqdo 评测 harness 指标 |

### 2.2 成本：路由 + 编译 + 小模型

| 来源 | 要点 | 对 Marqdo |
|------|------|-----------|
| Switchcraft | 工具调用场景路由到最便宜仍正确的模型；报道大幅降本 | 已有 `marqdo \| small-llm \| llm \| auto`；可加 **工具步/试跑步** 细分 |
| DSPy / GEPA | 用反馈改写指令，小模型追上大模型 | 优化对象应是 **`.mq.md` 提示文档 / standing / 路由策略文件**，不是黑盒字符串 |
| 工作流编译进权重 | 程序化流程进小模型权重，上下文常数级 | 长期对应「高频 OKF 技能 → 专用小模型」；短期优先 **llm_free 源码路径** |

### 2.3 安全：技能不可信 + 沙盒必验

恶意技能基准与 SoK 均强调：**技能说明与代码默认不可信**，须运行时隔离。Marqdo 新沙盒（默认拒 plugin/net/exec）正好是技能晋升前的证明场。

---

## 3. Marqdo 已有资产（优化起点）

| 资产 | 现状 | 优化杠杆 |
|------|------|----------|
| 代码即文档 | `.mq.md` = 程序 = 提示工作面；GFM 表 / 链接取元 | 减少 json 胶水；让模型「看见」结构 |
| 写回 | `marqdo-out` 槽；step 默认写回 | 晋升证据、失败避坑 |
| OKF / agent-kb | 命中 / near / promote / solidify / `llm_free` | 复用率、降本主轴 |
| EKC / EFI | L0–L4 catalog；preflight REUSE/ADAPT/CREATE | 生成前先查工程知识 |
| Adaptive routing | marqdo / small-llm / llm / auto + policy 文件 | 再细分 sandbox 试跑 vs 开放推理 |
| Sandbox | CoW + capped `run`；`ext/ai/sandbox` | **试跑闸门**；未 commit 不污染仓库 |
| MCP / corpus | 证据工具，`authority=workbook` | 保持不升权威 |

---

## 4. 问题分析（差距，不是功能羡慕）

### P1 — 试跑与晋升未焊死

沙盒已能 CoW 试跑，但 **`agent.step` / `plan` 默认仍可直接改真实树**；OKF `llm_free` 晋升不强制「沙盒金样绿」。  
→ 风险：未验证技能进入复用池（业界 ForgeAgent / MalSkillBench 已证明此问题）。

### P2 — 技能形态双轨

Anthropic Skills = 文件夹 + 渐进披露；Marqdo = 可执行 `.mq.md` + OKF 类型。  
→ 机会：用 **OKF frontmatter + 可执行体** 统一，不必引入第二套 SKILL.md 权威；缺的是 **披露层级与验证元数据**（适用条件、fixture、sandbox profile）。

### P3 — 路由粒度偏粗

现有路由偏「整任务后端」；未区分：查 OKF / 写补丁 / 沙盒 `run` / 对外 HTTP。  
→ Switchcraft 启示：按 **步骤类型** 选模型，且「便宜模型推理 token 爆炸」要计入总成本。

### P4 — 评测未锁差异化

Wave B1 仍 Active：缺稳定 **Skill Lift / 沙盒证明率 / llm_free 命中率 / 文档可读性** 指标。  
→ 没有指标，优化会滑回「多调几次模型」。

### P5 — 代码即文档在 Agent 产出侧仍易腐化

模型仍可能吐 `json.set` 链、混用 EN/ZH API、跳过表结构。  
→ 沙盒可跑通语法，但 **风格门禁**（lint / preflight）未默认卡在 commit 前。

### P6 — 上下文与 EKC 未默认进每步

EFI / catalog 能力在，但 step 默认上下文仍可能偏源码切片，缺少「先 REUSE」硬门槛。

### P7 — LLM 表面与 Agent 成本账未统一

`llm.ask` / `stream` / usage 已有；Agent 侧缺 **按会话/技能的成本仪表盘** 写回文档（可 git 审）。

---

## 5. 优化目标函数（建议）

在固定任务集上最大化：

\[
\begin{align*}
& \text{ReuseRate}(\text{OKF hit} + \text{llm\_free}) \\
&+ \text{SandboxProofRate}(\text{试跑绿再 commit/promote}) \\
&- \lambda_1 \cdot \text{TokenCost} - \lambda_2 \cdot \text{HostMutationsWithoutCommit}
\end{align*}
\]

约束：权威真相仍是 `.mq.md`；不引入隐藏 TOOL 主循环；不改 `doc/next/*`。

---

## 6. 外部资料索引（抽样）

| 类型 | 链接 |
|------|------|
| Skills | [Anthropic engineering: Agent Skills](https://www.anthropic.com/engineering/equipping-agents-for-the-real-world-with-agent-skills) · [Claude Code skills](https://code.claude.com/docs/en/skills) · [agentskills / anthropics/skills](https://github.com/anthropics/skills) |
| 论文 | [SoK: Agentic Skills](https://arxiv.org/html/2602.20867) · [Skill Issue (repo SKILL 优化)](https://arxiv.org/html/2609.12742v1) · [ACES Skill Lift](https://arxiv.org/html/2608.20614) · [MalSkillBench](https://arxiv.org/html/2606.07131v3) · [Compiling workflows into weights](https://arxiv.org/html/2605.22502) |
| 开源/产品 | [ForgeAgent](https://github.com/Yashasm18/ForgeAgent) · [DSPy GEPA](https://dspy.ai/current/getting-started/gepa-optimization/) · [Switchcraft](https://www.microsoft.com/en-us/research/publication/switchcraft-ai-model-router-for-agentic-tool-calling/) |

---

## 7. 结语

Marqdo 不缺「再一个 Agent SDK」；缺的是把 **文档真相、知识晋升、沙盒证明、模型路由** 收成可度量闭环。下一篇方案文档按问题编号给出可选方向与推荐波次，**不预设唯一银弹**。
