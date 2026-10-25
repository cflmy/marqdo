---
markup: occlusion
title: workbench F02 agent selection (mock upstream)
description: >-
  Explicit llm.create selection must drive agent.step; process env default model
  must not win. Upstream URL/key/model come from env set by the Rust harness.
import:
  llm: ext/ai/llm.mq.md
  agent: ext/ai/agent.mq.md
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
  > print text=missing-OPENAI_MODEL
  > sys.exit code=2
2. *
  **_ = 1**

Poison ambient default — selection on the handle must still win at the plugin.
> sys.env_set name="OPENAI_MODEL" value="env-poison-model"

**model = > llm.create model=`mdl` base_url=`base` api_key=`key` backend="openai-compatible" name=f02-agent**

`tools` =

| 工具 |
|------|

**ag = > agent.agent model=`model` tools=`tools` standing="F02 selection probe; answer with one word only"**
**step = > `ag`.step task="Reply with exactly one English word pong" writeback=False max_reads=1**
**st = > json.get value=`step` key="status"**
**ans = > json.get value=`step` key="result"**
1. not `ans`
  **ans = > json.get value=`step` key="answer"**
2. *
  **_ = 1**
1. not `ans`
  **ans = > json.get value=`step` key="text"**
2. *
  **_ = 1**

> print text=`st`
> print text=`ans`

**dir = ".marqdo/workbench-f02-resume"**
**saved = > agent.resume_save id="f02-sel" goal="pong" status="running" round=1 resume_dir=`dir`**
**ok = > json.get value=`saved` key="ok"**
1. `ok`
  > print text=resume-save-ok
2. *
  > print text=resume-save-fail
  > print text=`saved`
  > sys.exit code=1

**loaded = > agent.resume id="f02-sel" resume_dir=`dir`**
**found = > json.get value=`loaded` key="found"**
1. `found`
  > print text=resume-load-ok
2. *
  > print text=resume-load-fail
  > print text=`loaded`
  > sys.exit code=1

> agent.resume_clear id="f02-sel" resume_dir=`dir`
> print text=f02-agent-selection-ok
