# Official extensions (`ext/`) — LLM

| | |
|---|---|
| Status | **Accepted · v0.3 semantic**（005：Intelligence Primitive） |
| Date | 2026-09-24 |
| Related | [objects.md](objects.md) · [ext-agent.md](ext-agent.md) · [ext-cli.md](ext-cli.md) · [ai-stack.md](../roadmap/ai-stack.md) · [doc/next/005.md](../next/005.md) |

## Scope

`ext/ai/llm` is Marqdo’s **intelligence primitive**, not an OpenAI SDK wrapper.

```text
Author surface:  ask / stream / collect / create / fast / reasoning / prompt_load
Transport:       ext/ai/llm/openai.mq.md  (+ ollama defaults)
Compat:          complete / chat
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
| `## fast` / `## reasoning` | `## 快速` / `## 推理` | Named handles |
| `## prompt_load` / `## prompt` / `## prompt_body` | `## 加载提示` / `## 提示` / `## 提示正文` | Prompt artifacts (`type: prompt` → body) |
| `## ask` (module) | `## 提问` | Convenience → **text** (`prompt=` or `path=`) |
| `## collect` | `## 收集` | Events → text |
| `## stream_result` | `## 流式结果` | Alias of `collect` |
| `# llm` | `# 大模型` | Handle ctor (`backend=` openai-compatible \| ollama) |
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

**fast = > llm.fast**
**events = > `fast`.stream prompt=`prompt`**
**text = > llm.collect events=`events`**

**body = > llm.prompt path="fixtures/prompt-marqdo.md"**
**body2 = > llm.prompt text=`raw`**
```

### LLMResult

| Field | Meaning |
|-------|---------|
| `text` | Answer string |
| `model` | Model id |
| `finish` | Finish reason (`stop`, …) |
| `backend` | e.g. `openai-compatible` / `ollama` |
| `usage` | `{prompt_tokens,completion_tokens,total_tokens}` |
| `tokens` | Alias of `usage.total_tokens` (agent metrics) |
| `name` | Optional handle tag (`fast` / `reasoning`) |
| `llm_calls` | `1` per ask |

### Compat

`complete` / `chat` still return a bare string (or event list when `stream=True`) so `ext/ai/agent` and older runbooks keep working. **New code should use `ask` / `stream` / `collect`.**

## Environment

```env
OPENAI_API_KEY=sk-...
OPENAI_BASE_URL=https://api.openai.com/v1
OPENAI_MODEL=gpt-4o-mini
MARQDO_LLM_FAST_MODEL=...
MARQDO_LLM_REASONING_MODEL=...
OLLAMA_HOST=http://127.0.0.1:11434/v1
```

Fallbacks: `MARQDO_LLM_API_KEY`, `MARQDO_LLM_BASE_URL`, `MARQDO_LLM_MODEL`. Ctor also accepts `api_key=` / `base_url=` / `model=` / `backend=`.

## Backends

| `backend=` | Module | Notes |
|------------|--------|-------|
| `openai-compatible` (default) | `ext/ai/llm/openai.mq.md` | Chat Completions HTTP + SSE |
| `ollama` | defaults in ctor + same transport | Key defaults to `ollama`; host from `OLLAMA_HOST` |

## Tests

- `tests/ext/llm-import.mq.md`
- `tests/ext/llm-ctor-offline.mq.md`
- `tests/ext/llm-ask-offline.mq.md` (create + collect)
- `tests/ext/llm-stream-offline.mq.md`
- `tests/ext/llm-named-offline.mq.md` (fast/reasoning + prompt_load)
- Live: `llm-complete.mq.md` · `llm-stream-live.mq.md`
