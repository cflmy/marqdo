# Official agent development framework (`ext/agent`)

| | |
|---|---|
| Status | Accepted (v0.1 layout) · **roadmap recorded** |
| Date | 2026-08-06 |
| Related | [ext-llm.md](ext-llm.md) · [ext-cli.md](ext-cli.md) · [ext-abi.md](ext-abi.md) · [objects.md](objects.md) · [markdown-mapping.md](markdown-mapping.md) |

## Positioning

`ext/agent` / `ext/智能体` is an **official agent-development framework**, not a single chatbot wrapper.

| Layer | Role |
|-------|------|
| **Marqdo program** | Orchestration: tables, loops, branches, narration — **prompts and control flow live in the same `.mq.md`** |
| **`ext/llm`** | Model I/O (chat / complete); agent framework **imports and uses** it |
| **`ext/agent`** | Workspace conventions + (roadmap) match / assign / report helpers; optional **ABI** for heavy or OS-bound work |
| **`lib/*`** | fs / json / sys / net / table — generic I/O |

**Why Marqdo:** an agent runbook is readable docs and executable policy at once. Skill matrices, task queues, and escalation rules stay next to the prose that explains them.

```text
user agent .mq.md
    │  > ext/agent.mq.md   (+ often > ext/llm.mq.md)
    ▼
framework (# agent / # 智能体 + helpers)
    ├── ext/llm          → model
    ├── lib/fs|json|…    → data / side effects
    └── plugins/agent    → ABI helpers (layout today; more later)
```

Install story for end users (not yet shipped): [`ext-cli.md`](ext-cli.md) — `marqdo ext add agent` / `add llm`.

---

## Shipped in v0.1 (layout)

**Layout framework** for agent-oriented projects:

- Convention dirs under a project root
- Native helpers via **ABI v1** (`plugins/agent`)
- Marqdo object API in `ext/agent.mq.md` / `ext/智能体.mq.md`

### Project layout

| Path | Role |
|------|------|
| `agents/` | Agent programs (`.mq.md`) |
| `runbooks/` | Executable handbooks / checks |
| `templates/` | Scaffold sources |
| `reports/` | Generated artifacts (convention: gitignore) |

Root markers (any): directory `agents` or `runbooks`, or file `marqdo.agent.json`.

### ABI surface (`plugins/agent`)

| Name | Params | Result |
|------|--------|--------|
| `agent_find_root` | `start`, `markers` | text path |
| `agent_ensure_layout` | `root` | int (dirs created) |
| `agent_probe` | `root` | map (`has_agents`, …, `root`) |
| `agent_scaffold` | `root`, `name`, `template`, `dest` | path written; body substitutes `{{name}}` |

Path safety: under `root`; reject `..`. Build: `cargo build -p marqdo_plugin_agent`.

### Marqdo surface (v0.1)

| English | Chinese | Role |
|---------|---------|------|
| `## load_native` | `## 加载原生` | Env `MARQDO_AGENT_PLUGIN` → `lib/plugin` load |
| `# agent` | `# 智能体` | Workspace handle; optional `root=` |
| `## probe` | `## 探测` | Layout probe |
| `## ensure_layout` | `## 确保布局` | Create convention dirs |
| `## scaffold` | `## 脚手架` | `name=` `template=` `dest=` |

Gold: `tests/ext/agent-scaffold.mq.md`.

---

## Target product (record — not all implemented)

Framework should make it natural to write agents like **task assignment**: read queues and skill tables, match people, assign or escalate — with optional LLM for soft matching / message drafting.

### Orchestration sketch (illustrative)

Syntax below is **aspirational**; names need not match final APIs. Point is: **tables + loop + framework methods + narration**.

```markdown
---
> ext/agent.mq.md
> ext/llm.mq.md
> lib/table.mq.md
---

# main

> load_env
> load_native

*`model` = > llm *
*`bot` = > agent *

团队成员技能表

*`技能表` =
| 成员 | 技能 | 负载 |
|------|------|------|
| 张三 | Python, ML | 2 |
| 李四 | 前端, UX | 1 |
| 王五 | 后端, 运维 | 3 |
| 赵六 | 数据分析 | 0 |

*`任务队列` =
| 任务ID | 所需技能 | 优先级 |
|--------|----------|--------|
| T001 | Python | 高 |
| T002 | 前端 | 中 |
| T003 | 安全审计 | 高 |

> 分配策略 任务队列=`任务队列` 技能表=`技能表` model=`model` bot=`bot`

## 分配策略
    - 任务队列
    - 技能表
    - model
    - bot

- [任务](任务队列)
  *`匹配` = > `bot`.技能匹配 技能=`任务`.所需技能 成员表=`技能表` model=`model` *

  + `匹配`
    *`结果` = > `bot`.分配任务 任务ID=`任务`.任务ID 分配给=`匹配` *
    + …
      > `bot`.通知 …   # or lib/net / future channel helpers
  + *
    > `bot`.创建工单 标题=… 详情=…

> 完成报告
```

Marqdo strengths here:

1. **Prompt next to policy** — escalation wording and match rules sit in the same file as the loop.
2. **Tables as data** — skill matrix / queue are first-class collections.
3. **LLM as a tool** — `ext/llm` for fuzzy skill match or message text; deterministic path when tables suffice.
4. **Objects** — `# agent` / `# llm` handles; methods keep the runbook short.

Exact helper names (`技能匹配` / `match_skill`, `分配任务` / `assign`, …) to be fixed when implementing; may be pure Marqdo over `lib/table` + `ext/llm`, with ABI only where native code helps.

### Roadmap slices (ordered)

| Slice | Content |
|-------|---------|
| **A — Install** | [`ext-cli.md`](ext-cli.md): **shipped** `marqdo ext list` / `add` / `remove` |
| **B — Compose LLM** | `ext/agent` frontmatter imports `ext/llm`; helpers that take a model handle or call `complete` internally |
| **C — Match / assign** | Table-oriented match (substring / tag) + optional LLM rerank; assign + load update; escalate / ticket stubs (print or `reports/` files first) |
| **D — Channels** | Optional notify hooks (mail/chat) via `lib/net` or thin ABI — still no third-party marketplace |
| **E — Multi-agent** | Later: handoff between agent programs; out of scope until C is solid |

### Non-goals (still)

- Third-party plugin registry / arbitrary remote packages
- Embedding native plugins inside the main `marqdo` exe by default
- Replacing `ext/llm` with a second chat stack inside agent
