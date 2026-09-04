---
title: client counter (route D)
description: wired by client_embed auto-mount; GFM + lib/browser
import browser:lib/browser.mq.md
import table:lib/table.mq.md
---

# main

*`count` = 0*

`wire` =

| @ | 选择器 | 事件 | 调用 |
|---|--------|------|------|
| 1 | "#bump" | click | bump |
| 2 | "#reset" | click | reset |

*`ready` = > browser.set_text sel="#log" text="client.mq.md ready"*
*`boot` = > table.put in=None at="wire" value=wire*
**> browser.merge a=boot b=ready**

## bump
*`count` = count + 1*
*`label` = > str count*
**> browser.set_text sel="#count" text=label**

## reset
*`count` = 0*
**> browser.set_text sel="#count" text="0"**
