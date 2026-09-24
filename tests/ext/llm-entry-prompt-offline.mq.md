---
type: prompt
title: entry prompt offline
import llm:ext/ai/llm.mq.md
import sys:lib/sys.mq.md
---

Say hello to Marqdo.

# main

**raw = > sys.module_source**
**p = > llm.prompt_body src=`raw`**
> print text=`p`
