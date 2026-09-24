---
title: ext/ai/llm/openai
description: >-
  OpenAI-compatible chat completions transport (HTTP + SSE).
  Used by ext/ai/llm; authors should prefer llm.ask / llm.stream.
import net:lib/net.mq.md
import json:lib/json.mq.md
import table:lib/table.mq.md
import sys:lib/sys.mq.md
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
  **empty = > json.parse text=[]**
  *empty*
2. *
  **c0 = [1](choices)**
  **msg = > json.get value=`c0` key="message"**
  1. not `msg`
    **empty = > json.parse text=[]**
    *empty*
  2. *
    **tc = > json.get value=`msg` key="tool_calls"**
    1. `tc`
      *tc*
    2. *
      **empty = > json.parse text=[]**
      *empty*

## chat_completions
    + `base_url`
    + `api_key`
    + `model`
    + `prompt`
    + `stream`=False
    + `echo`=False
    + `suffix`=/chat/completions
    + `bearer`="Bearer "

POST `/chat/completions` (or stream SSE). Non-stream → `{text,usage,finish,tool_calls}`; stream → `{events,finish}`.

**url = base_url + suffix**
**auth = bearer + api_key**
**headers = > table.put in=None at="Authorization" value=`auth`**

`messages` =

| @ | role | content |
|---|------|---------|
| 1 | user | `prompt` |

`req` =

| model | messages |
|-------|----------|
| `model` | `messages` |

1. `stream`
  **req = > table.put in=`req` at="stream" value=True**
  **body = > json.stringify value=`req`**
  **resp = > net.http_post_sse url=`url` body=`body` headers=`headers` echo=`echo`**
  1. [status](resp) == 200
    **pack = > json.parse text={"finish":"stop"}**
    **pack = > json.set map=`pack` key="events" value=[events](resp)**
    *pack*
  2. *
    > print text=ext/ai/llm/openai: HTTP error (stream)
    > print text=[status](resp)
    > sys.exit code=1
2. *
  **body = > json.stringify value=`req`**
  **resp = > net.http_post url=`url` body=`body` headers=`headers`**
  1. [status](resp) == 200
    **data = > json.parse text=[body](resp)**
    **msg = [message]([1]([choices](data)))**
    **text = > json.get value=`msg` key="content"**
    1. not `text`
      **text = ""**
    2. *
      **_ = 1**
    **usage = > usage_from data=`data`**
    **fr = > json.get value=[1]([choices](data)) key="finish_reason"**
    1. not `fr`
      **fr = "stop"**
    2. *
      **_ = 1**
    **tc = > tool_calls_from data=`data`**
    **pack = > json.parse text={"finish":"stop"}**
    **pack = > json.set map=`pack` key="text" value=`text`**
    **pack = > json.set map=`pack` key="usage" value=`usage`**
    **pack = > json.set map=`pack` key="finish" value=`fr`**
    **pack = > json.set map=`pack` key="tool_calls" value=`tc`**
    *pack*
  2. *
    > print text=ext/ai/llm/openai: HTTP error
    > print text=[status](resp)
    > print text=[body](resp)
    > sys.exit code=1
