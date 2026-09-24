---
title: ext/ai/llm
description: >-
  Marqdo LLM intelligence primitive — ask / stream / collect / named handles.
  OpenAI-compatible and Ollama HTTP stay behind the semantic surface.
import sys:lib/sys.mq.md
import json:lib/json.mq.md
import fs:lib/fs.mq.md
import text:lib/text.mq.md
import openai:ext/ai/llm/openai.mq.md
---

## load_env
    + `path`=None

Load `.env` from cwd (optional `path=`). Does not override existing variables.

*> sys.load_dotenv path=`path`*

## collect
    + `events`

Collapse a stream event list to final answer text (`done.result` or `error.message`). Prefer this over raw SSE fields. Alias: `stream_result`.

**answer = ""**

- [ev](events)
  **t = [type](ev)**
  1. `t` == "done"
    **answer = [result](ev)**
  2. `t` == "error"
    **answer = [message](ev)**
  3. `t` == "delta"
  4. `t` == "text"
    **chunk = [text](ev)**
    1. `chunk`
      **answer = answer + chunk**
    2. *
      **_ = 1**
  5. *
    **_ = 1**

*answer*

## stream_result
    + `events`

Compatibility alias for `collect`.

*> collect events=`events`*

## usage_from
    + `data`

Extract `{prompt_tokens,completion_tokens,total_tokens}` from an OpenAI-shaped response map (or empty zeros).

*> openai.usage_from data=`data`*

## create
    + `model`=None
    + `base_url`=None
    + `api_key`=None
    + `backend`=openai-compatible
    + `name`=None

Factory for an LLM handle (same as constructing `# llm`). Env fallbacks apply inside the constructor. Optional `name=` tags the handle (`fast` / `reasoning`).

**h = > llm model=`model` base_url=`base_url` api_key=`api_key` backend=`backend` name=`name`**
*h*

## fast
    + `model`=None
    + `base_url`=None
    + `api_key`=None

Named small/fast handle. Model from arg, else `MARQDO_LLM_FAST_MODEL`, else default create model.

1. `model`
  **mdl = model**
2. *
  **mdl = > sys.env_get name="MARQDO_LLM_FAST_MODEL"**

*> create model=`mdl` base_url=`base_url` api_key=`api_key` name="fast" backend="openai-compatible"*

## reasoning
    + `model`=None
    + `base_url`=None
    + `api_key`=None

Named strong/reasoning handle. Model from arg, else `MARQDO_LLM_REASONING_MODEL`, else default create model.

1. `model`
  **mdl = model**
2. *
  **mdl = > sys.env_get name="MARQDO_LLM_REASONING_MODEL"**

*> create model=`mdl` base_url=`base_url` api_key=`api_key` name="reasoning" backend="openai-compatible"*

## prompt_body
    + `src`

If `src` starts with YAML frontmatter declaring `type: prompt` / `类型: 提示`, return the body after the closing `---`; otherwise return `src` unchanged.

**sw = > text.starts_with text=`src` prefix="---"**
1. not `sw`
  *src*
2. *
  **parts = > split value=`src` sep="\n---\n"**
  **n = > len value=`parts`**
  1. `n` < 2
    *src*
  2. *
    **fm = > at value=`parts` index=0**
    **is_p = > text.contains text=`fm` sub="type: prompt"**
    1. not `is_p`
      **is_p = > text.contains text=`fm` sub="类型: 提示"**
    2. *
      **_ = 1**
    1. `is_p`
      **body = > at value=`parts` index=1**
      *> text.str_trim s=`body`*
    2. *
      *src*

## prompt_load
    + `path`

Load a prompt artifact (`.mq.md` / `.md`) from disk for `ask`.

**raw = > fs.read_text path=`path`**
*> prompt_body src=`raw`*

## prompt
    + `path`=None
    + `text`=None

Prompt artifact helper: `path=` loads a file; `text=` strips `type: prompt` frontmatter from an in-memory string.

1. `path`
  *> prompt_load path=`path`*
2. `text`
  *> prompt_body src=`text`*
3. *
  > print text=ext/ai/llm.prompt: pass path= or text=
  > sys.exit code=1

## ask
    + `prompt`=None
    + `path`=None
    + `model`=None
    + `base_url`=None
    + `api_key`=None

Module convenience: create a default handle and return **answer text**. Pass `path=` to load a prompt document, or `prompt=` text.

1. `path`
  **p = > prompt_load path=`path`**
2. `prompt`
  **p = prompt**
3. *
  > print text=ext/ai/llm.ask: pass prompt= or path=
  > sys.exit code=1

**m = > create model=`model` base_url=`base_url` api_key=`api_key`**
**r = > m.ask prompt=`p`**
*[text](r)*

# llm
    + `model`=None
    + `base_url`=None
    + `api_key`=None
    + `backend`=openai-compatible
    + `name`=None

LLM handle — intelligence primitive, not an HTTP client.

Resolution: explicit args → entry Artifact Metadata (`sys.meta_get`) → `sys.env_get` → defaults.

1. `api_key`
  **key = api_key**
2. *
  **key = > sys.meta_get key="api_key"**
  1. not `key`
    **key = > sys.env_get name="OPENAI_API_KEY"**
    1. not `key`
      **key = > sys.env_get name="MARQDO_LLM_API_KEY"**
    2. *
      **_ = 1**
  2. *
    **_ = 1**

1. `backend` == ollama
  1. not `key`
    **key = "ollama"**
  2. *
    **_ = 1**
2. not `key`
  > print text=ext/ai/llm: set OPENAI_API_KEY or MARQDO_LLM_API_KEY (or pass api_key= / metadata api_key)
  > sys.exit code=1
3. *
  **_ = 1**

1. `base_url`
  **url = base_url**
2. *
  **url = > sys.meta_get key="base_url"**
  1. `url`
    **_ = 1**
  2. `backend` == ollama
    **url = > sys.env_get name="OLLAMA_HOST"**
    1. not `url`
      **url = "http://127.0.0.1:11434/v1"**
    2. *
      **_ = 1**
  3. *
    **url = > sys.env_get name="OPENAI_BASE_URL"**
    1. not `url`
      **url = > sys.env_get name="MARQDO_LLM_BASE_URL"**
    2. *
      **_ = 1**
    1. not `url`
      **url = "https://api.openai.com/v1"**
    2. *
      **_ = 1**

1. `model`
  **mdl = model**
2. *
  **mdl = > sys.meta_get key="model"**
  1. `mdl`
    **_ = 1**
  2. *
    **mdl = > sys.env_get name="OPENAI_MODEL"**
    1. not `mdl`
      **mdl = > sys.env_get name="MARQDO_LLM_MODEL"**
    2. *
      **_ = 1**
    1. not `mdl`
      1. `backend` == ollama
        **mdl = "llama3.2"**
      2. *
        **mdl = "gpt-4o-mini"**
    2. *
      **_ = 1**

1. `name`
  **nm = name**
2. *
  **nm = None**

`h` =

| _type | backend | api_key | base_url | model | suffix | bearer | name |
|-------|---------|---------|----------|-------|--------|--------|------|
| llm | `backend` | `key` | `url` | `mdl` | /chat/completions | "Bearer " | `nm` |

*h*

## ask
    + `prompt`=None
    + `path`=None

Semantic ask → **LLMResult** (`text`, `model`, `finish`, `backend`, `usage`, `name`). Prefer `[text](result)` for the answer string.

1. `path`
  **p = > prompt_load path=`path`**
2. `prompt`
  **p = prompt**
3. *
  > print text=ext/ai/llm: ask needs prompt= or path=
  > sys.exit code=1

**pack = > self.complete_result prompt=`p` stream=False echo=False**
**text = > json.get value=`pack` key="text"**
**usage = > json.get value=`pack` key="usage"**
**finish = > json.get value=`pack` key="finish"**

`result` =

| text | model | finish | backend | usage | name | llm_calls | tokens |
|------|-------|--------|---------|-------|------|-----------|--------|
| `text` | [model](self) | `finish` | [backend](self) | `usage` | [name](self) | 1 | [total_tokens](usage) |

*result*

## stream
    + `prompt`=None
    + `path`=None
    + `echo`=False

Semantic stream: return the event list (use `llm.collect` to reduce to text).

1. `path`
  **p = > prompt_load path=`path`**
2. `prompt`
  **p = prompt**
3. *
  > print text=ext/ai/llm: stream needs prompt= or path=
  > sys.exit code=1

*> self.complete prompt=`p` stream=True echo=`echo`*

## complete_result
    + `prompt`
    + `stream`=False
    + `echo`=False

Transport pack via `ext/ai/llm/openai` (Ollama uses the same OpenAI-compatible path). Non-stream → `{text,usage,finish}`; stream → `{events}`.

*> openai.chat_completions base_url=[base_url](self) api_key=[api_key](self) model=[model](self) prompt=`prompt` stream=`stream` echo=`echo` suffix=[suffix](self) bearer=[bearer](self)*

## complete
    + `prompt`
    + `stream`=False
    + `echo`=False

Internal / compatibility primitive. Prefer `ask` / `stream` in new code.

1. `stream`
  **pack = > self.complete_result prompt=`prompt` stream=True echo=`echo`**
  *[events](pack)*
2. *
  **pack = > self.complete_result prompt=`prompt` stream=False echo=`echo`**
  *[text](pack)*

## chat
    + `prompt`
    + `stream`=False
    + `echo`=False

Alias for `complete` (compat).

*> self.complete prompt=`prompt` stream=`stream` echo=`echo`*
