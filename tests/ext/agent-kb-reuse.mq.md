---
title: OKF agent-kb promote + lookup + spawn (offline)
description: Promote a solidified workbook, lookup by goal, spawn without parent LLM.
> ext/ai/agent.mq.md
> lib/fs.mq.md
> lib/json.mq.md
> lib/subtask.mq.md
> lib/plugin.mq.md
> lib/sys.mq.md
---

# main

*`p` = > plugin.native_path name=agent *
1. `p`
  > plugin.load path=`p`
2. *
  > print text=no-agent-plugin
  > sys.exit code=1

*`paths` = > json.parse text={"kb":".marqdo/agent-kb","wb":".marqdo/agent-runs/pong-solid.mq.md","goal":"Reply with exactly the word pong and nothing else.","body":"---\ntitle: pong skill\n---\n\n# main\n\n**pong**\n"} *
*`kb` = > json.get value=`paths` key=kb *
*`wb` = > json.get value=`paths` key=wb *
*`goal` = > json.get value=`paths` key=goal *
*`body` = > json.get value=`paths` key=body *

> fs.write_text path=`wb` text=`body`

*`sig` = > agent_goal_sig goal=`goal` *
> print text=`sig`

*`slug0` = > agent_goal_slug goal=`goal` *
> print text=`slug0`

*`prom` = > agent_kb_promote kb_dir=`kb` goal=`goal` workbook=`wb` *
*`okp` = > json.get value=`prom` key=promoted *
1. `okp`
  > print text=promoted
2. *
  > print text=promote-fail

*`st` = > json.get value=`prom` key=status *
> print text=`st`

*`slug` = > json.get value=`prom` key=slug *
> print text=`slug`

*`hit` = > agent_kb_lookup kb_dir=`kb` goal=`goal` *
1. `hit`
  > print text=lookup-ok
2. *
  > print text=lookup-miss

*`res` = > json.get value=`hit` key=resource *
*`id` = > subtask.spawn path=`res` *
*`waited` = > subtask.wait id=`id` *
*`code` = > json.get value=`waited` key=code *
*`val` = > json.get value=`waited` key=value *
> print text=`code`
> print text=`val`

*`hit2` = > agent_kb_lookup kb_dir=`kb` goal=`goal` *
*`res2` = > json.get value=`hit2` key=resource *
1. `res` == `res2`
  > print text=stable-path
2. *
  > print text=path-drift
