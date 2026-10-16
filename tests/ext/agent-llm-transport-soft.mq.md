---
title: agent / llm soft transport errors
description: >-
  Regression (2026-10): unreachable OpenAI upstream must not abort the VM.
  `llm.ask` / `agent.step` / `agent.run` return structured status=error.
import agent:ext/ai/agent.mq.md
import json:lib/json.mq.md
import llm:ext/ai/llm.mq.md
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
---

# main

> sys.env_set name="OPENAI_API_KEY" value="offline-transport-soft"
> sys.env_set name="OPENAI_BASE_URL" value="http://127.0.0.1:9"

**p = > plugin.native_path name="agent"**
1. `p`
  > plugin.load path=`p`
2. *
  > print text=no-agent-plugin
  > sys.exit code=1

**model = > llm.llm**
**ask = > model.ask prompt="ping soft transport"**
**fin = > json.get value=`ask` key="finish"**
**aerr = > json.get value=`ask` key="error"**
1. `fin` == "error"
  1. `aerr`
    > print text=llm-ask-soft-ok
  2. *
    > print text=llm-ask-missing-error
    > print text=`ask`
    > sys.exit code=1
2. *
  > print text=llm-ask-hard-or-ok-unexpected
  > print text=`ask`
  > sys.exit code=1

`tools` =

| 工具 |
|------|

**ag = > agent.agent model=`model` tools=`tools` standing="transport soft probe"**
**step = > `ag`.step task="unreachable upstream probe" writeback=False max_reads=1**
**sst = > json.get value=`step` key="status"**
**serr = > json.get value=`step` key="error"**
1. `sst` == "error"
  1. `serr`
    > print text=agent-step-soft-ok
  2. *
    > print text=agent-step-missing-error
    > print text=`step`
    > sys.exit code=1
2. *
  > print text=agent-step-unexpected
  > print text=`step`
  > sys.exit code=1

**out = > `ag`.run task="unreachable upstream probe" efi=False learn=False verify=False writeback=False reuse=False near_match=False max_rounds=1**
**rst = > json.get value=`out` key="status"**
**rerr = > json.get value=`out` key="error"**
1. `rst` == "error"
  1. `rerr`
    > print text=agent-run-soft-ok
  2. *
    > print text=agent-run-missing-error
    > print text=`out`
    > sys.exit code=1
2. *
  > print text=agent-run-unexpected
  > print text=`out`
  > sys.exit code=1

*"agent-llm-transport-soft-ok"*
