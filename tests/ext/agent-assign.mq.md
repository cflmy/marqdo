---
title: ext/agent match and assign
description: Multi-column tables + skill match (needs MARQDO_AGENT_PLUGIN).
> ext/agent.mq.md
---

# main

> load_native

*`cwd` = > cwd *
*`bot` = > agent root=`cwd` *

`技能表` =
| 成员 | 技能 | 负载 |
|------|------|------|
| 张三 | Python, ML | 2 |
| 李四 | 前端, UX | 1 |
| 王五 | 后端, 运维 | 3 |
| 赵六 | 数据分析 | 0 |

*`m` = > `bot`.match_skill skill=Python members=`技能表` *
*`name` = > get value=`m` key=成员 *
> print text=`name`

*`m2` = > `bot`.match_skill skill=安全审计 members=`技能表` *
+ `m2`
  > print text=unexpected-match
+ *
  *`path` = > `bot`.create_ticket title=no_owner detail=security *
  > print text=`path`

*`m3` = > `bot`.update_load member=`m` delta=1 *
*`load` = > get value=`m3` key=负载 *
> print text=`load`
