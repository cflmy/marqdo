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

流式开在 plan 上：非命中先 plan:decompose（父分解增量），再 plan:await（子 quiet），然后父修订轮。

*`out` = > `助手`.plan goal=`goal` max_rounds=3 writeback=False stream=True echo=True force=True *

*`result` = > json.get value=`out` key=result *
> print text=`result`
