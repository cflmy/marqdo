---
markup: occlusion
title: Built-in input
description: Read one line from stdin; optional prompt. Demo via frontmatter stdin.
stdin: Ada
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

The input builtin reads one line. For demos and marqdo view / static export, put sample lines in frontmatter (stdin:). CLI can also use a pipe or --stdin-file. Live view can override with the Preset input box.

We first **name = [Compute name](input(prompt="Name: "))** for the next step.

We now **[Display this step's result](> print text="Hello `name`!")**.
