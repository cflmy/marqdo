---
title: linalg formula-doc smoke (F1)
description: Run-implies-correct — (AB)^T ascii without explicit simplify.
import la:ext/linalg/linalg.mq.md
---

# Transpose reverses products

Narration: \((AB)^\top = B^\top A^\top\). The identity is checked by running this file.

# main

*`A` = > la.symbol name="A" rows=2 cols=3*
*`B` = > la.symbol name="B" rows=3 cols=2*
*`P` = `A` * `B`*
*`Pt` = > `P`.T*
*`t` = > `Pt`.ascii*
> print text=`t`
1. `t` == "B^T*A^T"
  > print text=identity-ok
2. *
  > print text=identity-fail

`shapes` =

| @ | name | rows | cols |
|---|------|------|------|
| 1 | X | 2 | 2 |
| 2 | Y | 2 | 2 |

*`env` = > la.declare table=`shapes`*
*`X` = env[^X]*
*`Y` = env[^Y]*
*`S` = `X` + `Y`*
*`st` = > la.ascii expr=`S`*
1. `st` == "X + Y"
  > print text=declare-ok
2. *
  > print text=declare-fail

*`raw` = > la.ascii expr=`Pt` raw=True*
1. `raw` == "(A*B)^T"
  > print text=raw-ok
2. *
  > print text=raw-fail
