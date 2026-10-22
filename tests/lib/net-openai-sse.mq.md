---
markup: occlusion
title: openai SSE parse offline
import net:lib/net.mq.md
import json:lib/json.mq.md
---

# main

**fixture = "data: {\"choices\":[{\"delta\":{\"content\":\"Hel\"}}]}\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"lo\"}}]}\n\ndata: [DONE]\n"**

**events = > net.openai_sse_parse text=`fixture`**

- [ev](遍历 `events` 为 `ev`)
  **t = > json.get value=`ev` key="type"**
  1. `t` == "delta"
    **chunk = > json.get value=`ev` key="text"**
    > print text=`chunk`
  2. `t` == "done"
    **answer = > json.get value=`ev` key="result"**
    > print text=`answer`
  3. *
    > print text=bad
