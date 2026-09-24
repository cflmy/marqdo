# Official extensions (`ext/`) — LLM

| | |
|---|---|
| Status | **Accepted · v0.2 semantic**（005：Intelligence Primitive） |
| Date | 2026-09-24 |
| Related | [objects.md](objects.md) · [ext-agent.md](ext-agent.md) · [ext-cli.md](ext-cli.md) · [ai-stack.md](../roadmap/ai-stack.md) · [doc/next/005.md](../next/005.md) |

## Scope

`ext/ai/llm` is Marqdo’s **intelligence primitive**, not an OpenAI SDK wrapper.

```text
Author surface:  ask / stream / collect / create
Internal:        complete (compat) + openai-compatible HTTP
```

Import:

```markdown
---
import llm:ext/ai/llm.mq.md
---
```

## Public API

| English | Chinese | Role |
|---------|---------|------|
| `## load_env` | `## 加载环境` | Load `.env` |
| `## create` | `## 创建` | Factory → LLM handle |
| `## ask` (module) | `## 提问` | Convenience → **text** |
| `## collect` | `## 收集` | Events → text |
| `## stream_result` | `## 流式结果` | Alias of `collect` |
| `# llm` | `# 大模型` | Handle ctor |
| `## ask` (method) | `## 提问` | → **LLMResult** map |
| `## stream` | `## 流式` | → event list |
| `## complete` / `## chat` | `## 运行` / `## 聊天` | Compat / transport primitive |

### Preferred

```markdown
**answer = > llm.ask prompt="What is Marqdo?"**
*answer*

**model = > llm.create**
**result = > `model`.ask prompt=`prompt`**
*[text](result)*

**events = > `model`.stream prompt=`prompt`**
**text = > llm.collect events=`events`**
```

### LLMResult (minimal)

| Field | Meaning |
|-------|---------|
| `text` | Answer string |
| `model` | Model id |
| `finish` | Finish reason (`stop`, …) |
| `backend` | e.g. `openai-compatible` |

### Compat

`complete` / `chat` still return a bare string (or event list when `stream=True`) so `ext/ai/agent` and older runbooks keep working. **New code should use `ask` / `stream` / `collect`.**

## Environment

```env
OPENAI_API_KEY=sk-...
OPENAI_BASE_URL=https://api.openai.com/v1
OPENAI_MODEL=gpt-4o-mini
```

Fallbacks: `MARQDO_LLM_API_KEY`, `MARQDO_LLM_BASE_URL`, `MARQDO_LLM_MODEL`. Ctor also accepts `api_key=` / `base_url=` / `model=` / `backend=`.

## Roadmap (see ai-stack.md)

Prompt-as-document, named models (`fast` / `reasoning`), multi-backend split, richer usage/tool_calls — deferred; keep transport behind the semantic API.

## Tests

- `tests/ext/llm-import.mq.md`
- `tests/ext/llm-ctor-offline.mq.md`
- `tests/ext/llm-ask-offline.mq.md` (create + collect)
- `tests/ext/llm-stream-offline.mq.md`
- Live: `llm-complete.mq.md` · `llm-stream-live.mq.md`
