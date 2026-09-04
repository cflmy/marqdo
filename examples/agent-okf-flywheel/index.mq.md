---
title: agent-okf-flywheel (Wave B0)
description: Promote solidified workbook; second plan is cache=hit — document as knowledge base (OKF).
import llm:ext/ai/llm.mq.md
import agent:ext/ai/agent.mq.md
import fs:lib/fs.mq.md
import json:lib/json.mq.md
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
---

# main

OKF flywheel offline: promote → plan same goal → `cache=hit` (no live LLM required for the hit path).

*p = > plugin.native_path name="agent"*
1. `p`
  > plugin.load path=`p`
2. *
  > print text=no-agent-plugin
  > sys.exit code=1

> sys.env_set name="OPENAI_API_KEY" value="offline-okf-dummy"
*model = > llm.llm*
*tools = > json.parse text=[]*
*助手 = > agent.agent model=`model` tools=`tools` standing="offline okf flywheel"*

*kb = ".marqdo/agent-kb-example-flywheel"*
*wb = ".marqdo/agent-runs/example-pong-solid.mq.md"*
*goal = "Reply with exactly the word pong and nothing else."*
*body = "---\ntitle: okf pong\n---\n\n# main\n\n*msg = \"pong\"*\n\n**msg**\n"*

1. > fs.exists path=`kb`
  > fs.remove path=`kb`
2. *
  ---

> fs.write_text path=`wb` text=`body`

*prom = > agent_kb_promote kb_dir=`kb` goal=`goal` workbook=`wb`*
*okp = > json.get value=`prom` key="promoted"*
1. `okp`
  > print text=promoted
2. *
  > print text=promote-fail
  > sys.exit code=1

*out = > `助手`.plan goal=`goal` max_rounds=2 writeback=False kb_dir=`kb` explore_n=0 promote=False*
*cache = > json.get value=`out` key="cache"*
> print text=`cache`
*match = > json.get value=`out` key="match"*
> print text=`match`
*val = > json.get value=`out` key="result"*
> print text=`val`

1. `cache` == "hit"
  > print text=flywheel-ok
2. *
  > print text=flywheel-miss
  > sys.exit code=1
