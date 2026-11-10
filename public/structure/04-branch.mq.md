---
markup: occlusion
title: Branch
description: Ordered branches select one case and retain an explicit fallback
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

Bold markers execute statements and single italics return a value. Ordered list items select a branch; the final asterisk condition is the fallback when earlier conditions do not match.

We first **x = 0** for the next step.

1. `x` > 0
  We now **[Display this step's result](> print text="positive")**.
2. `x` < 0
  We now **[Display this step's result](> print text="negative")**.
3. *
  We now **[Display this step's result](> print text="zero")**.
