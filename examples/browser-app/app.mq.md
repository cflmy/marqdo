---
title: browser-app (route E+)
description: list CRUD, storage, navigate, interval, fetch_all, ws — GFM + lib/browser
import browser:lib/browser.mq.md
import table:lib/table.mq.md
import text:lib/text.mq.md
---

# main

*`ticks` = 0*
*`bag` = ""*
*`path` = "/"*
*`ws_id` = "echo"*

`wire` =

| @ | 选择器 | 事件 | 调用 | 委托 | 值选择器 |
|---|--------|------|------|------|----------|
| 1 | "#add" | click | add | | "#draft" |
| 2 | "#save" | click | save_draft | | "#draft" |
| 3 | "#load" | click | load_draft | | |
| 4 | "#start-tick" | click | start_tick | | |
| 5 | "#stop-tick" | click | stop_tick | | |
| 6 | "#nav-home" | click | go_home | | |
| 7 | "#nav-about" | click | go_about | | |
| 8 | "#nav-net" | click | go_net | | |
| 9 | "window" | popstate | on_pop | | |
| 10 | "#items" | click | on_item | "button[data-id]" | |
| 11 | "#fetch-all" | click | do_fetch_all | | |
| 12 | "#ws-open" | click | ws_open | | |
| 13 | "#ws-send" | click | ws_send | | "#ws-msg" |
| 14 | "#ws-close" | click | ws_close | | |
| 15 | "#copy-log" | click | copy_log | | |
| 16 | "#dl-list" | click | dl_list | | |

`nav` =

| url | replace |
|-----|---------|
| / | False |

`st` =

| op | key | then |
|----|-----|------|
| get | mq-draft | on_draft |

`boot` =

| wire | navigate | storage |
|------|----------|---------|
| `wire` | `nav` | `st` |

**boot**

## add
    + `value`=""

1. `value` != ""
    1. `bag` == ""
        *`bag` = value*
    2. *
        *`bag` = bag + "\n" + value*
    *`parts` = > text.str_split s=bag sep="\n"*
    *`html` = ""*
    - [item](parts)
        1. `item` != ""
            *`html` = html + "<li><span>" + item + "</span> <button type=\"button\" data-id=\"" + item + "\">删</button></li>"*
    *`ret` = > browser.set_html sel="#items" html=html*
    *`clr` = > browser.set_value sel="#draft" value=""*
    *`ret` = > browser.merge a=ret b=clr*
    *`f` = > browser.focus sel="#draft"*
    **> browser.merge a=ret b=f**
2. *
    ****

## on_item
    + `data_id`=""

1. `data_id` != ""
    *`parts` = > text.str_split s=bag sep="\n"*
    *`next` = ""*
    - [item](parts)
        1. `item` != ""
            1. `item` != `data_id`
                1. `next` == ""
                    *`next` = item*
                2. *
                    *`next` = next + "\n" + item*
    *`bag` = next*
    *`parts2` = > text.str_split s=bag sep="\n"*
    *`html` = ""*
    - [item](parts2)
        1. `item` != ""
            *`html` = html + "<li><span>" + item + "</span> <button type=\"button\" data-id=\"" + item + "\">删</button></li>"*
    *`ret` = > browser.set_html sel="#items" html=html*
    *`msg` = "removed " + data_id*
    *`t` = > browser.set_text sel="#log" text=msg*
    **> browser.merge a=ret b=t**
2. *
    ****

## save_draft
    + `value`=""

`st` =

| op | key | value |
|----|-----|-------|
| set | mq-draft | `value` |

*`msg` = > browser.set_text sel="#log" text="draft saved"*
*`s` = > browser.storage spec=st*
**> browser.merge a=msg b=s**

## load_draft

`st` =

| op | key | then |
|----|-----|------|
| get | mq-draft | on_draft |

**> browser.storage spec=st**

## on_draft
    + `ok`=True
    + `value`=""
    + `found`=False

1. `found`
    *`ret` = > browser.set_value sel="#draft" value=value*
    *`msg` = > browser.set_text sel="#log" text="draft loaded"*
    **> browser.merge a=ret b=msg**
2. *
    **> browser.set_text sel="#log" text="no draft"**

## start_tick
**> browser.interval ms=1000 then="on_tick" id="tick"**

## stop_tick
**> browser.clear_interval id="tick"**

## on_tick
    + `ok`=True
    + `id`=""

*`ticks` = ticks + 1*
*`label` = > str ticks*
**> browser.set_text sel="#tick" text=label**

## go_home
*`path` = "/"*
**> show_path**

## go_about
*`path` = "/about"*
**> show_path**

## go_net
*`path` = "/net"*
**> show_path**

## on_pop
    + `path`=""

*`path` = path*
**> show_path**

## show_path
*`hide` = > table.put in=None at="hidden" value=""*
*`show` = > table.put in=None at="hidden" value=False*
1. `path` == "/about"
    `attrs` =

    | #panel-about | #panel-home | #panel-net |
    |--------------|-------------|------------|
    | `show` | `hide` | `hide` |
2. *
    1. `path` == "/net"
        `attrs` =

        | #panel-net | #panel-home | #panel-about |
        |------------|-------------|--------------|
        | `show` | `hide` | `hide` |
    2. *
        `attrs` =

        | #panel-home | #panel-about | #panel-net |
        |-------------|--------------|------------|
        | `show` | `hide` | `hide` |
*`label` = "path: " + path*
*`ret` = > browser.set_text sel="#path" text=label*
*`a` = > browser.wrap key="set_attr" value=attrs*
*`ret` = > browser.merge a=ret b=a*
*`nav` = > browser.navigate url=path*
**> browser.merge a=ret b=nav**

## do_fetch_all

`reqs` =

| @ | url |
|---|-----|
| 1 | "https://httpbin.org/uuid" |
| 2 | "https://httpbin.org/uuid" |

*`msg` = > browser.set_text sel="#net-out" text="fetch_all…"*
*`fa` = > browser.fetch_all requests=reqs then="on_fetch_all"*
**> browser.merge a=msg b=fa**

## on_fetch_all
    + `ok`=True
    + `results`=None

**> browser.set_text sel="#net-out" text="fetch_all done (see console / results in session)"**

## ws_open

`w` =

| op | id | url | then_open | then_message | then_error | then_close |
|----|----|-----|-----------|--------------|------------|------------|
| open | `ws_id` | "wss://echo.websocket.events" | on_ws_open | on_ws_msg | on_ws_err | on_ws_close |

**> browser.ws spec=w**

## on_ws_open
    + `ok`=True
    + `id`=""

**> browser.set_text sel="#net-out" text="ws open"**

## on_ws_msg
    + `ok`=True
    + `data`=""

*`msg` = "ws ← " + data*
**> browser.set_text sel="#net-out" text=msg**

## on_ws_err
    + `ok`=False
    + `error`=""

*`msg` = "ws error: " + error*
**> browser.set_text sel="#net-out" text=msg**

## on_ws_close
    + `ok`=True

**> browser.set_text sel="#net-out" text="ws closed"**

## ws_send
    + `value`=""

`w` =

| op | id | data |
|----|----|------|
| send | `ws_id` | `value` |

**> browser.ws spec=w**

## ws_close

`w` =

| op | id |
|----|----|
| close | `ws_id` |

**> browser.ws spec=w**

## copy_log
*`msg` = > browser.set_text sel="#log" text="copied bag to clipboard"*
*`c` = > browser.clipboard text=bag*
**> browser.merge a=msg b=c**

## dl_list
*`msg` = > browser.set_text sel="#log" text="download started"*
*`d` = > browser.download body=bag filename="items.txt"*
**> browser.merge a=msg b=d**
