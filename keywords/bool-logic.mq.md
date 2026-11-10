---
markup: occlusion
title: Booleans and logic
description: True / False / None and and / or / not
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

Keywords: `True`, `False`, `None`, and `and` / `or` / `not`.

We first **a = True** for the next step.
We first **b = False** for the next step.

1. `a` and not `b`
  We now **[Display this step's result](> print text="ok")**.
2. *
  We now **[Display this step's result](> print text="fail")**.

We first **probe = 1** for the next step.

1. None
  We now **[Display this step's result](> print text="none-truthy")**.
2. *
  We now **[Display this step's result](> print text="none-falsy")**.
