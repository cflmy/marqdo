---
title: agent inspect_workbook (offline)
description: Parse named writeback slots from a fixture workbook path.
import agent:ext/ai/agent.mq.md
import json:lib/json.mq.md
import fs:lib/fs.mq.md
---

# main

*`paths` = > json.parse text={"f":".marqdo/agent-runs/inspect-fixture.mq.md"} *
*`fixture` = > json.get value=`paths` key=f *
*`body` = > json.parse text={"t":"# main\n\n> print text=hi\n<!-- marqdo-out ok\n{\"status\":\"ok\",\"result\":\"done\"}\n-->\n<!-- marqdo-out error\npending\n-->\n"} *
*`t` = > json.get value=`body` key=t *
> fs.write_text path=`fixture` text=`t`

*`obs` = > agent.inspect_workbook path=`fixture` exit_code=0 *
*`code` = > json.get value=`obs` key=exit_code *
> print text=`code`

*`ok` = > json.get value=`obs` key=last_ok *
*`has` = > split value=`ok` sep=done *
*`n` = > len value=`has` *
1. `n` > 1
  > print text=ok-slot
2. *
  > print text=ok-missing

*`err` = > json.get value=`obs` key=last_error *
> print text=`err`
