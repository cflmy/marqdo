---
markup: occlusion
title: lib/text — strings
description: Trim, split, join (code-as-docs pilot)
import:
  text: lib/text.mq.md
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

Import lib/text. Bracket path or classic dotted calls both work.

We first **t = [Compute t](text.str_trim(s="  marqdo  "))** for the next step.
**打印 内容=`t`**

We first **parts = [Compute parts](text.str_split(s="a,b,c", sep=","))** for the next step.
We first **j = [Compute j](text.str_join(xs=`parts`, sep="-"))** for the next step.
**打印 内容=`j`**
