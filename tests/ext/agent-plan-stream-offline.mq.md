---
title: agent plan stream helpers + confirm (offline)
description: plan_append_*/plan_finish_stream; confirm=True with stream attaches done event.
> ext/ai/llm.mq.md
> ext/ai/agent.mq.md
> lib/json.mq.md
> lib/fs.mq.md
---

# main

*`events` = > json.parse text=[] *
*`events` = > agent.plan_append_round events=`events` round=1 workbook=wb.mq.md exit_code=0 stream=True *
*`from` = > json.parse text=[{"type":"delta","text":"Hi"},{"type":"done","result":"Hi"}] *
*`events` = > agent.plan_merge_deltas events=`events` from=`from` stream=True *
*`events` = > agent.plan_append_decision events=`events` decision=DONE stream=True summary=ok *
*`out` = > json.parse text={"status":"ok"} *
*`out` = > agent.plan_finish_stream out=`out` events=`events` stream=True result=child *

*`evs` = > json.get value=`out` key=events *
*`n` = > len value=`evs` *
> print text=`n`

*`e0` = > at value=`evs` index=0 *
*`t0` = > json.get value=`e0` key=type *
> print text=`t0`

*`e1` = > at value=`evs` index=1 *
*`t1` = > json.get value=`e1` key=type *
*`x1` = > json.get value=`e1` key=text *
> print text=`t1`
> print text=`x1`

*`e2` = > at value=`evs` index=2 *
*`t2` = > json.get value=`e2` key=type *
> print text=`t2`

*`e3` = > at value=`evs` index=3 *
*`t3` = > json.get value=`e3` key=type *
*`r3` = > json.get value=`e3` key=result *
> print text=`t3`
> print text=`r3`

> llm.load_env path=.env

*`model` = > llm.llm *
*`tools` = > json.parse text=[] *
*`助手` = > agent.agent model=`model` tools=`tools` standing=offline plan stream *

*`pout` = > `助手`.plan goal=say hi confirm=True workbook_dir=".marqdo/agent-runs" writeback=False stream=True *

*`st` = > json.get value=`pout` key=status *
> print text=`st`

*`pevs` = > json.get value=`pout` key=events *
*`pn` = > len value=`pevs` *
> print text=`pn`

*`plast` = > at value=`pevs` index=0 *
*`pt` = > json.get value=`plast` key=type *
> print text=`pt`
