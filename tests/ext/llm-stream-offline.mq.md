---
title: llm stream_result offline
description: No network — SSE fixture via net.openai_sse_parse + llm.stream_result
import llm:ext/ai/llm.mq.md
import net:lib/net.mq.md
---

# main

*fixture = "data: {\"choices\":[{\"delta\":{\"content\":\"Hi\"}}]}\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"!\"}}]}\n\ndata: [DONE]\n"*

*events = > net.openai_sse_parse text=`fixture`*
*answer = > llm.stream_result events=`events`*
> print text=`answer`
