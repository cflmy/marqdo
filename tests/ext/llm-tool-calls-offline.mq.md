---
title: llm tool_calls offline
description: Parse OpenAI-shaped tool_calls into a list; no network.
import openai:ext/ai/llm/openai.mq.md
import json:lib/json.mq.md
---

# main

**raw = "{\"choices\":[{\"finish_reason\":\"tool_calls\",\"message\":{\"role\":\"assistant\",\"content\":null,\"tool_calls\":[{\"id\":\"call_1\",\"type\":\"function\",\"function\":{\"name\":\"lookup\",\"arguments\":\"{\\\"q\\\":\\\"marqdo\\\"}\"}}]}}]}"**
**data = > json.parse text=`raw`**
**tc = > openai.tool_calls_from data=`data`**
**n = > len value=`tc`**
> print text=`n`
**first = [1](tc)**
> print text=[id](`first`)
> print text=[type](`first`)
**fn = [function](first)**
> print text=[name](`fn`)

**empty_data = > json.parse text={"choices":[{"message":{"role":"assistant","content":"hi"}}]}**
**empty = > openai.tool_calls_from data=`empty_data`**
**en = > len value=`empty`**
> print text=`en`
