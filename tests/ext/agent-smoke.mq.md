---
title: ext/agent framework smoke
description: Real llm handle from tests/ext/.env; context, extract_tool_name, run_tool via subtask.
import llm:ext/ai/llm.mq.md
import json:lib/json.mq.md
import agent:ext/ai/agent.mq.md
---

## 获取时间

*`esc` = > json.parse text={"v":"ok-time"} *
**> json.get value=`esc` key=v**

# main

> llm.load_env path=.env

`工具表` =
| 工具 |
|------|
| 获取时间 |

*`模型` = > llm.llm *
*`助手` = > agent.agent model=`模型` tools=`工具表` standing=you are a readable Marqdo agent *

*`id` = > json.get value=`助手` key=id *

*`ctx` = > agent.build_step_context agent=`助手` task=probe *

*`has_skill` = > split value=`ctx` sep=Marqdo *
*`sk_n` = > len value=`has_skill` *
1. `sk_n` > 1
  > print text=skill-ok
2. *
  > print text=skill-missing

*`has_tools` = > split value=`ctx` sep=获取时间 *
*`tool_n` = > len value=`has_tools` *
1. `tool_n` > 1
  > print text=tools-ok
2. *
  > print text=tools-missing

*`has_src` = > split value=`ctx` sep=probe *
*`src_n` = > len value=`has_src` *
1. `src_n` > 1
  > print text=source-ok
2. *
  > print text=source-missing

*`has_act` = > split value=`ctx` sep=CALL: *
*`act_n` = > len value=`has_act` *
1. `act_n` > 1
  > print text=protocol-ok
2. *
  > print text=protocol-missing

*`samples` = > json.parse text={"r":"CALL:获取时间","m":"think\nCALL:获取时间"} *
*`raw1` = > json.get value=`samples` key=r *
*`name1` = > agent.extract_tool_name reply=`raw1` *
> print text=`name1`

*`raw2` = > json.get value=`samples` key=m *
*`name2` = > agent.extract_tool_name reply=`raw2` *
> print text=`name2`

*`tool_out` = > agent.run_tool tools=`工具表` name=获取时间 *
> print text=`tool_out`

*`turn` = > json.parse text={"role":"user","content":"one"} *
> agent_history_append id=`id` item=`turn`
*`turn2` = > json.parse text={"role":"assistant","content":"two"} *
> agent_history_append id=`id` item=`turn2`

*`h1` = > agent_history_get id=`id` *
*`n1` = > len value=`h1` *
> print text=`n1`

> `助手`.clear_history

*`h2` = > agent_history_get id=`id` *
*`n2` = > len value=`h2` *
> print text=`n2`

*`site` = > agent_call_site *
*`fn` = > json.get value=`site` key=function *
> print text=`fn`
