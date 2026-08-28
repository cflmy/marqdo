---
title: agent workbook multi-CONTINUE patch + solidify (A0 offline)
description: Two precise CONTINUE patches then agent_workbook_solidify freezes step workbook.
import agent:ext/ai/agent.mq.md
import json:lib/json.mq.md
import fs:lib/fs.mq.md
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

*paths = > json.parse text={"wb":".marqdo/agent-runs/a0-continue-wb.mq.md"}*
*wb = > json.get value=`paths` key="wb"*

*seed = > json.parse text={"t":"---\ntitle: a0 workbook\n---\n\n# Goal\n\nMARK_A keep\n\n# main\n\n\u002a\u0060r\u0060 = > \u0060worker\u0060.step task=demo \u002a\n\u002a\u002a\u0060r\u0060\u002a\u002a\n"}*
*body = > json.get value=`seed` key="t"*
> fs.write_text path=`wb` text=`body`

*p1 = > json.parse text={"t":"DECISION: CONTINUE\nPATCH:\n<<<\nFIND\nMARK_A keep\n===\nREPLACE\nMARK_B keep\n>>>\n"}*
*r1 = > json.get value=`p1` key="t"*
*n1 = > fs.apply_patch_blocks path=`wb` text=`r1`*
> print text=`n1`

*p2 = > json.parse text={"t":"DECISION: CONTINUE\nPATCH:\n<<<\nFIND\nMARK_B keep\n===\nREPLACE\nMARK_C done\n>>>\n"}*
*r2 = > json.get value=`p2` key="t"*
*n2 = > fs.apply_patch_blocks path=`wb` text=`r2`*
> print text=`n2`

*mid = > fs.read_text path=`wb`*
*has_c = > split value=`mid` sep="MARK_C done"*
*nc = > len value=`has_c`*
1. `nc` > 1
  > print text=continue-ok
2. *
  > print text=continue-fail

*obs = > json.parse text={"value":"frozen-answer"}*
*sol = > agent_workbook_solidify path=`wb` observation=`obs`*
*ok = > json.get value=`sol` key="solidified"*
1. `ok`
  > print text=solidify-ok
2. *
  > print text=solidify-fail

*final = > fs.read_text path=`wb`*
*still = > split value=`final` sep=".step"*
*ns = > len value=`still`*
1. `ns` == 1
  > print text=step-gone
2. *
  > print text=step-still

*has_ans = > split value=`final` sep="frozen-answer"*
*na = > len value=`has_ans`*
1. `na` > 1
  > print text=answer-ok
2. *
  > print text=answer-missing
