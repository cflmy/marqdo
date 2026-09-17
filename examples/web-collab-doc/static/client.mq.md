---
title: collab client
description: LWW text sync over WS room + presence (DOC\\t protocol)
import browser:lib/browser.mq.md
import table:lib/table.mq.md
import text:lib/text.mq.md
---

# main

**`ws_id` = "doc"**
**`applying` = False**
**`doc` = ""**

`wire` =

| @ | 选择器 | 事件 | 调用 | 值选择器 |
|---|--------|------|------|----------|
| 1 | "#doc" | input | on_edit | "#doc" |

`open` =

| op | id | url | then_open | then_message | then_error | then_close |
|----|----|-----|-----------|--------------|------------|------------|
| open | `ws_id` | "ws://127.0.0.1:18094/live/demo" | on_open | on_msg | on_err | on_close |

**`w` = > table.put in=None at="wire" value=wire**
**`ws` = > browser.ws spec=open**
**`boot` = > browser.merge a=w b=ws**
**`ready` = > browser.set_text sel="#log" text="connecting…"**
*> browser.merge a=boot b=ready*

## on_open
    + `ok`=True

*> browser.set_text sel="#log" text="ws open · room demo"*

## on_err
    + `error`=""

**`msg` = "ws error: " + error**
*> browser.set_text sel="#log" text=msg*

## on_close
*> browser.set_text sel="#log" text="ws closed"*

## on_edit
    + `value`=""

1. `applying`
    **`applying` = False**
    ****
2. *
    **`doc` = value**
    **`payload` = "DOC\t" + value**
    `send` =

    | op | id | data |
    |----|----|------|
    | send | `ws_id` | `payload` |

    *> browser.ws spec=send*

## on_msg
    + `data`=""

1. `data` != ""
    **`is_pres` = > text.starts_with text=data prefix="{"**
    1. `is_pres`
        **`msg` = "presence: " + data**
        *> browser.set_text sel="#presence" text=msg*
    2. *
        **`is_doc` = > text.starts_with text=data prefix="DOC\t"**
        1. `is_doc`
            **`remote` = > text.replace text=data old="DOC\t" new="" count=1**
            1. `remote` != `doc`
                **`doc` = remote**
                **`applying` = True**
                *> browser.set_value sel="#doc" value=remote*
            2. *
                ****
        2. *
            ****
2. *
    ****
