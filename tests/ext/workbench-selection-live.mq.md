---
markup: occlusion
title: workbench M01 live selection (USTC-compatible)
description: Explicit ModelSelection via llm.create; no llm.llm. Credentials from process env only.
import:
  llm: ext/ai/llm.mq.md
  sys: lib/sys.mq.md
  json: lib/json.mq.md
---

# main

**base = > sys.env_get name="OPENAI_BASE_URL"**
**key = > sys.env_get name="OPENAI_API_KEY"**
**mdl = > sys.env_get name="OPENAI_MODEL"**
1. not `base`
  > print text=missing-OPENAI_BASE_URL
  > sys.exit code=2
2. *
  **_ = 1**
1. not `key`
  > print text=missing-OPENAI_API_KEY
  > sys.exit code=2
2. *
  **_ = 1**
1. not `mdl`
  **mdl = "qwen3.8-chat"**
2. *
  **_ = 1**

**model = > llm.create model=`mdl` base_url=`base` api_key=`key` backend="openai-compatible" name=workbench-live**

**q = "Reply with exactly one English word pong"**
**reply = > model.ask prompt=`q`**
**text = > json.get value=`reply` key="text"**
**usage = > json.get value=`reply` key="usage"**
**pt = > json.get value=`usage` key="prompt_tokens"**

> print text=`text`
1. `pt`
  > print text=usage-present
2. *
  > print text=usage-unknown
