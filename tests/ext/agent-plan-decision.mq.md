---
title: agent plan decision + dual skeleton (offline)
description: extract_plan_decision/summary; dual skeleton contains research+writer; patch apply on fixture.
> ext/ai/agent.mq.md
> lib/json.mq.md
> lib/fs.mq.md
---

# main

*`samples` = > json.parse text={"done":"DECISION: DONE\nSUMMARY: all good\n","cont":"DECISION: CONTINUE\nPATCH:\n<<<\nFIND\nold\n===\nREPLACE\nnew\n>>>\n","zh":"决定：完成\n汇总：好了\n"} *
*`d1` = > json.get value=`samples` key=done *
*`r1` = > agent.extract_plan_decision reply=`d1` *
> print text=`r1`
*`s1` = > agent.extract_plan_summary reply=`d1` *
> print text=`s1`

*`d2` = > json.get value=`samples` key=cont *
*`r2` = > agent.extract_plan_decision reply=`d2` *
> print text=`r2`

*`d3` = > json.get value=`samples` key=zh *
*`r3` = > agent.extract_plan_decision reply=`d3` *
> print text=`r3`

*`dual` = > agent.render_workbook_skeleton goal=demo skeleton=dual *
*`has_r` = > split value=`dual` sep=research *
*`n_r` = > len value=`has_r` *
1. `n_r` > 1
  > print text=dual-ok
2. *
  > print text=dual-missing

*`has_s` = > split value=`dual` sep=Solidify *
*`n_s` = > len value=`has_s` *
1. `n_s` > 1
  > print text=solidify-ok
2. *
  > print text=solidify-missing

*`paths` = > json.parse text={"p":".marqdo/agent-runs/decision-patch.md","t":"keep old keep"} *
*`path` = > json.get value=`paths` key=p *
*`seed` = > json.get value=`paths` key=t *
> fs.write_text path=`path` text=`seed`
*`n` = > fs.apply_patch_blocks path=`path` text=`d2` *
> print text=`n`
*`out` = > fs.read_text path=`path` *
> print text=`out`
