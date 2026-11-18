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

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| data | any | required | Structured input supplied to this adapter or resource |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Extract `{prompt_tokens,completion_tokens,total_tokens}` from an OpenAI-shaped response map. |

Use `data` as the inputs for this operation.

Extract `{prompt_tokens,completion_tokens,total_tokens}` from an OpenAI-shaped response map.

We retain **`u` = [Read the usage field](json.get(value=`data`, key="usage"))**.
1. `u`
  We retain **`pt` = [Read the prompt_tokens field](json.get(value=`u`, key="prompt_tokens"))**.
  We retain **`ct` = [Read the completion_tokens field](json.get(value=`u`, key="completion_tokens"))**.
  We retain **`tt` = [Read the total_tokens field](json.get(value=`u`, key="total_tokens"))**.
  1. not `pt`
    We retain **pt = 0**.
  2. *
    **_ = 1**
  1. not `ct`
    We retain **ct = 0**.
  2. *
    **_ = 1**
  1. not `tt`
    We retain **tt = pt + ct**.
  2. *
    **_ = 1**
2. *
  We retain **pt = 0**.
  We retain **ct = 0**.
  We retain **tt = 0**.

`usage` =

| prompt_tokens | completion_tokens | total_tokens |
|---------------|-------------------|--------------|
| `pt` | `ct` | `tt` |

Return *usage*.

## tool_calls_from

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| data | any | required | Structured input supplied to this adapter or resource |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Extract OpenAI-shaped `choices[0].message.tool_calls` as a list (or empty list). |

Use `data` as the inputs for this operation.

Extract OpenAI-shaped `choices[0].message.tool_calls` as a list (or empty list).

We retain **`choices` = [Read the choices field](json.get(value=`data`, key="choices"))**.
1. not `choices`
  Return *[Empty list](table.empty_list())*.
2. *
  We retain **`c0` = [Read 1 field](access(`choices`, "1"))**.
  We retain **`msg` = [Read the message field](json.get(value=`c0`, key="message"))**.
  1. not `msg`
    Return *[Empty list](table.empty_list())*.
  2. *
    We retain **`tc` = [Read the tool_calls field](json.get(value=`msg`, key="tool_calls"))**.
    1. `tc`
      Return *tc*.
    2. *
      Return *[Empty list](table.empty_list())*.

## chat_completions

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| base_url | any | required | Provider API base URL |
| api_key | any | required | Provider credential from trusted runtime configuration |
| model | any | required | Model resource or provider model identifier, as required by this operation |
| prompt | any | None | Prompt text or structured prompt expected by this adapter |
| messages | any | None | Ordered chat message records |
| selection | any | None | Explicit runtime/model selection |
| strict_selection | any | False | Whether an unresolved selection is an error |
| auth_mode | any | None | Authentication mode used by the provider adapter |
| stream | any | False | Whether to preserve streaming behavior or emit stream events |
| echo | any | False | Whether to print the documented incremental output |
| stream_kind | any | "delta" | Streaming event projection selected by the caller |
| suffix | any | "/chat/completions" | Provider endpoint suffix |
| bearer | any | "Bearer " | Bearer credential supplied by the configured provider |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | POST `/chat/completions` (or stream SSE). Prefers `plugins/llm` ABI (`llm_chat_completions`) when the native plugin is resolvable; otherwise `lib/net` HTTP/SSE. Non-stream → `{text,usage,finish,tool_calls}`; stream → `{events,finish}`. Soft transport / HTTP failures return `finish=error` packs (never abort the VM). Pass either `messages=` (role/content list) or legacy `prompt=` (single user message). Optional `selection=` is a ModelSelectionV1 snapshot; `strict_selection=True` refuses env fallbacks inside the plugin. |

Use `base_url`, `api_key`, `model`, `prompt`=None, `messages`=None, `selection`=None, `strict_selection`=False, `auth_mode`=None, `stream`=False, `echo`=False, `stream_kind`="delta", `suffix`="/chat/completions", `bearer`="Bearer " as the inputs for this operation.

POST `/chat/completions` (or stream SSE). Prefers `plugins/llm` ABI (`llm_chat_completions`) when the native plugin is resolvable; otherwise `lib/net` HTTP/SSE. Non-stream → `{text,usage,finish,tool_calls}`; stream → `{events,finish}`. Soft transport / HTTP failures return `finish=error` packs (never abort the VM).

Pass either `messages=` (role/content list) or legacy `prompt=` (single user message). Optional `selection=` is a ModelSelectionV1 snapshot; `strict_selection=True` refuses env fallbacks inside the plugin.
`stream_kind` tags live EventBus frames: `delta` (answer tokens) or `plan` (parent protocol — UI process trail, not the bubble).

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="llm"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
  Return *[Llm chat completions](llm_chat_completions(base_url=`base_url`, api_key=`api_key`, model=`model`, prompt=`prompt`, messages=`messages`, selection=`selection`, strict_selection=`strict_selection`, auth_mode=`auth_mode`, stream=`stream`, echo=`echo`, stream_kind=`stream_kind`, suffix=`suffix`, bearer=`bearer`))*.
2. *
  We retain **url = base_url + suffix**.
  We retain **auth = bearer + api_key**.
  We retain **`headers` = [Set the Authorization field in the new record](table.put(in=None, at="Authorization", value=`auth`))**.

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
    We retain **`req` = [Set the stream field in the new record](table.put(in=`req`, at="stream", value=True))**.

    `stream_opts` =

    | include_usage |
    |---------------|
    | True |

    We retain **`req` = [Set the stream_options field in the new record](table.put(in=`req`, at="stream_options", value=`stream_opts`))**.
    We retain **`body` = [Encode the structured value as JSON text](json.stringify(value=`req`))**.
    We retain **`resp` = [Http post sse](net.http_post_sse(url=`url`, body=`body`, headers=`headers`, echo=`echo`))**.
    We retain **`st` = [Read the status field](json.get(value=`resp`, key="status"))**.
    1. `st` == 200
      We retain **`evs` = [Read the events field](json.get(value=`resp`, key="events"))**.

      `pack` =

      | finish | events |
      |--------|--------|
      | stop | `evs` |

      Return *pack*.
    2. *
      Soft transport / HTTP failure — structured pack, never abort the VM.
      We retain **msg = "HTTP " + `st`**.
      We retain **`evs` = [Read the events field](json.get(value=`resp`, key="events"))**.
      1. `evs`
        We retain **`e0` = [At](at(value=`evs`, index=0))**.
        We retain **`m2` = [Read the message field](json.get(value=`e0`, key="message"))**.
        1. `m2`
          We retain **msg = `m2`**.

      `pack` =

      | finish | text | error | events |
      |--------|------|-------|--------|
      | error | "" | `msg` | `evs` |

      Return *pack*.
  2. *
    We retain **`body` = [Encode the structured value as JSON text](json.stringify(value=`req`))**.
    We retain **`resp` = [Http post](net.http_post(url=`url`, body=`body`, headers=`headers`))**.
    We retain **`st` = [Read the status field](json.get(value=`resp`, key="status"))**.
    1. `st` == 200
      We retain **`raw` = [Read the body field](json.get(value=`resp`, key="body"))**.
      We retain **`data` = [Parse the JSON transport text](json.parse(text=`raw`))**.
      We retain **`choices` = [Read the choices field](json.get(value=`data`, key="choices"))**.
      We retain **`choice0` = [At](at(value=`choices`, index=0))**.
      We retain **`msg` = [Read the message field](json.get(value=`choice0`, key="message"))**.
      We retain **`text` = [Read the content field](json.get(value=`msg`, key="content"))**.
      1. not `text`
        We retain **text = ""**.
      2. *
        **_ = 1**
      We retain **`usage` = [Usage from](usage_from(data=`data`))**.
      We retain **`fr` = [Read the finish_reason field](json.get(value=`choice0`, key="finish_reason"))**.
      1. not `fr`
        We retain **fr = "stop"**.
      2. *
        **_ = 1**
      We retain **`tc` = [Tool calls from](tool_calls_from(data=`data`))**.

      `pack` =

      | finish | text | usage | tool_calls |
      |--------|------|-------|------------|
      | `fr` | `text` | `usage` | `tc` |

      Return *pack*.
    2. *
      Soft transport / HTTP failure — structured pack, never abort the VM.
      We retain **`raw` = [Read the body field](json.get(value=`resp`, key="body"))**.
      We retain **msg = "HTTP " + `st`**.
      1. `raw`
        We retain **msg = `msg` + ": " + `raw`**.

      `usage0` =

      | prompt_tokens | completion_tokens | total_tokens |
      |---------------|-------------------|--------------|
      | 0 | 0 | 0 |

      We retain **`empty_tc` = [Empty list](table.empty_list())**.

      `pack` =

      | finish | text | error | usage | tool_calls |
      |--------|------|-------|-------|------------|
      | error | "" | `msg` | `usage0` | `empty_tc` |

      Return *pack*.
