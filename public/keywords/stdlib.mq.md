---
markup: occlusion
title: Built-ins len / str / int / type / trim / split / join / at
description: Host builtins for strings and tables
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

Length and type:

We first **s = "hello"** for the next step.

We first **n = [Compute n](len(`s`))** for the next step.

We now **[Display this step's result](> print text=`n`)**.

We first **ty = [Compute ty](type(`s`))** for the next step.

We now **[Display this step's result](> print text=`ty`)**.

Trim, split, join, and index:

We first **t = [Compute t](trim(value="  hi  "))** for the next step.

We first **parts = [Compute parts](split(value="a,b", sep=","))** for the next step.

We first **mid = [Compute mid](at(value=`parts`, index=0))** for the next step.

We first **j = [Compute j](join(value=`parts`, sep="-"))** for the next step.

We now **[Display this step's result](> print text=`t`)**.

We now **[Display this step's result](> print text=`mid`)**.

We now **[Display this step's result](> print text=`j`)**.
