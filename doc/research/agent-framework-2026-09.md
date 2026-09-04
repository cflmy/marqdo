# 调研刷新：2026-09 智能体框架与 Marqdo「代码即文档」

| | |
|---|---|
| 状态 | **Wave B1 Active**（B0/B0.5 已完成；宪法优先） |
| 日期 | 2026-09-04 |
| 前置 | [agent-frameworks-and-marqdo.md](agent-frameworks-and-marqdo.md) · [agent-framework-gaps-after-a4.md](agent-framework-gaps-after-a4.md) |
| 路线 | [ext-agent-optimize.md](../roadmap/ext-agent-optimize.md) |

## 0. 结论

业界框架（LangGraph / Microsoft Agent Framework / CrewAI / OpenAI Agents SDK）在编排、工具、检查点、MCP、评测上继续收敛；**权威真相仍默认在对象图 + transcript + 外置 Store**。

Marqdo 的差异化不是「再做一个 loop」，而是：

1. **代码即文档** — 编排 `.mq.md` = 提示词工作面  
2. **调用位置可见** — `agent_call_site`（路径 / 函数 / 行）是一等上下文  
3. **文档即知识库（OKF）** — 写回 → promote / solidify → 二次命中 / `llm_free`

Wave B 的目标函数：**证明并硬化上述三点**；产品化示例与评测必须服务差异化，禁止做成无源码上下文的 chatbot demo。

## 1. 业界快照（2026）

| 模式 | 代表 | 状态记忆 | 对 Marqdo |
|------|------|----------|-----------|
| 图 + checkpointer | LangGraph | typed state 持久化 | 续跑应对齐**文件化工作簿**，不造 Python 图 DSL |
| Agent vs Workflow | Microsoft AF | 自主 + 确定性流 | 已有 step / plan，强化即可 |
| 角色班组 | CrewAI | Task 管道 | 多 Agent 低优先；父 CALL + 多工作簿 |
| Handoff + Guardrails | OpenAI Agents SDK | 显式交接 | 可借鉴语义；权威仍在 `.mq.md` |

## 2. 深远影响（开发硬约束）

| 约束 | 含义 |
|------|------|
| 提示不黑盒 | standing / 工具 / 任务写在文件；拼装可审查 |
| 位置感知 | 每次推理注入 call_site；示例与评测可断言 |
| 源码工作面 | `source_brief` + `READ:source\|skill`；禁止藏整份 Skill 进不可见提示 |
| 写回即记忆 | ok/error 槽 + view；非默认向量库 |
| OKF 飞轮 | promote / solidify / hit / llm_free |
| 外置工具 | corpus / MCP 仅证据，`authority=workbook` |

## 3. Wave B 范围

| 波次 | 主题 |
|------|------|
| **B0** | 宪法可见示例 + 上下文管道可读性 + Skill/文档 |
| **B0.5** | 本机 live（`.env` → cflmy）验证「模型用上了文档」 |
| **B1** | 评测锁住 call_site / OKF / 写回 / 预算 |

暂缓：真 MCP、resume/HITL view、kb stats、并行、A2A。

## 4. 开发期 LLM

仓库根目录 `.env`（gitignore）：`OPENAI_*` 或 `MARQDO_LLM_*`。模板见 `.env.example`。
