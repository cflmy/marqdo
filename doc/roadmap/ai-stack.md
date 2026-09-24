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
| **Transport** | HTTP / SSE / provider | llm 内部后端；用户不可见 |

硬规则：`ext/**` 不调 `host_*`；agent/OKF/路由热路径只进 `plugins/agent`。

## 3. LLM（005）— 已/待

| 项 | 状态 |
|----|------|
| 语义 API：`ask` / `stream` / `collect`（`complete` 降为兼容原语） | **done** |
| `create` 工厂；handle 带 `backend` | **done** |
| LLMResult：`text` / `model` / `finish` / `backend` | **done**（最小字段） |
| Prompt = Document / `llm.ask` 无参吃正文 | 暂缓 |
| Named models（`fast` / `reasoning`） | 暂缓 |
| 拆 `ext/ai/llm/openai.mq.md` 等多后端 | 暂缓 |
| usage / tool_calls 完整 Result | 暂缓 |
| Transport → 可选 ABI 插件 | 暂缓 |

作者面优先：

```marqdo
**answer = > llm.ask prompt="What is Marqdo?"**
**events = > model.stream prompt=`p`**
**text = > llm.collect events=`events`**
```

## 4. Agent（002–004）— 已/待

| 项 | 状态 |
|----|------|
| Skill Compilation：episode → maybe_learn → `llm_free` | **done** |
| Adaptive Routing：`marqdo` \| `small-llm` \| `jev?` \| `llm` \| `auto` | **done** |
| Policy Compilation → `policies/router.mq.md` | **done** |
| `run` 串联 route → execute → record → learn | **done** |
| Agent 默认走 `model.ask` + Result.usage 记成本 | 下一步 |
| Prompt artifact / document-as-prompt | 与 LLM Prompt 层一起做 |
| 真 MCP client / resume / Jev 接线 / 小模型 live 路由 | 暂缓 |

## 5. 联调顺序（继续开发）

1. **LLM Result → Agent metrics** — `step`/`plan` 记 `llm_calls`/`tokens` 从 Result 取。  
2. **Named handles** — `llm.create model=…` 命名 fast/reasoning，接到 `run router_model=`。  
3. **Prompt document** — `type: prompt` 文件 / 当前文档切片作 ask 输入。  
4. **Backend split** — openai-compatible 抽到子模块；ollama 第二后端。  
5. **Execution compilation** — policy+skill 全路径零 LLM 金样与 harness 指标。

## 6. 验收金样

- Agent：`tests/ext/agent-v2-learn.mq.md` · `agent-p4-route.mq.md` · `scripts/agent-harness.sh`  
- LLM：`tests/ext/llm-ask-offline.mq.md` · `llm-stream-offline.mq.md` · `llm-ctor-offline.mq.md`
