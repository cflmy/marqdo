# `ext/agent` 深入优化（调研驱动）

| | |
|---|---|
| 状态 | **A0–A4 完成**（调研驱动优化闭环；A4：corpus/MCP 证据工具，权威仍是 `.mq.md`） |
| 日期 | 2026-08-28 |
| 调研 | [agent-frameworks-and-marqdo.md](../research/agent-frameworks-and-marqdo.md) |
| 设计 | [ext-agent.md](../design/ext-agent.md) · [ext-agent-plan.md](../design/ext-agent-plan.md) · [ext-agent-parent.md](../design/ext-agent-parent.md) · [okf.md](../design/okf.md) |
| 邻近 | [agent-streaming.md](agent-streaming.md) · [okf-near-match.md](okf-near-match.md) |

---

## 1. 优化原则（来自调研）

1. **强化差异化，不向 LangChain 薄循环回退**（禁止隐藏 TOOL: 主路径、禁止整文件 LLM 重写工作簿）。  
2. 把资源砸在：**工作簿质量、写回、固化、OKF 命中、过程可视**。  
3. 外部 RAG/MCP 只作**工具**，不升为权威真相源。

自评四问见调研 §9。

---

## 2. 建议分期（摘要）

| 波次 | 主题 | 验收线索 |
|------|------|----------|
| **A0** | 工作簿补丁 / 固化可靠性 | **done** — 整文件 FIND 护栏；`soft` 仅 miss；decompose `n` 检查；`workbook_solidify` 单测；金样 `agent-workbook-patch-a0`（双 CONTINUE + solidify） |
| **A1** | OKF 复用飞轮（lookup / near / soft_match UX） | **done** — `list_tasks` 含 description/aliases/status/llm_free/hits；`plan` 命中路径暴露 `match`/`score`；soft_match 策展行含 meta；view plan 卡显示 match；金样 `agent-kb-plan-hit`（exact hit + near soft-hit） |
| **A2** | 过程可见（stream + view 过程卡） | **done** — `plan_append_*` / `plan_finish_stream` 始终写入 `events`（SSE 仍仅 `stream=True`）；OKF REUSE 记 `decision`；view `plan-card` 渲染过程时间线（跳过 delta）；金样断言 `events-ok`；对齐 [agent-streaming.md](agent-streaming.md) |
| **A3** | 上下文预算（源码/Skill 渐进披露） | **done** — `source_brief` / 加深 `skill_brief`；`build_step_context` 默认预算 + `READ:source|skill`；`step max_reads`；父 `READ:skill`；金样 `agent-context-budget-a3` |
| **A4** | 可选：RAG/MCP 工具适配器 | **done** — `corpus_search`（本地语料关键词）；`mcp_list_tools` / `mcp_call`（JSON fixture）；`authority=workbook`；金样 `agent-tools-rag-a4` |

细节与行业对照以调研正文为准；本文件只跟踪实现分期。
