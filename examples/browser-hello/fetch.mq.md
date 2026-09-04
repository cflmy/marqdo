---
title: browser fetch (C4)
description: fetch/after effects via lib/browser — no json glue
import browser:lib/browser.mq.md
---

# main

`wire` =

| @ | 选择器 | 事件 | 调用 |
|---|--------|------|------|
| 1 | "#load" | click | load |
| 2 | "#ping" | click | ping |

**`wire`**

## load
*`msg` = > browser.set_text sel="#status" text="loading…"*
*`req` = > browser.fetch url="https://httpbin.org/uuid" then="on_uuid"*
**> browser.merge a=msg b=req**

## on_uuid
    + `ok`
    + `status`
    + `body`=""
    + `error`=""

1. ok
    **> browser.set_text sel="#status" text=body**
2. *
    **> browser.set_text sel="#status" text="fetch failed"**

## ping
*`msg` = > browser.set_text sel="#status" text="wait 600ms…"*
*`t` = > browser.after ms=600 then="on_ping"*
**> browser.merge a=msg b=t**

## on_ping
    + `ok`=True

**> browser.set_text sel="#status" text="pong"**
