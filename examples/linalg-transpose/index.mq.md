---
title: Transpose identity (formula document)
description: (AB)^T = B^T A^T as a runnable lecture fragment.
import la:ext/linalg/linalg.mq.md
---

# Transpose reverses products

Declare shapes in a table, form \(P = AB\), then check \((AB)^\top = B^\top A^\top\) by running the document.

# main

`shapes` =

| @ | name | rows | cols |
|---|------|------|------|
| 1 | A | 2 | 3 |
| 2 | B | 3 | 2 |

*`env` = > la.declare table=`shapes`*
*`A` = env[^A]*
*`B` = env[^B]*
*`P` = `A` * `B`*
*`Pt` = > `P`.T*
*`t` = > `Pt`.ascii*
> print text=`t`
1. `t` == "B^T*A^T"
  > print text=identity-ok
2. *
  > print text=identity-fail
