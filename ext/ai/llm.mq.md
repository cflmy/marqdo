---
title: ext/ai/llm
description: OpenAI-compatible LLM object (English). Import ext/ai/llm.mq.md.
> lib/sys.mq.md
> lib/net.mq.md
> lib/json.mq.md
---

## load_env
    + `path`=None

Load `.env` from cwd (optional named arg `path=`). Does not override existing variables.

**> sys.load_dotenv path=`path`**

## stream_result
    + `events`

Reduce a `complete stream=True` event list to the final `done.result` text (or `error.message`). Unknown event types abort.

*answer = "" *

- [ev](events)
  *t = ev[^type] *
  1. `t` == "done"
    *answer = ev[^result] *
  2. `t` == "error"
    *answer = ev[^message] *
  3. `t` == "delta"
  4. *
    > print text=ext/ai/llm: unexpected SSE event type
    > print text=`t`
    > sys.exit code=1

**answer**

# llm

Construct an LLM handle from `OPENAI_*` / `MARQDO_LLM_*`.

*api_key = > sys.env_get name=OPENAI_API_KEY *
1. not `api_key`
  *api_key = > sys.env_get name=MARQDO_LLM_API_KEY *

1. not `api_key`
  > print text=ext/ai/llm: set OPENAI_API_KEY or MARQDO_LLM_API_KEY
  > sys.exit code=1

*base_url = > sys.env_get name=OPENAI_BASE_URL *
1. not `base_url`
  *base_url = > sys.env_get name=MARQDO_LLM_BASE_URL *
1. not `base_url`
  *base_url = "https://api.openai.com/v1" *

*model = > sys.env_get name=OPENAI_MODEL *
1. not `model`
  *model = > sys.env_get name=MARQDO_LLM_MODEL *
1. not `model`
  *model = "gpt-4o-mini" *

`h` =

| api_key | base_url | model | suffix | bearer |
|---------|----------|-------|--------|--------|
| `api_key` | `base_url` | `model` | /chat/completions | "Bearer " |

**h**

## complete
    + `prompt`
    + `stream`=False
    + `echo`=False

Chat completion using `self` handle fields.

Wire body is `{model, messages:[{role,content}], stream?}` — message row is a table; `stream` flag still needs a map set when true. Headers use one `json.set` because a 1-col table is a List, not a Map.

*url = self[^base_url] + self[^suffix] *
*auth = self[^bearer] + self[^api_key] *
*headers = > json.set map=None key=Authorization value=`auth` *

`msg` =

| role | content |
|------|---------|
| user | `prompt` |

*messages = > json.append list=None item=`msg` *

`req` =

| model | messages |
|-------|----------|
| self[^model] | `messages` |

1. `stream`
  *req = > json.set map=`req` key=stream value=True *
  *body = > json.stringify value=`req` *
  *resp = > net.http_post_sse url=`url` body=`body` headers=`headers` echo=`echo` *
  1. resp[^status] == 200
    **resp[^events]**
  2. *
    > print text=ext/ai/llm: HTTP error (stream)
    > print text=resp[^status]
    > sys.exit code=1
2. *
  *body = > json.stringify value=`req` *
  *resp = > net.http_post url=`url` body=`body` headers=`headers` *
  1. resp[^status] == 200
    *data = > json.parse text=resp[^body] *
    **data[^choices][^1][^message][^content]**
  2. *
    > print text=ext/ai/llm: HTTP error
    > print text=resp[^status]
    > print text=resp[^body]
    > sys.exit code=1

---

## chat
    + `prompt`
    + `stream`=False
    + `echo`=False

Alias for `complete`.

**> `self`.complete prompt=`prompt` stream=`stream` echo=`echo`**
