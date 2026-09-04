---
title: client counter (route D)
description: wired by client_embed auto-mount
import json:lib/json.mq.md
---

# main

*`count` = 0*

`wire` =

| @ | 选择器 | 事件 | 调用 |
|---|--------|------|------|
| 1 | "#bump" | click | bump |
| 2 | "#reset" | click | reset |

*`ready` = > json.set map=None key="#log" value="client.mq.md ready"*
*`enable_bump` = > json.set map=None key="disabled" value=False*
*`enable_reset` = > json.set map=None key="disabled" value=False*
*`attrs` = > json.set map=None key="#bump" value=enable_bump*
*`attrs` = > json.set map=attrs key="#reset" value=enable_reset*
*`boot` = > json.set map=None key="wire" value=wire*
*`boot` = > json.set map=boot key="set_text" value=ready*
**> json.set map=boot key="set_attr" value=attrs**

## bump
*`count` = count + 1*
*`label` = > str count*
*`patch` = > json.set map=None key="#count" value=label*
**> json.set map=None key="set_text" value=patch**

## reset
*`count` = 0*
*`patch` = > json.set map=None key="#count" value="0"*
**> json.set map=None key="set_text" value=patch**
