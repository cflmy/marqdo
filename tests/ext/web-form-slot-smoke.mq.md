---
title: web form slot smoke
description: compose_form target= mounts form inside intro HTML (GAP-11).
import web:ext/web/web.mq.md
import json:lib/json.mq.md
import text:lib/text.mq.md
---

# main

`fields` =

| 字段 | 标签 | 类型 | 必填 | 默认 |
|------|------|------|------|------|
| title | Title | text | true | |

*f = > web.form table="notes" action="insert"*
*f = > `f`.fields fields=`fields`*

*intro = "<div class=\"cols\"><section id=\"left\">voice</section><div id=\"qd-note-form-mount\"></div></div>"*
*page = > web.page title="Slot" intro=`intro`*
*page = > `page`.compose_form id="note" form=`f` target="#qd-note-form-mount"*

*tgt = > json.get value=`page` key="form_target"*
1. `tgt`
  > print text=target-ok
2. *
  > print text=target-fail

*html = > `page`.render*
*`has_mount` = > text.contains text=`html` sub="qd-note-form-mount"*
*`has_form` = > text.contains text=`html` sub="site-form"*
*`order_ok` = > text.contains text=`html` sub="qd-note-form-mount\"><div class=\"site-form\""*
1. `has_mount`
  1. `has_form`
    1. `order_ok`
      > print text=slot-ok
    2. *
      > print text=slot-order-fail
  2. *
    > print text=slot-form-fail
2. *
  > print text=slot-mount-fail
