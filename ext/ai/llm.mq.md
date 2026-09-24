---
title: ext/ai/llm
description: >-
  Marqdo LLM intelligence primitive — ask / stream / collect.
  OpenAI-compatible HTTP is an internal backend, not the authoring surface.
import sys:lib/sys.mq.md
import net:lib/net.mq.md
import json:lib/json.mq.md
import table:lib/table.mq.md
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

## create
    + `model`=None
    + `base_url`=None
    + `api_key`=None
    + `backend`=openai-compatible

Factory for an LLM handle (same as constructing `# llm`). Env fallbacks apply inside the constructor.

**h = > llm model=`model` base_url=`base_url` api_key=`api_key` backend=`backend`**
*h*

## ask
    + `prompt`
    + `model`=None
    + `base_url`=None
    + `api_key`=None

Module convenience: create a default handle and return **answer text** (document-native one-liner).

**m = > create model=`model` base_url=`base_url` api_key=`api_key`**
**r = > m.ask prompt=`prompt`**
*[text](r)*

# llm
    + `model`=None
    + `base_url`=None
    + `api_key`=None
    + `backend`=openai-compatible

LLM handle — intelligence primitive, not an HTTP client. Transport fields stay on the handle for the openai-compatible backend only.

1. `api_key`
  **key = api_key**
2. *
  **key = > sys.env_get name="OPENAI_API_KEY"**
  1. not `key`
    **key = > sys.env_get name="MARQDO_LLM_API_KEY"**
  2. *
    **_ = 1**

1. not `key`
  > print text=ext/ai/llm: set OPENAI_API_KEY or MARQDO_LLM_API_KEY (or pass api_key=)
  > sys.exit code=1
2. *
  **_ = 1**

1. `base_url`
  **url = base_url**
2. *
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
  **mdl = > sys.env_get name="OPENAI_MODEL"**
  1. not `mdl`
    **mdl = > sys.env_get name="MARQDO_LLM_MODEL"**
  2. *
    **_ = 1**
  1. not `mdl`
    **mdl = "gpt-4o-mini"**
  2. *
    **_ = 1**

`h` =

| _type | backend | api_key | base_url | model | suffix | bearer |
|-------|---------|---------|----------|-------|--------|--------|
| llm | `backend` | `key` | `url` | `mdl` | /chat/completions | "Bearer " |

*h*

## ask
    + `prompt`

Semantic ask: one-shot answer as an **LLMResult** map (`text`, `model`, `finish`, `backend`). Prefer `[text](result)` or `result` in agent metrics.

**text = > self.complete prompt=`prompt` stream=False echo=False**

`result` =

| text | model | finish | backend |
|------|-------|--------|---------|
| `text` | [model](self) | stop | [backend](self) |

*result*

## stream
    + `prompt`
    + `echo`=False

Semantic stream: return the event list (use `llm.collect` to reduce to text).

*> self.complete prompt=`prompt` stream=True echo=`echo`*

## complete
    + `prompt`
    + `stream`=False
    + `echo`=False

Internal / compatibility primitive (API-client shape). Prefer `ask` / `stream` in new code. OpenAI-compatible transport only when `backend` is openai-compatible.

**url = [base_url](self) + [suffix](self)**
**auth = [bearer](self) + [api_key](self)**
**headers = > table.put in=None at="Authorization" value=`auth`**

`messages` =

| @ | role | content |
|---|------|---------|
| 1 | user | `prompt` |

`req` =

| model | messages |
|-------|----------|
| [model](self) | `messages` |

1. `stream`
  **req = > table.put in=`req` at="stream" value=True**
  **body = > json.stringify value=`req`**
  **resp = > net.http_post_sse url=`url` body=`body` headers=`headers` echo=`echo`**
  1. [status](resp) == 200
    *[events](resp)*
  2. *
    > print text=ext/ai/llm: HTTP error (stream)
    > print text=[status](resp)
    > sys.exit code=1
2. *
  **body = > json.stringify value=`req`**
  **resp = > net.http_post url=`url` body=`body` headers=`headers`**
  1. [status](resp) == 200
    **data = > json.parse text=[body](resp)**
    *[content]([message]([1]([choices](data))))*
  2. *
    > print text=ext/ai/llm: HTTP error
    > print text=[status](resp)
    > print text=[body](resp)
    > sys.exit code=1

## chat
    + `prompt`
    + `stream`=False
    + `echo`=False

Alias for `complete` (compat).

*> self.complete prompt=`prompt` stream=`stream` echo=`echo`*
