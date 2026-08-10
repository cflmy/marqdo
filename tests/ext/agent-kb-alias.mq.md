---
title: OKF agent-kb alias lookup (offline)
description: Promote resource; patch Task aliases; variant goal hits same resource.
> ext/ai/agent.mq.md
> lib/fs.mq.md
> lib/json.mq.md
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

*`kb` = .marqdo/agent-kb-alias-test *
*`wb` = .marqdo/agent-runs/alias-pong.mq.md *
*`goal` = Reply with exactly the word pong and nothing else. *
*`alias` = say pong only *
*`body` = "---\ntitle: pong skill\n---\n\n# main\n\n**pong**\n" *

> fs.write_text path=`wb` text=`body`
*`prom` = > agent_kb_promote kb_dir=`kb` goal=`goal` workbook=`wb` aliases=`alias` *
*`okp` = > json.get value=`prom` key=promoted *
1. `okp`
  > print text=promoted
2. *
  > print text=promote-fail

*`slug` = > json.get value=`prom` key=slug *
*`parts` = > json.parse text={"a":"/concepts/tasks/","b":".md"} *
*`a` = > json.get value=`parts` key=a *
*`b` = > json.get value=`parts` key=b *
*`task` = `kb` + `a` + `slug` + `b` *
*`src` = > fs.read_text path=`task` *
*`has` = > split value=`src` sep=say pong only *
*`hn` = > len value=`has` *
1. `hn` > 1
  > print text=alias-written
2. *
  > print text=alias-missing

*`hit` = > agent_kb_lookup kb_dir=`kb` goal=`alias` *
1. `hit`
  > print text=alias-ok
2. *
  > print text=alias-miss

*`mk` = > json.get value=`hit` key=match *
> print text=`mk`

*`slug_hit` = > json.get value=`hit` key=slug *
1. `slug` == `slug_hit`
  > print text=same-slug
2. *
  > print text=slug-drift

*`exact` = > agent_kb_lookup kb_dir=`kb` goal=`goal` *
*`mk2` = > json.get value=`exact` key=match *
> print text=`mk2`
