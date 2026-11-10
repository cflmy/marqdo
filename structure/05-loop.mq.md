---
markup: occlusion
title: Loops
description: Condition loops and table walks
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

Inside a function body, `-` starts a loop (not a parameter).

Countdown:

We first **n = 3** for the next step.

- `n` > 0
  We now **[Display this step's result](> print text=`n`)**.
  We first **n = (`n` - 1)** for the next step.

Walk a table (tables are collections):

`basket` =

| fruit |
|-------|
| apple |
| pear |

- [fruit](遍历 basket 为 fruit)
  We now **[Display this step's result](> print text="today `fruit`")**.
