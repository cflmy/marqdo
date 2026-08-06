---
title: ext llm DeepSeek complete (live)
description: Live OpenAI-compatible chat via DeepSeek. Needs tests/ext/.env (gitignored). Not in gold suite.
> ext/ai/llm.mq.md
---

# main

> load_env

*`model` = > llm *

*`reply` = > `model`.complete prompt=Reply with exactly one English word: pong *

> print text=`reply`
