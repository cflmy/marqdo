---
title: OKF agent-kb alias lookup (offline)
description: Promote resource; patch Task aliases; variant goal hits same resource.
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

*kb = ".marqdo/agent-kb-alias-test"*
*wb = ".marqdo/agent-runs/alias-pong.mq.md"*
*goal = "Reply with exactly the word pong and nothing else."*
*alias = "say pong only"*
*body = "---\ntitle: pong skill\n---\n\n# main\n\n*msg = \"pong\"*\n\n**msg**\n"*

> fs.write_text path=`wb` text=`body`
*prom = > agent_kb_promote kb_dir=`kb` goal=`goal` workbook=`wb` aliases=`alias`*
*okp = > json.get value=`prom` key="promoted"*
1. `okp`
  > print text=promoted
2. *
  > print text=promote-fail

*slug = > json.get value=`prom` key="slug"*
*parts = > json.parse text={"a":"/concepts/tasks/","b":".md"}*
*a = > json.get value=`parts` key="a"*
*b = > json.get value=`parts` key="b"*
*task = kb + a + slug + b*
*src = > fs.read_text path=`task`*
*has = > split value=`src` sep="say pong only"*
*hn = > len value=`has`*
1. `hn` > 1
  > print text=alias-written
2. *
  > print text=alias-missing

*hit = > agent_kb_lookup kb_dir=`kb` goal=`alias`*
1. `hit`
  > print text=alias-ok
2. *
  > print text=alias-miss

*mk = > json.get value=`hit` key="match"*
> print text=`mk`

*slug_hit = > json.get value=`hit` key="slug"*
1. `slug` == `slug_hit`
  > print text=same-slug
2. *
  > print text=slug-drift

*exact = > agent_kb_lookup kb_dir=`kb` goal=`goal`*
*mk2 = > json.get value=`exact` key="match"*
> print text=`mk2`

*g_trip = "帮我规划明天的行程"*
*wb2 = ".marqdo/agent-runs/alias-trip.mq.md"*
*body2 = "---\ntitle: trip\n---\n\n# main\n\n*msg = \"trip-ok\"*\n\n**msg**\n"*
> fs.write_text path=`wb2` text=`body2`
*prom2 = > agent_kb_promote kb_dir=`kb` goal=`g_trip` workbook=`wb2`*
*ok2 = > json.get value=`prom2` key="promoted"*
1. `ok2`
  > print text=trip-promoted
2. *
  > print text=trip-promote-fail

*g_canon = "你是一个智能体，帮我规划明天的行程"*
*c = > agent_kb_canonicalize goal=`g_canon`*
> print text=`c`
*hit_c = > agent_kb_lookup kb_dir=`kb` goal=`g_canon`*
1. `hit_c`
  > print text=canonical-ok
2. *
  > print text=canonical-miss
*mk3 = > json.get value=`hit_c` key="match"*
> print text=`mk3`

*listed = > agent_kb_list_tasks kb_dir=`kb`*
*tc = > json.get value=`listed` key="count"*
1. `tc` > 1
  > print text=list-ok
2. *
  > print text=list-bad
