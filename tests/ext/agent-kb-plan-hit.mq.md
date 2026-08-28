---
title: OKF plan llm_free hit + list_tasks curation (A1 offline)
description: Promote solidified workbook; plan same goal takes cache=hit; near→soft-hit; list_tasks exposes status/llm_free.
import llm:ext/ai/llm.mq.md
import agent:ext/ai/agent.mq.md
import fs:lib/fs.mq.md
import json:lib/json.mq.md
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
---

# main

*p = > plugin.native_path name="agent"*
1. `p`
  > plugin.load path=`p`
2. *
  > print text=no-agent-plugin
  > sys.exit code=1

> sys.env_set name="OPENAI_API_KEY" value="offline-a1-dummy"
*model = > llm.llm*
*tools = > json.parse text=[]*
*助手 = > agent.agent model=`model` tools=`tools` standing="offline plan-hit"*

*paths = > json.parse text={"kb":".marqdo/agent-kb-a1-plan-hit","wb":".marqdo/agent-runs/a1-pong-solid.mq.md","goal":"Reply with exactly the word pong and nothing else.","near":"Reply with exactly the word pong.","body":"---\ntitle: a1 pong\n---\n\n# main\n\n*msg = \"pong\"*\n\n**msg**\n"}*
*kb = > json.get value=`paths` key="kb"*
*wb = > json.get value=`paths` key="wb"*
*goal = > json.get value=`paths` key="goal"*
*near_g = > json.get value=`paths` key="near"*
*body = > json.get value=`paths` key="body"*

> fs.write_text path=`wb` text=`body`

*prom = > agent_kb_promote kb_dir=`kb` goal=`goal` workbook=`wb`*
*okp = > json.get value=`prom` key="promoted"*
1. `okp`
  > print text=promoted
2. *
  > print text=promote-fail
  > sys.exit code=1

*listed = > agent_kb_list_tasks kb_dir=`kb`*
*tc = > json.get value=`listed` key="count"*
1. `tc` > 0
  > print text=list-ok
2. *
  > print text=list-bad
  > sys.exit code=1

*tasks = > json.get value=`listed` key="tasks"*
*t0 = > at value=`tasks` index=0*
*lf = > json.get value=`t0` key="llm_free"*
1. `lf`
  > print text=list-free
2. *
  > print text=list-not-free
*st = > json.get value=`t0` key="status"*
> print text=`st`
*desc = > json.get value=`t0` key="description"*
1. `desc`
  > print text=list-desc
2. *
  > print text=list-no-desc

*out = > `助手`.plan goal=`goal` max_rounds=2 writeback=False kb_dir=`kb` explore_n=0 promote=False*
*cache = > json.get value=`out` key="cache"*
> print text=`cache`
*mk = > json.get value=`out` key="match"*
> print text=`mk`
*val = > json.get value=`out` key="result"*
> print text=`val`
*sum = > json.get value=`out` key="summary"*
*has = > split value=`sum` sep="exact"*
*hn = > len value=`has`*
1. `hn` > 1
  > print text=sum-exact
2. *
  > print text=sum-bad

*out2 = > `助手`.plan goal=`near_g` max_rounds=2 writeback=False kb_dir=`kb` explore_n=0 promote=False*
*cache2 = > json.get value=`out2` key="cache"*
> print text=`cache2`
*mk2 = > json.get value=`out2` key="match"*
> print text=`mk2`
*val2 = > json.get value=`out2` key="result"*
> print text=`val2`
