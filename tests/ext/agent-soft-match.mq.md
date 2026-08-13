---
title: OKF soft_match protocol parse (offline)
description: REUSE/NEW + SLUG extractors; no network.
import agent:ext/ai/agent.mq.md
import json:lib/json.mq.md
---

# main

*`samples` = > json.parse text={"reuse":"DECISION: REUSE\nSLUG: trip-plan\n","new":"DECISION: NEW\n","zh":"决定：复用\n标识：行程-a\n"} *
*`r1` = > json.get value=`samples` key=reuse *
*`d1` = > agent.extract_plan_decision reply=`r1` *
> print text=`d1`
*`s1` = > agent.extract_soft_match_slug reply=`r1` *
> print text=`s1`

*`r2` = > json.get value=`samples` key=new *
*`d2` = > agent.extract_plan_decision reply=`r2` *
> print text=`d2`

*`r3` = > json.get value=`samples` key=zh *
*`d3` = > agent.extract_plan_decision reply=`r3` *
> print text=`d3`
*`s3` = > agent.extract_soft_match_slug reply=`r3` *
> print text=`s3`

*`tasks` = > json.parse text=[{"slug":"a","title":"Alpha"},{"slug":"b","title":"Beta"}] *
*`prompt` = > agent.build_soft_match_prompt goal=demo tasks=`tasks` *
*`has` = > split value=`prompt` sep=DECISION: REUSE *
*`n` = > len value=`has` *
1. `n` > 1
  > print text=prompt-ok
2. *
  > print text=prompt-bad
