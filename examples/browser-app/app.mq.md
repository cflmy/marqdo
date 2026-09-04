---
title: browser-app (route E)
description: list html, storage, navigate, interval — author zero JS
import json:lib/json.mq.md
---

# main

*`ticks` = 0*
*`items_html` = ""*
*`path` = "/"*

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
| 8 | "window" | popstate | on_pop | | |
| 9 | "#items" | click | on_item | "button" | |

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
    *`items_html` = items_html + "<li><span>" + value + "</span> <button type=\"button\" data-id=\"" + value + "\">删</button></li>"*
    *`h` = > json.set map=None key="#items" value=items_html*
    *`ret` = > json.set map=None key="set_html" value=h*
    *`clr` = > json.set map=None key="#draft" value=""*
    *`ret` = > json.set map=ret key="set_value" value=clr*
    **> json.set map=ret key="focus" value="#draft"**
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

## on_pop
    + `path`=""

*`path` = path*
**> show_path**

## show_path
*`nav` = > json.set map=None key="url" value=path*
1. `path` == "/about"
    *`hide` = > json.set map=None key="hidden" value=""*
    *`show` = > json.set map=None key="hidden" value=False*
    *`attrs` = > json.set map=None key="#panel-about" value=show*
    *`attrs` = > json.set map=attrs key="#panel-home" value=hide*
2. *
    *`hide` = > json.set map=None key="hidden" value=""*
    *`show` = > json.set map=None key="hidden" value=False*
    *`attrs` = > json.set map=None key="#panel-home" value=show*
    *`attrs` = > json.set map=attrs key="#panel-about" value=hide*
*`label` = "path: " + path*
*`t` = > json.set map=None key="#path" value=label*
*`ret` = > json.set map=None key="set_text" value=t*
*`ret` = > json.set map=ret key="set_attr" value=attrs*
**> json.set map=ret key="navigate" value=nav**

## on_item
    + `data_id`=""
    + `event`=""

1. `data_id` != ""
    *`msg` = "removed hint: " + data_id + " (re-add to refresh list demo)"*
    *`t` = > json.set map=None key="#log" value=msg*
    **> json.set map=None key="set_text" value=t**
2. *
    ****
