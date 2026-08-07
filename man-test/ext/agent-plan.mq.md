---
title: ext/agent plan live (DeepSeek)
description: Full plan loop — spawn workbook, inspect; simple pong goal.
> ext/ai/llm.mq.md
> ext/ai/agent.mq.md
> lib/json.mq.md
> lib/fs.mq.md
---

# main

*`goal`=> input "你想完成什么任务？"*

> llm.load_env path=.env

*`model` = > llm.llm *
*`tools` = > json.parse text=[] *
*`助手` = > agent.agent model=`model` tools=`tools` standing=You are a Marqdo agent-development master. Prefer DONE when the workbook already satisfies the goal. *

*`out` = > `助手`.plan goal=`goal` max_rounds=3 writeback=False *

> print `out`
