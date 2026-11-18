---
markup: occlusion
title: Functions and named calls
description: Callable headings, narrative inputs and readable named calls
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

Headings expose callable definitions. The input contract sits directly below the heading, and its body names the recipient in prose; callers supply that value by name.

Call with `>` and named args `param=value`:

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
