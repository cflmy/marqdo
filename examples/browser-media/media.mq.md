---
title: browser-media (route F)
description: file / canvas / audio / observers / drag — GFM tables + lib/browser
import browser:lib/browser.mq.md
import table:lib/table.mq.md
---

# main

Route F demo: author data stays in tables; effects via `browser.*`.

*`obs_on` = False*

`wire` =

| @ | 选择器 | 事件 | 调用 | 委托 |
|---|--------|------|------|------|
| 1 | "#read-text" | click | read_text | |
| 2 | "#read-dataurl" | click | read_dataurl | |
| 3 | "#draw" | click | draw_box | |
| 4 | "#play" | click | play_tone | |
| 5 | "#watch" | click | start_watch | |
| 6 | "#unwatch" | click | stop_watch | |
| 7 | "#chips" | dragstart | on_drag | "[data-drag]" |
| 8 | "#dropzone" | dragover | on_dragover | |
| 9 | "#dropzone" | drop | on_drop | |

*`ready` = > browser.set_text sel="#log" text="ready (route F)"*
*`boot` = > table.put in=None at="wire" value=wire*
**> browser.merge a=boot b=ready**

## read_text
**> browser.read_file sel="#file" then="on_file" as="text"**

## read_dataurl
**> browser.read_file sel="#file" then="on_file" as="data_url"**

## on_file
    + `ok`=False
    + `name`=""
    + `size`=0
    + `body`=""
    + `data_url`=""
    + `as`=""
    + `error`=""

1. `ok`
    1. `as` == "data_url"
        *`n` = > len value=data_url*
        *`ns` = > str `n`*
        *`preview` = name + " data_url chars=" + ns*
        *`ret` = > browser.set_text sel="#log" text=preview*
        1. `data_url` != ""
            *`h` = "<img alt=\"preview\" src=\"" + data_url + "\" style=\"max-width:12rem;max-height:8rem\" />"*
            *`img` = > browser.set_html sel="#preview" html=h*
            **> browser.merge a=ret b=img**
        2. *
            **ret**
    2. *
        *`n` = > len value=body*
        *`ns` = > str `n`*
        *`ss` = > str `size`*
        *`preview` = name + " (" + ss + " B, chars=" + ns + ") ok"*
        **> browser.set_text sel="#log" text=preview**
2. *
    *`preview` = "file error: " + error*
    **> browser.set_text sel="#log" text=preview**

## draw_box

Canvas commands as a readable `@` table:

`commands` =

| @ | op | x | y | w | h | fill | stroke | text |
|---|----|---|---|---|---|------|--------|------|
| 1 | fillrect | 10 | 10 | 120 | 80 | "#2f6f4e" | | |
| 2 | strokerect | 20 | 20 | 100 | 60 | | "#122018" | |
| 3 | filltext | 40 | 55 | | | "#f5faf6" | | Marqdo |

*`cv` = > browser.canvas sel="#pad" commands=commands clear=True*
*`msg` = > browser.set_text sel="#log" text="canvas drawn"*
**> browser.merge a=cv b=msg**

## play_tone
*`tone` = > browser.audio op="play" id="demo" src="beep" freq=523 ms=220*
*`msg` = > browser.set_text sel="#log" text="audio beep"*
**> browser.merge a=tone b=msg**

## start_watch
*`obs_on` = True*

`obs` =

| @ | kind | sel | id | then | threshold |
|---|------|-----|----|------|-----------|
| 1 | intersect | "#beacon" | beacon | on_intersect | 0.25 |
| 2 | resize | "#pad" | pad | on_resize | |

*`o` = > browser.observe specs=obs*
*`msg` = > browser.set_text sel="#log" text="observers on"*
**> browser.merge a=o b=msg**

## stop_watch
*`obs_on` = False*

`ids` =

| @ | id |
|---|----|
| 1 | beacon |
| 2 | pad |

*`u` = > browser.unobserve specs=ids*
*`msg` = > browser.set_text sel="#log" text="observers off"*
**> browser.merge a=u b=msg**

## on_intersect
    + `intersecting`=False
    + `ratio`=0

1. `obs_on`
    *`state` = "hidden"*
    1. `intersecting`
        *`state` = "visible"*
    2. *
        ---
    *`r` = > str `ratio`*
    *`msg` = "beacon " + state + " ratio=" + r*
    **> browser.set_text sel="#obs" text=msg**
2. *
    ****

## on_resize
    + `width`=0
    + `height`=0

1. `obs_on`
    *`w` = > str `width`*
    *`h` = > str `height`*
    *`msg` = "pad size " + w + "×" + h*
    **> browser.set_text sel="#obs" text=msg**
2. *
    ****

## on_drag
    + `drag`=""

*`msg` = "dragging: " + drag*
**> browser.set_text sel="#log" text=msg**

## on_dragover
****

## on_drop
    + `drop_text`=""

*`msg` = "dropped: " + drop_text*
*`h` = "<strong>" + drop_text + "</strong>"*
*`t` = > browser.set_text sel="#log" text=msg*
*`box` = > browser.set_html sel="#dropzone" html=h*
**> browser.merge a=t b=box**
