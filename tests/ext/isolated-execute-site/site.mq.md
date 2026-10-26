---
markup: occlusion
title: isolated execute site libs
description: Chat (execute) + models (short) for MR-01 gold sample.
import:
  llm: ext/ai/llm.mq.md
  sys: lib/sys.mq.md
  json: lib/json.mq.md
  table: lib/table.mq.md
---

# chat
    + `model`="mr01"
    + `messages`=None
    + `stream`=True
    + `base_url`=""
    + `prompt`=""
    + `authorization`=""
    + `api_key`=""
    + `payload`=None

OpenAI-compatible chat. Prefer request `base_url` so dual MR-01 streams can hit distinct fake upstreams.

**base = base_url**
1. not `base`
  1. `payload`
    **base = > json.get value=`payload` key="base_url"**
  2. *
    **base = > sys.env_get name="OPENAI_BASE_URL"**
2. *
  **_ = 1**
1. not `base`
  **base = "http://127.0.0.1:9/v1"**
2. *
  **_ = 1**

**key = api_key**
1. not `key`
  **key = > sys.env_get name="OPENAI_API_KEY"**
2. *
  **_ = 1**
1. not `key`
  **key = "mr01-key"**
2. *
  **_ = 1**

**mdl = model**
1. not `mdl`
  **mdl = "mr01"**
2. *
  **_ = 1**

**m = > llm.create model=`mdl` base_url=`base` api_key=`key` name="mr01" backend="openai-compatible"**
1. `messages`
  *> `m`.complete_result messages=`messages` stream=True echo=False*
2. `prompt`
  *> `m`.complete_result prompt=`prompt` stream=True echo=False*
3. *
  *> `m`.complete_result prompt="ping" stream=True echo=False*

# models
    + `payload`=None

Short-class read: list models without holding an execute slot.

`out` =

| object | ok |
|--------|-----|
| list | True |

*out*
