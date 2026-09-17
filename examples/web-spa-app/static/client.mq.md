---
title: spa client
description: routes table + memory store; list / detail / edit
import browser:lib/browser.mq.md
import table:lib/table.mq.md
import text:lib/text.mq.md
---

# main

Routes (documentation + titles). Runtime switches panels by `path`.

`routes` =

| @ | 路径 | 面板 | 标题 |
|---|------|------|------|
| 1 | / | #view-list | Items |
| 2 | /item | #view-detail | Detail |
| 3 | /edit | #view-edit | Edit |

**`path` = "/"**
**`cur_id` = ""**
**`items` = "Alpha\nBeta"**

`wire` =

| @ | 选择器 | 事件 | 调用 | 委托 | 值选择器 |
|---|--------|------|------|------|----------|
| 1 | "#nav-list" | click | go_list | | |
| 2 | "#nav-new" | click | go_new | | |
| 3 | "#items" | click | open_item | "li[data-id]" | |
| 4 | "#btn-edit" | click | go_edit | | |
| 5 | "#btn-back" | click | go_list | | |
| 6 | "#btn-save" | click | save_item | | "#edit-title" |
| 7 | "#btn-cancel" | click | go_list | | |
| 8 | "window" | popstate | on_pop | | |

`seed` =

| op | key | value | scope |
|----|-----|-------|-------|
| set | spa-items | `items` | memory |

**`w` = > table.put in=None at="wire" value=wire**
**`s` = > browser.storage spec=seed**
**`boot` = > browser.merge a=w b=s**
**`ready` = > browser.set_text sel="#log" text="spa ready — click Items"**
*> browser.merge a=boot b=ready*

## on_pop
    + `path`=""

1. `path` != ""
    **`path` = path**
2. *
    ****
*> spa_render*

## go_list
**`path` = "/"**
**`cur_id` = ""**
*> spa_render*

## go_new
**`path` = "/edit"**
**`cur_id` = ""**
*> spa_render*

## go_edit
**`path` = "/edit"**
*> spa_render*

## open_item
    + `data_id`=""

1. `data_id` != ""
    **`cur_id` = data_id**
    **`path` = "/item"**
    *> spa_render*
2. *
    ****

## save_item
    + `value`=""

1. `value` != ""
    1. `cur_id` != ""
        **`parts` = > text.str_split s=items sep="\n"**
        **`next` = ""**
        - [item](parts)
            1. `item` != ""
                1. `item` == `cur_id`
                    1. `next` == ""
                        **`next` = value**
                    2. *
                        **`next` = next + "\n" + value**
                2. *
                    1. `next` == ""
                        **`next` = item**
                    2. *
                        **`next` = next + "\n" + item**
        **`items` = next**
        **`cur_id` = value**
    2. *
        1. `items` == ""
            **`items` = value**
        2. *
            **`items` = items + "\n" + value**
        **`cur_id` = value**
    **`path` = "/item"**
    *> spa_render*
2. *
    *> browser.set_text sel="#log" text="title required"*

## spa_render
*> spa_paint*

## spa_paint
**`hide` = > table.put in=None at="hidden" value=""**
**`show` = > table.put in=None at="hidden" value=False**
1. `path` == "/item"
    `attrs` =

    | #view-detail | #view-list | #view-edit |
    |--------------|------------|------------|
    | `show` | `hide` | `hide` |
    **`title` = "Detail"**
    **`body` = cur_id**
2. *
    1. `path` == "/edit"
        `attrs` =

        | #view-edit | #view-list | #view-detail |
        |------------|------------|--------------|
        | `show` | `hide` | `hide` |
        **`title` = "Edit"**
        **`body` = cur_id**
    2. *
        `attrs` =

        | #view-list | #view-detail | #view-edit |
        |------------|--------------|------------|
        | `show` | `hide` | `hide` |
        **`title` = "Items"**
        **`body` = ""**
**`parts` = > text.str_split s=items sep="\n"**
**`html` = ""**
- [item](parts)
    1. `item` != ""
        **`html` = html + "<li data-id=\"" + item + "\">" + item + "</li>"**
`persist` =

| op | key | value | scope |
|----|-----|-------|-------|
| set | spa-items | `items` | memory |

**`t` = > browser.set_text sel="#title" text=title**
**`a` = > browser.wrap key="set_attr" value=attrs**
**`ret` = > browser.merge a=t b=a**
**`list` = > browser.set_html sel="#items" html=html**
**`ret` = > browser.merge a=ret b=list**
**`d` = > browser.set_text sel="#detail-body" text=body**
**`ret` = > browser.merge a=ret b=d**
1. `path` == "/edit"
    **`v` = > browser.set_value sel="#edit-title" value=cur_id**
    **`ret` = > browser.merge a=ret b=v**
2. *
    ****
**`nav` = > browser.spa_goto url=path**
**`ret` = > browser.merge a=ret b=nav**
**`log` = > browser.set_text sel="#log" text=path**
**`ret` = > browser.merge a=ret b=log**
**`sav` = > browser.storage spec=persist**
*> browser.merge a=ret b=sav*
