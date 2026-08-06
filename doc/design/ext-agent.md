# Official agent development framework (`ext/ai/agent`)

| | |
|---|---|
| Status | Accepted · **thin agent loop locked** |
| Date | 2026-08-06 |
| Related | [ext-llm.md](ext-llm.md) · [ext-cli.md](ext-cli.md) · [objects.md](objects.md) |

## What we are building

An **agent-development framework** — smallest runtime surface for agents on Marqdo.

**Not** task-dispatch, **not** bundled domain tools, **not** layout/match/ticket helpers in official ext.

| In scope | Out of scope (your runbook / your code) |
|----------|-------------------------------------------|
| `# 智能体` ctor | `## 技能匹配` · `## 分配任务` · `## 创建工单` … |
| `## 执行` | Anything task-specific in `ext/` |
| `## 清空历史` | |
| Auto context: source, position, history, Marqdo skill | |

## Repo layout (`ext/`)

Official extensions for LLM + agent live under **`ext/ai/`** only. No flat `.mq.md` at `ext/` root; future domains get their own subdir.

```text
ext/
  ai/
    llm.mq.md
    大模型.mq.md
    agent.mq.md
    智能体.mq.md
```

Import: `> ext/ai/智能体.mq.md` · `> ext/ai/llm.mq.md`

## Layering

| Layer | Role |
|-------|------|
| **Your runbook** | Tables, loops, **your** tools & logic |
| **`ext/ai/llm`** | Model I/O |
| **`ext/ai/agent`** | Framework: ctor + `执行` + `清空历史` |
| **`lib/*`** | Generic I/O |

```text
> ext/ai/智能体.mq.md
    ▼
# 智能体 (大模型, 工具, 用户提示信息?)
    ## 执行
    ## 清空历史
```

`工具` = caller-supplied **function catalog** (table of `##` names); framework resolves names via `call_fn` / `调用` and runs the matching zero-arg function in the runbook module.

Install: [`ext-cli.md`](ext-cli.md) — `marqdo ext add agent` / `add llm`.

### Tools (v1)

Tools are **not** framework builtins — you define `## 函数名` in your runbook and list names in a table:

```markdown
`工具表` =
| 工具 |
|------|
| 获取时间 |
| 分配任务 |
```

- Single-column table → list of function **names** (text).
- Multi-column table → each row is a map; name from column `工具` / `tools` / `name`.
- Marqdo has no first-class function values in tables yet; the name must match a `##` in the same module (or imports). `执行` calls it with `call_fn` after the model replies `TOOL:<name>`.
- Do **not** use `parse text=["fn"]` for the tools list — unquoted identifiers inside `parse` are evaluated as calls.

---

## Public API (v1)

| | Role |
|--|------|
| `# 智能体` / `# agent` | **大模型**, **工具**, optional **用户提示信息** |
| `## 执行` / `## run` | **额外信息**; context + LLM + history append |
| `## 清空历史` / `## clear_history` | Wipe history on this handle |

Module-level `##` helpers (e.g. context assembly) are internal — not public contract.

### Auto context (each `执行`)

- `.mq.md` source
- Call-site / position
- Conversation history on this handle
- Marqdo skill (`skills/marqdo/`)

### History (v1)

- Host-backed id per handle
- Append after each successful `执行`
- `清空历史` ships with `执行`

---

## Example runbook (application — not framework)

```markdown
---
> ext/ai/智能体.mq.md
> ext/ai/大模型.mq.md
---

# main

*`模型` = > 大模型 *
*`工具名` = | 技能匹配 | 分配任务 | … |
*`助手` = > 智能体 大模型=`模型` 工具=`工具名` 用户提示信息=… *

- [任务](任务队列)
  > `助手`.清空历史
  *`结果` = > `助手`.执行 额外信息=`任务` *
```

Domain `##` functions live in **this file** or your lib — never in `ext/ai/`.

---

## Shipped / roadmap

| Item | Status |
|------|--------|
| `ext/ai/` + framework + context injection | in progress |
| Live `执行` test (`tests/ext/agent-run-live.mq.md` + `.env`) | shipped |
| Tool-call loop in `执行` (`TOOL:<name>` → `call_fn`) | shipped |

### Non-goals

- Domain tools in official ext
- Flat `ext/*.mq.md` at repo root
- Second chat stack beside llm
