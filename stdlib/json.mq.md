---
markup: occlusion
title: lib/json — JSON
description: Shared EN/ZH path; parse and stringify (code-as-docs pilot)
import:
  json: lib/json.mq.md
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

Shared JSON helpers for EN and ZH docs. Prefer parse, stringify, and quote.
Build maps with tables or table.put — not json.set chains.

We first **obj = [Compute obj](json.parse(text="{}"))** for the next step.
We first **ty = [Compute ty](type(`obj`))** for the next step.
**打印 内容=`ty`**

We first **out = [Compute out](json.stringify(value=`obj`))** for the next step.
**打印 内容=`out`**
