---
title: OKF agent-kb near_match (offline)
description: Promote trip resource; near-dupe goals hit match=near; unrelated misses.
import agent:ext/ai/agent.mq.md
import fs:lib/fs.mq.md
import json:lib/json.mq.md
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
---

# main

*`p` = > plugin.native_path name=agent *
1. `p`
  > plugin.load path=`p`
2. *
  > print text=no-agent-plugin
  > sys.exit code=1

*`kb` = ".marqdo/agent-kb-near-test" *
*`wb` = ".marqdo/agent-runs/near-trip.mq.md" *
*`goal` = "帮我规划明天的行程" *
*`body` = "---\ntitle: trip\n---\n\n# main\n\n*msg = \"trip-ok\"*\n\n**msg**\n" *

> fs.write_text path=`wb` text=`body`
*`prom` = > agent_kb_promote kb_dir=`kb` goal=`goal` workbook=`wb` *
*`okp` = > json.get value=`prom` key=promoted *
1. `okp`
  > print text=promoted
2. *
  > print text=promote-fail

*`slug` = > json.get value=`prom` key=slug *

*`g1` = "帮我规划明天行程" *
*`hit1` = > agent_kb_lookup kb_dir=`kb` goal=`g1` near_match=True near_threshold=0.78 *
1. `hit1`
  > print text=near-ok
2. *
  > print text=near-miss
*`mk1` = > json.get value=`hit1` key=match *
> print text=`mk1`
*`slug1` = > json.get value=`hit1` key=slug *
1. `slug` == `slug1`
  > print text=same-slug
2. *
  > print text=slug-drift

*`off` = > agent_kb_lookup kb_dir=`kb` goal=`g1` near_match=False *
1. `off`
  > print text=off-hit
2. *
  > print text=off-miss

*`g2` = "明天做什么好" *
*`hit2` = > agent_kb_lookup kb_dir=`kb` goal=`g2` near_match=True near_threshold=0.78 *
1. `hit2`
  > print text=neg-hit
2. *
  > print text=neg-miss

*`ranked` = > agent_kb_near_match kb_dir=`kb` goal=`g1` *
*`best` = > json.get value=`ranked` key=best *
*`bs` = > json.get value=`best` key=slug *
1. `bs` == `slug`
  > print text=rank-ok
2. *
  > print text=rank-bad

*`tasks` = > json.get value=`ranked` key=candidates *
*`prompt` = > agent.build_soft_match_prompt goal=`g1` tasks=`tasks` *
*`has` = > split value=`prompt` sep=score= *
*`hn` = > len value=`has` *
1. `hn` > 1
  > print text=prompt-score-ok
2. *
  > print text=prompt-score-bad
