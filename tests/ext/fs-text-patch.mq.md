---
title: fs text_patch / apply_patch_blocks (offline)
> lib/fs.mq.md
> lib/json.mq.md
---

# main

*`paths` = > json.parse text={"p":".marqdo/agent-runs/patch-demo.md","t":"alpha beta gamma"} *
*`path` = > json.get value=`paths` key=p *
*`seed` = > json.get value=`paths` key=t *
> fs.write_text path=`path` text=`seed`

> fs.text_patch path=`path` find=beta replace=BETA
*`s1` = > fs.read_text path=`path` *
> print text=`s1`

*`reply` = > json.parse text={"t":"DECISION: CONTINUE\n<<<\nFIND\nalpha\n===\nREPLACE\nALPHA\n>>>\n<<<\nFIND\ngamma\n===\nREPLACE\nGAMMA\n>>>\n"} *
*`t` = > json.get value=`reply` key=t *
*`n` = > fs.apply_patch_blocks path=`path` text=`t` *
> print text=`n`
*`s2` = > fs.read_text path=`path` *
> print text=`s2`
