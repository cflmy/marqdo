---
title: ext/agent step (DeepSeek)
description: Real agent.step with tool via lib/subtask; default writeback to ok/error slots.
> lib/time.mq.md
> lib/writeback.mq.md
> lib/json.mq.md
> ext/ai/llm.mq.md
> ext/ai/agent.mq.md
---

## 获取时间

*`u` = > time.now_unix *
*`s` = > time.format unix=`u` pattern=%Y-%m-%d *
**`s`**

# main

> llm.load_env path=.env

*`模型` = > llm.llm *

`工具表` =
| 工具 |
|------|
| 获取时间 |

*`助手` = > agent.agent model=`模型` tools=`工具表` standing=You are a Marqdo agent. When you need the current date, reply with exactly one line CALL:获取时间 and nothing else. *

*`out` = > `助手`.step task=请调用获取时间工具，然后用一句话告诉用户今天的日期。 *
<!-- marqdo-out ok
{"decision":"CALL:获取时间","result":"今天是2026年8月7日。","status":"ok","task":"请调用获取时间工具，然后用一句话告诉用户今天的日期。","tool":"获取时间","tool_result":"\"2026-08-07\""}
-->

*`status` = > json.get value=`out` key=status *
*`回复` = > json.get value=`out` key=result *

> print text=`回复`

*`cached` = > writeback.get key=ok *
1. `cached`
  > print text=writeback-ok
2. *
  > print text=writeback-missing

*`id` = > json.get value=`助手` key=id *
*`历史` = > agent_history_get id=`id` *
*`条数` = > len value=`历史` *
> print text=`条数`

> `助手`.clear_history

*`历史2` = > agent_history_get id=`id` *
*`条数2` = > len value=`历史2` *
> print text=`条数2`
