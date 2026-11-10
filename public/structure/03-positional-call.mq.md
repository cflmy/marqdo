---
markup: occlusion
title: Positional arguments
description: Bind by parameter order; named form still works
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

Positional args bind in declaration order (no `who=` needed):

We now **[Call greet](> greet "Marqdo")**.

Named form still works:

We now **[Call greet](> greet who="World")**.

## greet

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| who | any | required | Recipient name inserted into the greeting |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; display the computed content on stdout |
Input `who`, used by the steps below.

We now **[Display this step's result](> print text="Hello `who`!")**.
