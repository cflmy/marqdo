---
markup: occlusion
title: About panel Markdown leaf
import:
  component: ext/web/component/index.mq.md
  fs: lib/fs.mq.md
---

## nodes

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Structured View rows for this component |

About is a terminal Markdown leaf, so further layout splitting would add no useful responsibility. Read its content through the native filesystem under the granted site root. The wrapper supplies only its client visibility id and initial hidden state. The page describes official host capabilities without embedding implementation JavaScript or an HTML shell.

We **source = [Read the literal About content](fs.read_text(path="content/about.md"))** and **children = [Keep it as ordinary Markdown](component.markdown(text=`source`, slot="about-content"))**.

`attrs` =

| id | hidden |
|---|---|
| panel-about | True |

Return *[Describe the initially hidden About panel](component.wrap(children=`children`, kind="section", attrs=`attrs`, slot="about"))*.
