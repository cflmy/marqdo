---
title: browser form (D3)
description: input wire passes value; set_text / set_class / set_attr effects
import json:lib/json.mq.md
---

# main

`wire` =

| @ | 选择器 | 事件 | 调用 |
|---|--------|------|------|
| 1 | "#name" | input | on_input |

**`wire`**

## on_input
    + `value`=""
    + `event`=""

1. `value` != ""
    *`label` = "你好，" + value*
    *`t` = > json.set map=None key="#echo" value=label*
    *`ret` = > json.set map=None key="set_text" value=t*
    *`c` = > json.set map=None key="#echo" value="hi"*
    *`ret` = > json.set map=ret key="set_class" value=c*
    *`attrs` = > json.set map=None key="data-name" value=value*
    *`a` = > json.set map=None key="#echo" value=attrs*
    **> json.set map=ret key="set_attr" value=a**
2. *
    *`t` = > json.set map=None key="#echo" value="（等待输入）"*
    *`ret` = > json.set map=None key="set_text" value=t*
    *`c` = > json.set map=None key="#echo" value=""*
    **> json.set map=ret key="set_class" value=c**
