---
title: dump_step_context (Wave B1 offline)
description: Assert dump_step_context exposes call_site + prompt without LLM.
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

> sys.env_set name="OPENAI_API_KEY" value="offline-dump-dummy"
*model = > llm.llm*

`工具表` =
| 工具 |
|------|
| probe_tool |

*助手 = > agent.agent model=`model` tools=`工具表` standing="dump probe"*
*dump = > agent.dump_step_context agent=`助手` task="dump-probe-task"*

*site = > json.get value=`dump` key="call_site"*
*path = > json.get value=`site` key="path"*
*chars = > json.get value=`dump` key="prompt_chars"*
*prompt = > json.get value=`dump` key="prompt"*

1. `path`
  1. `chars` > 100
    *parts = > split value=`prompt` sep="--- call site ---"*
    *n = > len value=`parts`*
    1. `n` > 1
      > print text=dump-context-ok
    2. *
      > print text=dump-prompt-missing-site
      > sys.exit code=1
  2. *
    > print text=dump-chars-too-small
    > sys.exit code=1
2. *
  > print text=dump-site-path-missing
  > sys.exit code=1
