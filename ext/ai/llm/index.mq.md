---
markup: occlusion
title: Provider adapters — requests, configuration and usage
import:
  ollama: ext/ai/llm/ollama.mq.md
  openai: ext/ai/llm/openai.mq.md
---

This directory owns provider adaptation. [OpenAI-compatible requests](openai.mq.md) encode chat
messages, decode replies and usage, and retain streaming/error conventions. [Ollama](ollama.mq.md)
selects its documented local defaults while reusing compatible transport. The parent [model API](../llm.mq.md)
creates author model resources; [the AI root](../index.mq.md) explains agent and trial responsibilities.

This offline usage example inspects response data only. It does not read credentials, load a
plugin or contact the provider:

```sh
marqdo call ext/ai/llm/index.mq.md usage 'data={"usage":{"prompt_tokens":2,"completion_tokens":3,"total_tokens":5}}' --json
```

For local configuration, resolve_local reads the documented environment fallbacks and therefore
needs environment.read. Sending chat_completions is a real HTTP effect and requires an explicitly
configured endpoint, timeout and credential handling. Do not print a live configuration record
containing secrets merely to prove that an adapter exists.

When adding a provider, preserve semantic message/event/result shapes at the public model boundary.
Keep provider-specific headers and endpoint paths in the adapter, document supported response
variants and test them against local fixtures before live traffic. A usage fixture does not verify
provider availability, streaming latency or authentication.

## usage

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| data | map | required | Already received OpenAI-compatible response record |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Public prompt_tokens/completion_tokens/total_tokens usage projection |

The `data` record is an observation rather than a request body. Missing counters follow the
public adapter convention; the example above contains two input tokens, three output tokens and
five total tokens. We return *[Read actual usage counters from the supplied response](openai.usage_from(data=`data`))*
without claiming to have measured a new model invocation.

## resolve_local

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| model | any | None | Explicit local model override; None uses the adapter fallback |
| base_url | any | None | Explicit local API base URL; None uses the documented environment/default |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Actual Ollama configuration selected by the existing resolver |

The caller may supply `model`=None and `base_url`=None to use the shared local configuration
rules. Resolving the configuration does not launch a service or ask the model for a response.
We return *[Resolve local model fields with the existing adapter](ollama.resolve(model=`model`, base_url=`base_url`))*.
Treat the resulting configuration as runtime data and only execute a request after granting its effects.
