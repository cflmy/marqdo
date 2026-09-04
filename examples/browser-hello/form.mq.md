---
title: browser form (D3)
description: input wire passes value; effects via lib/browser
import browser:lib/browser.mq.md
import table:lib/table.mq.md
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
    *`ret` = > browser.set_text sel="#echo" text=label*
    *`cls` = > browser.set_class sel="#echo" class="hi"*
    *`ret` = > browser.merge a=ret b=cls*
    *`attrs` = > table.put in=None at="data-name" value=value*
    *`attr` = > browser.set_attr sel="#echo" attrs=attrs*
    **> browser.merge a=ret b=attr**
2. *
    *`ret` = > browser.set_text sel="#echo" text="（等待输入）"*
    *`cls` = > browser.set_class sel="#echo" class=""*
    **> browser.merge a=ret b=cls**
