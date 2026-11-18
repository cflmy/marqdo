---
markup: occlusion
title: lib/table — collections
description: put, append, and map helpers (code-as-docs pilot)
import:
  table: lib/table.mq.md
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

Prefer table.put for element updates (at= text key, 1-based int, or list path).
Read with link-index get. Use json for parse/stringify only.

We first **h = [Compute h](table.put(in=None, at="Authorization", value="Bearer-demo"))** for the next step.

`xs` =

| v |
|---|
| a |
| b |

We first **xs = [Compute xs](table.put(in=`xs`, at=1, value="A"))** for the next step.
We first **n = [Compute n](table.rows(xs=`xs`))** for the next step.

**打印 内容=[Authorization](access(h, "Authorization"))**
**打印 内容=[1](access(xs, "1"))**
**打印 内容=`n`**
