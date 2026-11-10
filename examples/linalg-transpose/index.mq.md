---
markup: occlusion
title: Transpose identity (formula document)
description: (AB)^T = B^T A^T as a runnable lecture fragment.
import:
  la: ext/linalg/linalg.mq.md
---

# Transpose reverses products


Declare shapes in a table, form \(P = AB\), then check \((AB)^\top = B^\top A^\top\) by running the document.

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

`shapes` =

| @ | name | rows | cols |
|---|------|------|------|
| 1 | A | 2 | 3 |
| 2 | B | 3 | 2 |

We first **env = [Compute env](la.declare(table=`shapes`))** for the next step.
We first **A = [A](access(`env`, "A"))** for the next step.
We first **B = [B](access(`env`, "B"))** for the next step.
We first **P = (`A` * `B`)** for the next step.
We first **t = [Compute t](> `P`.T_ascii)** for the next step.
We now **[Display this step's result](> print text=`t`)**.
1. `t` == "B^T*A^T"
  We now **[Display this step's result](> print text="identity-ok")**.
2. *
  We now **[Display this step's result](> print text="identity-fail")**.
