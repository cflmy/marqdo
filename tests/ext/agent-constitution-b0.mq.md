---
title: agent constitution markers (Wave B1 offline)
description: Assert call_site / source / protocol order in build_step_context.
import llm:ext/ai/llm.mq.md
import agent:ext/ai/agent.mq.md
import json:lib/json.mq.md
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
---

## probe_tool

**"probe-ok"**

# main

*p = > plugin.native_path name="agent"*
1. `p`
  > plugin.load path=`p`
2. *
  > print text=no-agent-plugin
  > sys.exit code=1

> sys.env_set name="OPENAI_API_KEY" value="offline-constitution-dummy"
*model = > llm.llm*

`工具表` =

| 工具 |
|------|
| probe_tool |

*助手 = > agent.agent model=`model` tools=`工具表` standing="constitution probe"*
*ctx = > agent.build_step_context agent=`助手` task="constitution-probe-task"*

*a = > split value=`ctx` sep="--- call site ---"*
*b = > split value=`ctx` sep="--- source (.mq.md) ---"*
*c = > split value=`ctx` sep="--- how to act ---"*
*d = > split value=`ctx` sep="Code is documentation"*
*na = > len value=`a`*
*nb = > len value=`b`*
*nc = > len value=`c`*
*nd = > len value=`d`*
1. `na` > 1
  1. `nb` > 1
    1. `nc` > 1
      1. `nd` > 1
        > print text=constitution-ok
      2. *
        > print text=motto-missing
        > sys.exit code=1
    2. *
      > print text=act-missing
      > sys.exit code=1
  2. *
    > print text=source-missing
    > sys.exit code=1
2. *
  > print text=call-site-missing
  > sys.exit code=1

*site = > agent_call_site*
*path = > json.get value=`site` key="path"*
1. `path`
  > print text=site-path-ok
2. *
  > print text=site-path-missing
  > sys.exit code=1
