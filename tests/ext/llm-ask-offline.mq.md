---
title: llm create + collect offline
description: Factory create + collect alias; no network.
import llm:ext/ai/llm.mq.md
import net:lib/net.mq.md
---

# main

**model = > llm.create**
> print text=[model](`model`)
> print text=[backend](`model`)

**fixture = "data: {\"choices\":[{\"delta\":{\"content\":\"Hi\"}}]}\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"!\"}}]}\n\ndata: [DONE]\n"**
**events = > net.openai_sse_parse text=`fixture`**
**answer = > llm.collect events=`events`**
> print text=`answer`
**alias = > llm.stream_result events=`events`**
> print text=`alias`
