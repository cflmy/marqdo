---
title: agent plan decision + dual skeleton (offline)
description: extract_plan_decision/summary; dual skeleton contains research+writer; patch apply on fixture.
> ext/ai/agent.mq.md
> lib/json.mq.md
> lib/fs.mq.md
---

# main

*`samples` = > json.parse text={"done":"DECISION: DONE\nSUMMARY: all good\n","cont":"DECISION: CONTINUE\nPATCH:\n<<<\nFIND\nold\n===\nREPLACE\nnew\n>>>\n","zh":"决定：完成\n汇总：好了\n","run":"DECISION: RUN\n","zh_run":"决定：运行\n"} *
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

*`d4` = > json.get value=`samples` key=run *
*`r4` = > agent.extract_plan_decision reply=`d4` *
> print text=`r4`

*`d5` = > json.get value=`samples` key=zh_run *
*`r5` = > agent.extract_plan_decision reply=`d5` *
> print text=`r5`

*`act_call` = "CALL:lib_catalog" *
*`a1` = > agent.extract_plan_act reply=`act_call` *
*`a1k` = > json.get value=`a1` key=kind *
*`a1n` = > json.get value=`a1` key=name *
> print text=`a1k`
> print text=`a1n`

*`act_read` = "READ:stderr" *
*`a2` = > agent.extract_plan_act reply=`act_read` *
*`a2k` = > json.get value=`a2` key=kind *
*`a2n` = > json.get value=`a2` key=name *
> print text=`a2k`
> print text=`a2n`

*`act_dec` = > json.get value=`samples` key=done *
*`a3` = > agent.extract_plan_act reply=`act_dec` *
*`a3k` = > json.get value=`a3` key=kind *
> print text=`a3k`

*`cat` = > agent.lib_catalog *
*`files` = > json.get value=`cat` key=files *
*`nf` = > len value=`files` *
1. `nf` > 5
  > print text=catalog-ok
2. *
  > print text=catalog-bad

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
