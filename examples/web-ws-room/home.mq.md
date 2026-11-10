---
markup: occlusion
title: Room page components
import:
  component: ext/web/component/index.mq.md
  table: lib/table.mq.md
  fs: lib/fs.mq.md
---

The parent contains an introduction and one log control. Ordinary copy lives in intro.md; the control keeps only its event target and plain initial text. Content has no transport setup, server lifetime or generated HTML.

## nodes

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Ordered introduction and log control rows |

The introduction is a terminal Markdown content block and is read within the granted site root. The event log is a preformatted native leaf with id log; incoming client messages may replace its text without changing the layout. Attribute values remain typed data until rendering, and initial text is escaped. These two independent responsibilities are merged in reading order, without a string of raw HTML or a second layout schema.

We **intro = [Read the terminal Markdown content](fs.read_text(path="intro.md"))** and **overview = [Describe the introduction as Markdown](component.markdown(text=`intro`, slot="intro"))**.

`attributes` =

| id | aria-live |
|---|---|
| log | polite |

We **log = [Describe the independent event log](component.element(kind="pre", value="Connect a client to /chat/lobby.", attrs=`attributes`, slot="log"))**.
Return *[Keep introduction and event log in reading order](table.concat(a=`overview`, b=`log`))*.
