# Marqdo AI 栈统一开发方案（llm + agent）

| | |
|---|---|
| Status | **Active** |
| Date | 2026-09-24 |
| Source | [doc/next/002–005](../next/) · [ext-llm.md](../design/ext-llm.md) · [ext-agent.md](../design/ext-agent.md) |

## 1. 一句话

> **Document → Prompt → LLM → Agent → Skill → Policy → Executable Knowledge**

大模型负责探索，小模型/可选 Jev 负责决策，Marqdo 负责执行；经验编译进 `.mq.md` 后推理成本下降。

## 2. 分层（勿混）

| 层 | 职责 | 落点 |
|----|------|------|
| **Intelligence primitive** | `ask` / `stream` / `collect`；结果对象 | `ext/ai/llm` |
| **Agent runtime** | `run` / `step` / `plan` / `route` | `ext/ai/agent` |
| **Memory / compile** | episode → skill → policy | `plugins/agent`（ABI；**不进** `src/host`） |
| **Transport** | HTTP / SSE / provider | `ext/ai/llm/openai` · `ollama`；用户不可见 |

硬规则：`ext/**` 不调 `host_*`；agent/OKF/路由热路径只进 `plugins/agent`。

## 3. LLM（005）— 已/待

| 项 | 状态 |
|----|------|
| 语义 API：`ask` / `stream` / `collect`（`complete` 降为兼容原语） | **done** |
| `create` 工厂；handle 带 `backend` | **done** |
| LLMResult：`text` / `model` / `finish` / `backend` / `usage` / `tokens` | **done** |
| Named models（`llm.fast` / `llm.reasoning`） | **done** |
| Prompt document：`prompt` / `prompt_load` + `type: prompt` 正文 | **done**（无参 `ask`/`stream` → `sys.module_source` + `prompt_body`） |
| 拆 `ext/ai/llm/openai.mq.md` + `ollama` 后端 | **done** |
| tool_calls 完整 Result | 暂缓 |
| Transport → 可选 ABI 插件 | 暂缓 |
| Artifact Metadata Binding（`${env.*}` / `secret` · `sys.meta`） | **done**（Phase 1 Metadata-only + Phase 2 scope lift；[binding.md](../design/binding.md)） |

作者面优先：

```marqdo
**answer = > llm.ask prompt="What is Marqdo?"**
**events = > model.stream prompt=`p`**
**text = > llm.collect events=`events`**
**fast = > llm.fast**
**prompt = > llm.prompt path="prompts/task.md"**
```

## 4. Agent（002–004）— 已/待

| 项 | 状态 |
|----|------|
| Skill Compilation：episode → maybe_learn → `llm_free` | **done** |
| Adaptive Routing：`marqdo` \| `small-llm` \| `jev?` \| `llm` \| `auto` | **done** |
| Policy Compilation → `policies/router.mq.md` | **done** |
| `run` 串联 route → execute → record → learn | **done** |
| Agent 默认走 `model.ask` + Result.usage 记成本 | **done**（`model_turn`） |
| Prompt artifact / document-as-prompt | **done**（经 llm `path=` / `prompt`） |
| 零 LLM 执行（policy+skill → `llm_calls=0`） | **done** |
| `plan` 不覆盖已有 `llm_free` resource | **done** |
| 真 MCP client / resume / Jev 接线 / 小模型 live 路由 | 暂缓 |

## 5. 联调顺序

1. **LLM Result → Agent metrics** — **done**  
2. **Named handles** — **done**  
3. **Prompt document** — **done**（`type: prompt` 剥 frontmatter；无参 `ask`/`stream` 经 `sys.module_source`）  
4. **Backend split** — **done**  
5. **Execution compilation** — **done**（`agent-zero-llm` 金样 + harness）
6. **Metadata Binding** — **done**（Phase 1 + Phase 2 scope lift；`llm-meta-offline` · `meta-lift`）

## 6. 验收金样

- Agent：`tests/ext/agent-v2-learn.mq.md` · `agent-p4-route.mq.md` · `agent-zero-llm.mq.md` · `agent-plan-preserve.mq.md` · `scripts/agent-harness.sh`  
- LLM：`tests/ext/llm-ask-offline.mq.md` · `llm-stream-offline.mq.md` · `llm-ctor-offline.mq.md` · `llm-named-offline.mq.md` · `llm-meta-offline.mq.md` · `llm-entry-prompt-offline.mq.md`
- Binding：`tests/structure/meta-binding.mq.md` · `meta-lift.mq.md`
