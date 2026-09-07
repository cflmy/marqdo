---
title: linalg infix and matrix methods
description: A+B / A*B dispatch; `A`.T method chain.
import la:ext/linalg/linalg.mq.md
---

# main

*`A` = > la.symbol name="A" rows=2 cols=2*
*`B` = > la.symbol name="B" rows=2 cols=2*
*`C` = `A` + `B`*
*`ct` = > la.ascii expr=`C`*
> print text=`ct`

*`P` = `A` * `B`*
*`Pt` = > `P`.T*
*`s` = > `Pt`.simplify*
*`st` = > `s`.ascii*
> print text=`st`

*`ty` = C[^_type]*
1. `ty` == matrix
  > print text=type-ok
2. *
  > print text=type-fail

`M` =
$$
\begin{bmatrix}1&0\\0&1\end{bmatrix}
$$

`N` =
$$
\begin{bmatrix}2&0\\0&3\end{bmatrix}
$$

*`S` = `M` + `N`*
*`Se` = > la.explicit expr=`S`*
*`d` = Se[^data]*
*`r0` = > at value=`d` index=0*
*`x00` = > at value=`r0` index=0*
*`xs` = > str value=`x00`*
1. `xs` == "3"
  > print text=formula-add-ok
2. *
  > print text=formula-add-fail
