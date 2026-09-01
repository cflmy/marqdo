---
title: browser fetch (C4)
description: return fetch/after effects; bridge continues via mq_call
import json:lib/json.mq.md
---

# main

`wire` =

| @ | 选择器 | 事件 | 调用 |
|---|--------|------|------|
| 1 | "#load" | click | load |
| 2 | "#ping" | click | ping |

**`wire`**

## load
*`patch` = > json.set map=None key="#status" value="loading…"*
*`ret` = > json.set map=None key="set_text" value=patch*
*`req` = > json.set map=None key="url" value="https://httpbin.org/uuid"*
*`req` = > json.set map=req key="then" value="on_uuid"*
**> json.set map=ret key="fetch" value=req**

## on_uuid
    + `ok`
    + `status`
    + `body`=""
    + `error`=""

1. ok
    *`p` = > json.set map=None key="#status" value=body*
    **> json.set map=None key="set_text" value=p**
2. *
    *`msg` = "fetch failed"*
    *`p` = > json.set map=None key="#status" value=msg*
    **> json.set map=None key="set_text" value=p**

## ping
*`patch` = > json.set map=None key="#status" value="wait 600ms…"*
*`ret` = > json.set map=None key="set_text" value=patch*
*`t` = > json.set map=None key="ms" value=600*
*`t` = > json.set map=t key="then" value="on_ping"*
**> json.set map=ret key="after" value=t**

## on_ping
    + `ok`=True

*`p` = > json.set map=None key="#status" value="pong"*
**> json.set map=None key="set_text" value=p**
