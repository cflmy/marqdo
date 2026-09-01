---
title: browser counter (C3)
description: wire table + shared entry env; click updates DOM via set_text
import json:lib/json.mq.md
---

# main

*`count` = 0*

`wire` =

| @ | 选择器 | 事件 | 调用 |
|---|--------|------|------|
| 1 | "#bump" | click | bump |
| 2 | "#reset" | click | reset |

**`wire`**

## bump
*`count` = count + 1*
*`label` = > str count*
*`patch` = > json.set map=None key="#count" value=label*
**> json.set map=None key="set_text" value=patch**

## reset
*`count` = 0*
*`patch` = > json.set map=None key="#count" value="0"*
**> json.set map=None key="set_text" value=patch**
