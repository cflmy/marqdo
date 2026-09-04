---
title: browser counter (C3)
description: wire table + browser.set_text — no json glue
import browser:lib/browser.mq.md
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
**> browser.set_text sel="#count" text=label**

## reset
*`count` = 0*
**> browser.set_text sel="#count" text="0"**
