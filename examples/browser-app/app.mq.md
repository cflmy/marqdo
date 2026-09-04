---
title: browser-app (route E+)
description: list CRUD, storage, navigate, interval, fetch_all, ws — author zero JS
import json:lib/json.mq.md
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

*`nav` = > json.set map=None key="url" value="/"*
*`boot` = > json.set map=None key="wire" value=wire*
*`boot` = > json.set map=boot key="navigate" value=nav*
*`st` = > json.set map=None key="op" value="get"*
*`st` = > json.set map=st key="key" value="mq-draft"*
*`st` = > json.set map=st key="then" value="on_draft"*
**> json.set map=boot key="storage" value=st**

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
    *`h` = > json.set map=None key="#items" value=html*
    *`ret` = > json.set map=None key="set_html" value=h*
    *`clr` = > json.set map=None key="#draft" value=""*
    *`ret` = > json.set map=ret key="set_value" value=clr*
    **> json.set map=ret key="focus" value="#draft"**
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
    *`h` = > json.set map=None key="#items" value=html*
    *`ret` = > json.set map=None key="set_html" value=h*
    *`msg` = "removed " + data_id*
    *`t` = > json.set map=None key="#log" value=msg*
    **> json.set map=ret key="set_text" value=t**
2. *
    ****

## save_draft
    + `value`=""

*`st` = > json.set map=None key="op" value="set"*
*`st` = > json.set map=st key="key" value="mq-draft"*
*`st` = > json.set map=st key="value" value=value*
*`msg` = > json.set map=None key="#log" value="draft saved"*
*`ret` = > json.set map=None key="set_text" value=msg*
**> json.set map=ret key="storage" value=st**

## load_draft
*`st` = > json.set map=None key="op" value="get"*
*`st` = > json.set map=st key="key" value="mq-draft"*
*`st` = > json.set map=st key="then" value="on_draft"*
**> json.set map=None key="storage" value=st**

## on_draft
    + `ok`=True
    + `value`=""
    + `found`=False

1. `found`
    *`v` = > json.set map=None key="#draft" value=value*
    *`ret` = > json.set map=None key="set_value" value=v*
    *`msg` = > json.set map=None key="#log" value="draft loaded"*
    **> json.set map=ret key="set_text" value=msg**
2. *
    *`msg` = > json.set map=None key="#log" value="no draft"*
    **> json.set map=None key="set_text" value=msg**

## start_tick
*`iv` = > json.set map=None key="ms" value=1000*
*`iv` = > json.set map=iv key="then" value="on_tick"*
*`iv` = > json.set map=iv key="id" value="tick"*
**> json.set map=None key="interval" value=iv**

## stop_tick
*`c` = > json.set map=None key="id" value="tick"*
**> json.set map=None key="clear_interval" value=c**

## on_tick
    + `ok`=True
    + `id`=""

*`ticks` = ticks + 1*
*`label` = > str ticks*
*`p` = > json.set map=None key="#tick" value=label*
**> json.set map=None key="set_text" value=p**

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
*`nav` = > json.set map=None key="url" value=path*
*`hide` = > json.set map=None key="hidden" value=""*
*`show` = > json.set map=None key="hidden" value=False*
1. `path` == "/about"
    *`attrs` = > json.set map=None key="#panel-about" value=show*
    *`attrs` = > json.set map=attrs key="#panel-home" value=hide*
    *`attrs` = > json.set map=attrs key="#panel-net" value=hide*
2. *
    1. `path` == "/net"
        *`attrs` = > json.set map=None key="#panel-net" value=show*
        *`attrs` = > json.set map=attrs key="#panel-home" value=hide*
        *`attrs` = > json.set map=attrs key="#panel-about" value=hide*
    2. *
        *`attrs` = > json.set map=None key="#panel-home" value=show*
        *`attrs` = > json.set map=attrs key="#panel-about" value=hide*
        *`attrs` = > json.set map=attrs key="#panel-net" value=hide*
*`label` = "path: " + path*
*`t` = > json.set map=None key="#path" value=label*
*`ret` = > json.set map=None key="set_text" value=t*
*`ret` = > json.set map=ret key="set_attr" value=attrs*
**> json.set map=ret key="navigate" value=nav**

## do_fetch_all
*`a` = > json.set map=None key="url" value="https://httpbin.org/uuid"*
*`b` = > json.set map=None key="url" value="https://httpbin.org/uuid"*
*`reqs` = > json.set map=None key="0" value=a*
*`reqs` = > json.set map=reqs key="1" value=b*
*`fa` = > json.set map=None key="then" value="on_fetch_all"*
*`fa` = > json.set map=fa key="requests" value=reqs*
*`msg` = > json.set map=None key="#net-out" value="fetch_all…"*
*`ret` = > json.set map=None key="set_text" value=msg*
**> json.set map=ret key="fetch_all" value=fa**

## on_fetch_all
    + `ok`=True
    + `results`=None

*`msg` = "fetch_all done (see console / results in session)"*
*`t` = > json.set map=None key="#net-out" value=msg*
**> json.set map=None key="set_text" value=t**

## ws_open
*`w` = > json.set map=None key="op" value="open"*
*`w` = > json.set map=w key="id" value=ws_id*
*`w` = > json.set map=w key="url" value="wss://echo.websocket.events"*
*`w` = > json.set map=w key="then_open" value="on_ws_open"*
*`w` = > json.set map=w key="then_message" value="on_ws_msg"*
*`w` = > json.set map=w key="then_error" value="on_ws_err"*
*`w` = > json.set map=w key="then_close" value="on_ws_close"*
**> json.set map=None key="ws" value=w**

## on_ws_open
    + `ok`=True
    + `id`=""

*`t` = > json.set map=None key="#net-out" value="ws open"*
**> json.set map=None key="set_text" value=t**

## on_ws_msg
    + `ok`=True
    + `data`=""

*`msg` = "ws ← " + data*
*`t` = > json.set map=None key="#net-out" value=msg*
**> json.set map=None key="set_text" value=t**

## on_ws_err
    + `ok`=False
    + `error`=""

*`msg` = "ws error: " + error*
*`t` = > json.set map=None key="#net-out" value=msg*
**> json.set map=None key="set_text" value=t**

## on_ws_close
    + `ok`=True

*`t` = > json.set map=None key="#net-out" value="ws closed"*
**> json.set map=None key="set_text" value=t**

## ws_send
    + `value`=""

*`w` = > json.set map=None key="op" value="send"*
*`w` = > json.set map=w key="id" value=ws_id*
*`w` = > json.set map=w key="data" value=value*
**> json.set map=None key="ws" value=w**

## ws_close
*`w` = > json.set map=None key="op" value="close"*
*`w` = > json.set map=w key="id" value=ws_id*
**> json.set map=None key="ws" value=w**

## copy_log
*`c` = > json.set map=None key="text" value=bag*
*`msg` = > json.set map=None key="#log" value="copied bag to clipboard"*
*`ret` = > json.set map=None key="set_text" value=msg*
**> json.set map=ret key="clipboard" value=c**

## dl_list
*`d` = > json.set map=None key="filename" value="items.txt"*
*`d` = > json.set map=d key="body" value=bag*
*`msg` = > json.set map=None key="#log" value="download started"*
*`ret` = > json.set map=None key="set_text" value=msg*
**> json.set map=ret key="download" value=d**
