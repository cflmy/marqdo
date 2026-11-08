---
markup: occlusion
title: ext/ai/llm/ollama
description: >-
  Ollama backend defaults. Transport reuses openai-compatible /v1 chat completions.
import:
  openai: ext/ai/llm/openai.mq.md
  sys: lib/sys.mq.md
---

## resolve

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| model | any | None | Model resource or provider model identifier, as required by this operation |
| base_url | any | None | Provider API base URL |
| api_key | any | None | Provider credential from trusted runtime configuration |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Resolve Ollama handle fields: default host `http://127.0.0.1:11434/v1`, key `ollama`, model `llama3.2`. |

Use `model`=None, `base_url`=None, `api_key`=None as the inputs for this operation.

Resolve Ollama handle fields: default host `http://127.0.0.1:11434/v1`, key `ollama`, model `llama3.2`.

1. `api_key`
  We retain **key = api_key**.
2. *
  We retain **key = "ollama"**.

1. `base_url`
  We retain **url = base_url**.
2. *
  We retain **`url` = [Env get](sys.env_get(name="OLLAMA_HOST"))**.
  1. not `url`
    We retain **url = "http://127.0.0.1:11434/v1"**.
  2. *
    **_ = 1**

1. `model`
  We retain **mdl = model**.
2. *
  We retain **`mdl` = [Env get](sys.env_get(name="MARQDO_LLM_MODEL"))**.
  1. not `mdl`
    We retain **mdl = "llama3.2"**.
  2. *
    **_ = 1**

`cfg` =

| backend | api_key | base_url | model |
|---------|---------|----------|-------|
| ollama | `key` | `url` | `mdl` |

Return *cfg*.

## chat_completions

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| base_url | any | required | Provider API base URL |
| api_key | any | required | Provider credential from trusted runtime configuration |
| model | any | required | Model resource or provider model identifier, as required by this operation |
| prompt | any | required | Prompt text or structured prompt expected by this adapter |
| stream | any | False | Whether to preserve streaming behavior or emit stream events |
| echo | any | False | Whether to print the documented incremental output |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Send the prepared chat request through this provider adapter and retain its usage, stream and failure conventions. |

Use `base_url`, `api_key`, `model`, `prompt`, `stream`=False, `echo`=False as the inputs for this operation.

Send the prepared chat request through this provider adapter and retain its usage, stream and failure conventions.

Return *[Chat completions](openai.chat_completions(base_url=`base_url`, api_key=`api_key`, model=`model`, prompt=`prompt`, stream=`stream`, echo=`echo`))*.
