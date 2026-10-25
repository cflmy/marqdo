---
markup: occlusion
title: ext/ai/llm/openai
description: >-
  OpenAI-compatible chat completions transport (HTTP + SSE).
  Prefers optional plugins/llm ABI when present; falls back to lib/net.
  Used by ext/ai/llm; authors should prefer llm.ask / llm.stream.
  Result packs are GFM tables / table.put (code-as-documentation).
import:
  net: lib/net.mq.md
  json: lib/json.mq.md
  table: lib/table.mq.md
  sys: lib/sys.mq.md
  plugin: lib/plugin.mq.md
---

## usage_from
    + `data`

Extract `{prompt_tokens,completion_tokens,total_tokens}` from an OpenAI-shaped response map.

**u = > json.get value=`data` key="usage"**
1. `u`
  **pt = > json.get value=`u` key="prompt_tokens"**
  **ct = > json.get value=`u` key="completion_tokens"**
  **tt = > json.get value=`u` key="total_tokens"**
  1. not `pt`
    **pt = 0**
  2. *
    **_ = 1**
  1. not `ct`
    **ct = 0**
  2. *
    **_ = 1**
  1. not `tt`
    **tt = pt + ct**
  2. *
    **_ = 1**
2. *
  **pt = 0**
  **ct = 0**
  **tt = 0**

`usage` =

| prompt_tokens | completion_tokens | total_tokens |
|---------------|-------------------|--------------|
| `pt` | `ct` | `tt` |

*usage*

## tool_calls_from
    + `data`

Extract OpenAI-shaped `choices[0].message.tool_calls` as a list (or empty list).

**choices = > json.get value=`data` key="choices"**
1. not `choices`
  *> table.empty_list*
2. *
  **c0 = [1](access(choices, "1"))**
  **msg = > json.get value=`c0` key="message"**
  1. not `msg`
    *> table.empty_list*
  2. *
    **tc = > json.get value=`msg` key="tool_calls"**
    1. `tc`
      *tc*
    2. *
      *> table.empty_list*

## chat_completions
    + `base_url`
    + `api_key`
    + `model`
    + `prompt`=None
    + `messages`=None
    + `selection`=None
    + `strict_selection`=False
    + `auth_mode`=None
    + `stream`=False
    + `echo`=False
    + `suffix`=/chat/completions
    + `bearer`="Bearer "

POST `/chat/completions` (or stream SSE). Prefers `plugins/llm` ABI (`llm_chat_completions`) when the native plugin is resolvable; otherwise `lib/net` HTTP/SSE. Non-stream → `{text,usage,finish,tool_calls}`; stream → `{events,finish}`. Soft transport / HTTP failures return `finish=error` packs (never abort the VM).

Pass either `messages=` (role/content list) or legacy `prompt=` (single user message). Optional `selection=` is a ModelSelectionV1 snapshot; `strict_selection=True` refuses env fallbacks inside the plugin.

**p = > plugin.native_path name="llm"**
1. `p`
  > plugin.load path=`p`
  *> llm_chat_completions base_url=`base_url` api_key=`api_key` model=`model` prompt=`prompt` messages=`messages` selection=`selection` strict_selection=`strict_selection` auth_mode=`auth_mode` stream=`stream` echo=`echo` suffix=`suffix` bearer=`bearer`*
2. *
  **url = base_url + suffix**
  **auth = bearer + api_key**
  **headers = > table.put in=None at="Authorization" value=`auth`**

  1. not `messages`
    `messages` =

    | @ | role | content |
    |---|------|---------|
    | 1 | user | `prompt` |
  2. *
    **_ = 1**

  `req` =

  | model | messages |
  |-------|----------|
  | `model` | `messages` |

  1. `stream`
    **req = > table.put in=`req` at="stream" value=True**
    **body = > json.stringify value=`req`**
    **resp = > net.http_post_sse url=`url` body=`body` headers=`headers` echo=`echo`**
    **st = > json.get value=`resp` key="status"**
    1. `st` == 200
      **evs = > json.get value=`resp` key="events"**

      `pack` =

      | finish | events |
      |--------|--------|
      | stop | `evs` |

      *pack*
    2. *
      Soft transport / HTTP failure — structured pack, never abort the VM.
      **msg = "HTTP " + `st`**
      **evs = > json.get value=`resp` key="events"**
      1. `evs`
        **e0 = > at value=`evs` index=0**
        **m2 = > json.get value=`e0` key="message"**
        1. `m2`
          **msg = `m2`**
      `pack` =

      | finish | text | error | events |
      |--------|------|-------|--------|
      | error | "" | `msg` | `evs` |

      *pack*
  2. *
    **body = > json.stringify value=`req`**
    **resp = > net.http_post url=`url` body=`body` headers=`headers`**
    **st = > json.get value=`resp` key="status"**
    1. `st` == 200
      **raw = > json.get value=`resp` key="body"**
      **data = > json.parse text=`raw`**
      **choices = > json.get value=`data` key="choices"**
      **choice0 = > at value=`choices` index=0**
      **msg = > json.get value=`choice0` key="message"**
      **text = > json.get value=`msg` key="content"**
      1. not `text`
        **text = ""**
      2. *
        **_ = 1**
      **usage = > usage_from data=`data`**
      **fr = > json.get value=`choice0` key="finish_reason"**
      1. not `fr`
        **fr = "stop"**
      2. *
        **_ = 1**
      **tc = > tool_calls_from data=`data`**

      `pack` =

      | finish | text | usage | tool_calls |
      |--------|------|-------|------------|
      | `fr` | `text` | `usage` | `tc` |

      *pack*
    2. *
      Soft transport / HTTP failure — structured pack, never abort the VM.
      **raw = > json.get value=`resp` key="body"**
      **msg = "HTTP " + `st`**
      1. `raw`
        **msg = `msg` + ": " + `raw`**
      `usage0` =

      | prompt_tokens | completion_tokens | total_tokens |
      |---------------|-------------------|--------------|
      | 0 | 0 | 0 |

      **empty_tc = > table.empty_list**

      `pack` =

      | finish | text | error | usage | tool_calls |
      |--------|------|-------|-------|------------|
      | error | "" | `msg` | `usage0` | `empty_tc` |

      *pack*
