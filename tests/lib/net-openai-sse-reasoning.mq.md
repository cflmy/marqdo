---
title: openai SSE reasoning_content offline
description: DeepSeek-style thinking streams as type=reasoning; answer stays delta/done.result.
import net:lib/net.mq.md
import json:lib/json.mq.md
---

# main

*fixture = "data: {\"choices\":[{\"delta\":{\"reasoning_content\":\"think\"}}]}\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"Hi\"}}]}\n\ndata: [DONE]\n"*

*events = > net.openai_sse_parse text=`fixture`*
*n = > len value=`events`*
> print text=`n`

*e0 = > at value=`events` index=0*
*t0 = > json.get value=`e0` key=type*
*x0 = > json.get value=`e0` key=text*
> print text=`t0`
> print text=`x0`

*e1 = > at value=`events` index=1*
*t1 = > json.get value=`e1` key=type*
*x1 = > json.get value=`e1` key=text*
> print text=`t1`
> print text=`x1`

*e2 = > at value=`events` index=2*
*t2 = > json.get value=`e2` key=type*
*r2 = > json.get value=`e2` key=result*
> print text=`t2`
> print text=`r2`
