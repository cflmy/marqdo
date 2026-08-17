---
title: ext/agent plan live (DeepSeek)
description: Full plan loop — spawn workbook, inspect, secondary plan; simple pong goal.
import llm:ext/ai/llm.mq.md
import agent:ext/ai/agent.mq.md
import json:lib/json.mq.md
import fs:lib/fs.mq.md
---

# main

> llm.load_env path=.env

*model = > llm.llm*
*tools = > json.parse text=[]*
*助手 = > agent.agent model=`model` tools=`tools` standing=You are a Marqdo agent-development master. Prefer DONE when the workbook already satisfies the goal.*

*out = > `助手`.plan goal=Reply with exactly the word pong and nothing else. max_rounds=3 writeback=False*

*st = > json.get value=`out` key=status*
> print text=`st`

*sum = > json.get value=`out` key=summary*
1. `sum`
  > print text=`sum`
2. *
  > print text=no-summary

*cache = > json.get value=`out` key=cache*
> print text=`cache`

*path = > json.get value=`out` key=workbook*
*ex = > fs.exists path=`path`*
1. `ex`
  > print text=workbook-ok
2. *
  > print text=workbook-missing

*out2 = > `助手`.plan goal=Reply with exactly the word pong and nothing else. max_rounds=3 writeback=False*
*st2 = > json.get value=`out2` key=status*
> print text=`st2`
*cache2 = > json.get value=`out2` key=cache*
> print text=`cache2`
