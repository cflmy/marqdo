---
markup: occlusion
title: Object inheritance
description: "# Child = > Parent; methods walk the base chain"
---

# Greeter

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Object/type namespace for the child functions below; call the intended child directly |

## hello

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| who | any | required | Recipient name inserted into the greeting |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | The value explicitly returned below; conditional returns follow the selected branch |
Input `who`, used by the steps below.

We first **msg = "Hello, `who`!"** for the next step.
After these steps, return *msg*.

# Loud = > Greeter

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Object/type namespace for the child functions below; call the intended child directly |

## hello

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| who | any | required | Recipient name inserted into the greeting |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | The value explicitly returned below; conditional returns follow the selected branch |
Input `who`, used by the steps below.

We first **msg = "HELLO, `who`!"** for the next step.
After these steps, return *msg*.

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

We first **g = [Compute g](Greeter())** for the next step.
We first **m = `g`.hello(who="world")** for the next step.
We now **[Display this step's result](> print text=`m`)**.
We first **l = [Compute l](Loud())** for the next step.
We first **m2 = `l`.hello(who="world")** for the next step.
We now **[Display this step's result](> print text=`m2`)**.
