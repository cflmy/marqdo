---
title: agent plan confirm (offline)
description: Create workbook skeleton with confirm=True; no LLM spawn loop.
import llm:ext/ai/llm.mq.md
import agent:ext/ai/agent.mq.md
import json:lib/json.mq.md
import fs:lib/fs.mq.md
---

# main

> llm.load_env path=.env

*model = > llm.llm*
*tools = > json.parse text=[]*
*助手 = > agent.agent model=`model` tools=`tools` standing=offline plan confirm*

*out = > `助手`.plan goal=say hi confirm=True workbook_dir=".marqdo/agent-runs" writeback=False*

*st = > json.get value=`out` key=status*
> print text=`st`

*path = > json.get value=`out` key=workbook*
*ex = > fs.exists path=`path`*
1. `ex`
  > print text=workbook-ok
2. *
  > print text=workbook-missing

*src = > fs.read_text path=`path`*
*parts = > split value=`src` sep=agent.agent*
*n = > len value=`parts`*
1. `n` > 1
  > print text=skeleton-ok
2. *
  > print text=skeleton-bad
