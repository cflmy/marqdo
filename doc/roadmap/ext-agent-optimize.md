# `ext/agent` 深入优化（调研驱动）

| | |
|---|---|
| 状态 | **进行中 · A0 开工**（整文件补丁护栏 + soft 收紧 + solidify 单测） |
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
| **A0** | 工作簿补丁 / 固化可靠性 | **in progress** — `apply_patch_blocks` 禁整文件 FIND；`soft` 仅吞「FIND not found」；decompose CONTINUE 检查 `n`；`workbook_solidify` 单测 |
| **A1** | OKF 复用飞轮（lookup / near / soft_match UX） | 二次同类 `plan` 走快路径；策展可读 |
| **A2** | 过程可见（stream + view 过程卡） | 对齐 [agent-streaming.md](agent-streaming.md)；真相仍可写回 |
| **A3** | 上下文预算（源码/Skill 渐进披露） | 长 runbook 不爆窗，仍保持「源码即提示」 |
| **A4** | 可选：RAG/MCP 工具适配器 | 金样调用外置检索，权威仍是 `.mq.md` |

细节与行业对照以调研正文为准；本文件只跟踪实现分期。
