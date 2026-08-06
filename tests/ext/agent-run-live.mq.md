---
title: ext/agent run live (DeepSeek)
description: Real 执行 via tests/ext/.env (gitignored). Skipped in CI when .env missing.
> ext/ai/agent.mq.md
> lib/time.mq.md
---

## 获取时间

*`u` = > now_unix *
**> format unix=`u` pattern=%Y%m%d **

# main

> load_env

*`模型` = > llm *

`工具表` =
| 工具 |
|------|
| 获取时间 |

*`助手` = > agent model=`模型` tools=`工具表` user_prompt=你是测试助手。需要工具时，必须只回复一行 TOOL:函数名，不要其它文字。 *

*`id` = > get value=`助手` key=id *

*`回复` = > `助手`.run extra=请调用获取时间工具，并把当前日期告诉用户。 *

> print text=`回复`

*`历史1` = > host_agent_history_get id=`id` *
*`条数1` = > len value=`历史1` *
> print text=`条数1`

> `助手`.clear_history

*`历史2` = > host_agent_history_get id=`id` *
*`条数2` = > len value=`历史2` *
> print text=`条数2`
