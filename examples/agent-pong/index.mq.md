---
title: agent-pong (Wave B0)
description: >-
  Constitution demo — call_site + source in step context; optional live CALL via .env.
  Code is documentation; tools are ## in this same file.
import llm:ext/ai/llm.mq.md
import agent:ext/ai/agent.mq.md
import json:lib/json.mq.md
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
import time:lib/time.mq.md
---

## 获取时间

Return today's date string (tool visible in source to the model).

*u = > time.now_unix*
**> time.format unix=`u` pattern="%Y-%m-%d"**

# main

*p = > plugin.native_path name="agent"*
1. `p`
  > plugin.load path=`p`
2. *
  > print text=no-agent-plugin
  > sys.exit code=1

> llm.load_env path="../../.env"

`工具表` =

| 工具 |
|------|
| 获取时间 |

*model = > llm.llm*
*助手 = > agent.agent model=`model` tools=`工具表` standing="You are a Marqdo agent. Prefer CALL:获取时间 when the task needs today's date."*

Assert constitution markers in the assembled prompt (call site before how-to-act):

*ctx = > agent.build_step_context agent=`助手` task="What is today's date? Use the tool in this runbook."*

*parts = > split value=`ctx` sep="--- call site ---"*
*n = > len value=`parts`*
1. `n` > 1
  > print text=call-site-ok
2. *
  > print text=call-site-missing
  > sys.exit code=1

*parts2 = > split value=`ctx` sep="--- source (.mq.md) ---"*
*n2 = > len value=`parts2`*
1. `n2` > 1
  > print text=source-ok
2. *
  > print text=source-missing
  > sys.exit code=1

*parts3 = > split value=`ctx` sep="获取时间"*
*n3 = > len value=`parts3`*
1. `n3` > 1
  > print text=tool-in-source-ok
2. *
  > print text=tool-in-source-missing
  > sys.exit code=1

*site = > agent_call_site*
*fn = > json.get value=`site` key="function"*
> print text=`fn`

Optional live step — set env AGENT_LIVE=1 and put keys in repo-root `.env`:

*live = > sys.env_get name="AGENT_LIVE"*
*key = > sys.env_get name="OPENAI_API_KEY"*
1. `live` == "1"
  1. `key`
    *out = > `助手`.step task="请调用获取时间工具，然后只回复日期字符串。" writeback=False*
    *status = > json.get value=`out` key="status"*
    > print text=`status`
    *tool = > json.get value=`out` key="tool"*
    > print text=`tool`
    *result = > json.get value=`out` key="result"*
    > print text=`result`
  2. *
    > print text=live-skipped-no-key
2. *
  > print text=live-skipped
