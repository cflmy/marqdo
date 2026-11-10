---
markup: occlusion
title: Utility module
description: Imported by main; italic is return; print for side effects
---

## add_one

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| n | any | required | Numeric input advanced or calculated by this example |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | The value explicitly returned below; conditional returns follow the selected branch |
Input `n`, used by the steps below.

After these steps, return *(`n` + 1)*.

## greet

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| who | any | required | Recipient name inserted into the greeting |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; this function performs the described effects instead of returning business data |
Input `who`, used by the steps below.

We now **[Display this step's result](> print text="Hello, `who`!")**.

After these steps, return *None*.
