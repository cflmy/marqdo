---
markup: occlusion
title: client counter (route D)
description: wired by client_embed auto-mount; GFM + lib/browser
import:
  browser: lib/browser.mq.md
  table: lib/table.mq.md
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | The value explicitly returned below; conditional returns follow the selected branch |

This client module starts one counter, registers bump/reset controls and returns a ready message with the wire table. The server-side page supplies #bump, #reset, #count and #log; the client does not build a second page from raw HTML.

The browser bridge applies the returned instructions and preserves state between callbacks. Initial readiness only shows that mounting occurred. Verify a click and reset separately to check the counter behavior, and keep selector names aligned with the page's Markdown view structure.

We first **count = 0** for the next step.

`wire` =

| @ | 选择器 | 事件 | 调用 |
|---|--------|------|------|
| 1 | "#bump" | click | bump |
| 2 | "#reset" | click | reset |

We first **ready = [Compute ready](browser.set_text(sel="#log", text="client.mq.md ready"))** for the next step.
We first **boot = [Compute boot](table.put(in=None, at="wire", value=`wire`))** for the next step.
After these steps, return *browser.merge(a=`boot`, b=`ready`)*.

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
