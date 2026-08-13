---
title: ext llm DeepSeek stream live
description: Live SSE complete with echo; credentials from tests/ext/.env.
import llm:ext/ai/llm.mq.md
import json:lib/json.mq.md
---

# main

> llm.load_env path=.env

*`model` = > llm.llm *

*`events` = > `model`.complete prompt=Reply with exactly one English word: pong stream=True echo=True *
*`n` = > len value=`events` *
> print text=`n`

*`answer` = > llm.stream_result events=`events` *
> print text=`answer`
