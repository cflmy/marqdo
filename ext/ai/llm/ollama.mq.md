---
title: ext/ai/llm/ollama
description: >-
  Ollama backend defaults. Transport reuses openai-compatible /v1 chat completions.
import openai:ext/ai/llm/openai.mq.md
import sys:lib/sys.mq.md
---

## resolve
    + `model`=None
    + `base_url`=None
    + `api_key`=None

Resolve Ollama handle fields: default host `http://127.0.0.1:11434/v1`, key `ollama`, model `llama3.2`.

1. `api_key`
  **key = api_key**
2. *
  **key = "ollama"**

1. `base_url`
  **url = base_url**
2. *
  **url = > sys.env_get name="OLLAMA_HOST"**
  1. not `url`
    **url = "http://127.0.0.1:11434/v1"**
  2. *
    **_ = 1**

1. `model`
  **mdl = model**
2. *
  **mdl = > sys.env_get name="MARQDO_LLM_MODEL"**
  1. not `mdl`
    **mdl = "llama3.2"**
  2. *
    **_ = 1**

`cfg` =

| backend | api_key | base_url | model |
|---------|---------|----------|-------|
| ollama | `key` | `url` | `mdl` |

*cfg*

## chat_completions
    + `base_url`
    + `api_key`
    + `model`
    + `prompt`
    + `stream`=False
    + `echo`=False

*> openai.chat_completions base_url=`base_url` api_key=`api_key` model=`model` prompt=`prompt` stream=`stream` echo=`echo`*
