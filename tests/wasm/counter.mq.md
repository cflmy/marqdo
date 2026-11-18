---
markup: occlusion
title: browser counter (C3)
description: wire table + browser.set_text — no json glue
import:
  browser: lib/browser.mq.md
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Browser event registration rows |

Initialize one browser counter and register two click handlers against the host page's bump and reset controls. The table is an event-routing list: each row identifies a selector, event and actual callback. Returning it does not simulate a click or increment the count.

The browser runtime preserves this instance's state between callbacks. To verify behavior, click bump and observe the displayed count, then reset it to zero. A native CLI run can inspect the registration value but does not establish that a browser applied it.

We first **count = 0** for the next step.

`wire` =

| @ | 选择器 | 事件 | 调用 |
|---|--------|------|------|
| 1 | "#bump" | click | bump |
| 2 | "#reset" | click | reset |

After these steps, return *wire*.

## bump

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | A plain set_text instruction with the next counter value |

This click callback reads the count already retained by the browser session. It takes no event input and does not ask the caller to supply the current state. After incrementing the native integer, convert it to text for the existing count target. Returning an instruction still relies on the official bridge to update the visible page.

We **count = count + 1** and **label = [Convert the updated native count to text](str(count))**.
Return *[Display this session's new count](browser.set_text(sel="#count", text=`label`))*.


## reset

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | A plain set_text instruction showing zero |

Reset changes the same session count used by bump. It takes no argument and does not create a new browser instance. The displayed text and retained integer are set together, so the next bump should show one. The callback describes a DOM effect and performs no direct server or file operation.

We **count = 0** before describing the visible result.
Return *[Show the reset state](browser.set_text(sel="#count", text="0"))*.
