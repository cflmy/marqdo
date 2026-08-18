---
title: ext llm DeepSeek complete
description: OpenAI-compatible chat via DeepSeek. Credentials from tests/ext/.env.
import llm:ext/ai/llm.mq.md
---

# main

> llm.load_env path=.env

*model = > llm.llm*

*reply = > `model`.complete prompt="Reply with exactly one English word: pong"*

> print text=`reply`
