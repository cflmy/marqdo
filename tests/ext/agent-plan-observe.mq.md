---
title: parent observation excerpt + deepen (offline)
import agent:ext/ai/agent.mq.md
import json:lib/json.mq.md
import fs:lib/fs.mq.md
---

# main

*`paths` = > json.parse text={"p":".marqdo/agent-runs/observe-fixture.mq.md"} *
*`path` = > json.get value=`paths` key=p *
*`seed` = > json.parse text={"t":"---\ntitle: fixture\n---\n\n# main\n\n<!-- marqdo-out ok\nHUGE_BODY_SHOULD_BE_STRIPPED\n-->\n\n*msg = \"ok\"*\n\n**msg**\n"} *
*`text` = > json.get value=`seed` key=t *
> fs.write_text path=`path` text=`text`

*`obs` = > agent.inspect_workbook path=`path` exit_code=0 value=ok *
*`compact` = > agent.compact_plan_observation observation=`obs` *
*`ex` = > json.get value=`compact` key=source_excerpt *
*`parts` = > split value=`ex` sep=HUGE_BODY *
*`n` = > len value=`parts` *
1. `n` == 1
  > print text=excerpt-stripped
2. *
  > print text=excerpt-leak

*`has` = > json.get value=`compact` key=has_value *
1. `has`
  > print text=has-value-ok
2. *
  > print text=has-value-bad

*`deep` = > agent.plan_read_deepen observation=`obs` kind=source path=`path` *
*`ex2` = > json.get value=`deep` key=source_excerpt *
1. `ex2`
  > print text=read-source-ok
2. *
  > print text=read-source-bad
