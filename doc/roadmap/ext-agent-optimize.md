# `ext/agent` 深入优化（调研驱动）

| | |
|---|---|
| 状态 | **A0–A4 完成**；**Wave B Active**（宪法优先：代码即文档 / call_site / OKF） |
| 日期 | 2026-09-04 |
| 调研 | [agent-frameworks-and-marqdo.md](../research/agent-frameworks-and-marqdo.md) · [agent-framework-gaps-after-a4.md](../research/agent-framework-gaps-after-a4.md) · **[agent-framework-2026-09.md](../research/agent-framework-2026-09.md)** |
| 设计 | [ext-agent.md](../design/ext-agent.md) · [ext-agent-plan.md](../design/ext-agent-plan.md) · [ext-agent-parent.md](../design/ext-agent-parent.md) · [okf.md](../design/okf.md) |
| 邻近 | [agent-streaming.md](agent-streaming.md) · [okf-near-match.md](okf-near-match.md) |

---

## 1. 优化原则（来自调研）

1. **强化差异化，不向 LangChain 薄循环回退**（禁止隐藏 TOOL: 主路径、禁止整文件 LLM 重写工作簿）。  
2. 把资源砸在：**工作簿质量、写回、固化、OKF 命中、过程可视、源码/调用位置注入**。  
3. 外部 RAG/MCP 只作**工具**，不升为权威真相源。

自评四问见调研 §9。A0–A4 之后的缺口见 [agent-framework-gaps-after-a4.md](../research/agent-framework-gaps-after-a4.md)。**2026-09 Wave B** 锁定见 [agent-framework-2026-09.md](../research/agent-framework-2026-09.md)。

---

## 2. A0–A4（已完成）

| 波次 | 主题 | 验收线索 |
|------|------|----------|
| **A0** | 工作簿补丁 / 固化可靠性 | **done** — 金样 `agent-workbook-patch-a0` |
| **A1** | OKF 复用飞轮 | **done** — 金样 `agent-kb-plan-hit` |
| **A2** | 过程可见 | **done** — events + view plan 卡 |
| **A3** | 上下文预算 | **done** — `source_brief` / `READ:`；金样 `agent-context-budget-a3` |
| **A4** | RAG/MCP 证据工具 | **done** — 金样 `agent-tools-rag-a4` |

---

## 3. Wave B（Active — 宪法优先）

| 波次 | 主题 | 状态 |
|------|------|------|
| **B0** | 官方示例演示 call_site/源码/OKF；Skill/文档；上下文管道可读 | **done** |
| **B0.5** | 本机 live（`.env` → 开发模型）验证「模型用上了文档」 | **done**（`AGENT_LIVE` / `AGENT_HARNESS_LIVE`） |
| **B1** | 评测 harness 锁差异化指标（constitution / dump / writeback 遮蔽） | **Active** |
| **B2–B5** | 真 MCP / resume+HITL / stats / 并行 | 暂缓 |

示例：`examples/agent-pong/` · `examples/agent-okf-flywheel/`。
