---
title: ext/agent framework smoke (offline)
description: Ctor, context, clear_history — no network. Gold suite.
> ext/ai/agent.mq.md
---

# main

`工具表` =
| 工具 |
|------|
| 获取时间 |

*`占位模型` = > parse text={"_type":"llm"} *
*`助手` = > agent model=`占位模型` tools=`工具表` user_prompt=you are a dispatcher *

*`id` = > get value=`助手` key=id *

*`ctx` = > build_agent_context agent=`助手` extra=probe *

*`has_skill` = > split value=`ctx` sep=Marqdo *
*`sk_n` = > len value=`has_skill` *
+ `sk_n` > 1
  > print text=skill-ok
+ *
  > print text=skill-missing

*`has_tools` = > split value=`ctx` sep=获取时间 *
*`tool_n` = > len value=`has_tools` *
+ `tool_n` > 1
  > print text=tools-ok
+ *
  > print text=tools-missing

*`src` = > host_module_source *
*`has_src` = > split value=`src` sep=probe *
*`src_n` = > len value=`has_src` *
+ `src_n` > 1
  > print text=source-ok
+ *
  > print text=source-missing

*`turn` = > parse text={"role":"user","content":"one"} *
> host_agent_history_append id=`id` item=`turn`
*`turn2` = > parse text={"role":"assistant","content":"two"} *
> host_agent_history_append id=`id` item=`turn2`

*`h1` = > host_agent_history_get id=`id` *
*`n1` = > len value=`h1` *
> print text=`n1`

> `助手`.clear_history

*`h2` = > host_agent_history_get id=`id` *
*`n2` = > len value=`h2` *
> print text=`n2`

*`site` = > host_call_site *
*`fn` = > get value=`site` key=function *
> print text=`fn`
