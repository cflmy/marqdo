---
title: linalg basic smoke (L2)
description: from_formula, det, solve 2×2, explicit mul.
import la:ext/linalg/linalg.mq.md
import math:lib/math.mq.md
---

# main

`A` =
$$
\begin{bmatrix}2&1\\1&3\end{bmatrix}
$$

`b` =
$$
\begin{pmatrix}5\\5\end{pmatrix}
$$

*`A` = > la.from_formula formula=`A`*
*`b` = > la.from_formula formula=`b`*
*`eps` = > math.div a=1 b=1000*

*`d` = > la.det expr=`A`*
*`d0` = > math.sub a=`d` b=5*
*`d0` = > math.abs value=`d0`*
1. `d0` < `eps`
  > print text=det-ok
2. *
  > print text=det-fail

*`tr` = > la.trace expr=`A`*
*`t0` = > math.sub a=`tr` b=5*
*`t0` = > math.abs value=`t0`*
1. `t0` < `eps`
  > print text=trace-ok
2. *
  > print text=trace-fail

*`sol` = > la.solve a=`A` b=`b`*
*`x` = sol[^x]*
*`data` = x[^data]*
*`row0` = > at value=`data` index=0*
*`x0` = > at value=`row0` index=0*
*`e0` = > math.sub a=`x0` b=2*
*`e0` = > math.abs value=`e0`*
1. `e0` < `eps`
  > print text=solve-ok
2. *
  > print text=solve-fail

*`I` = > la.eye n=2*
*`P` = > la.mul a=`A` b=`I`*
*`E` = > la.explicit expr=`P`*
*`ed` = E[^data]*
*`ed0` = > at value=`ed` index=0*
*`ed00` = > at value=`ed0` index=0*
*`ee` = > math.sub a=`ed00` b=2*
*`ee` = > math.abs value=`ee`*
1. `ee` < `eps`
  > print text=explicit-ok
2. *
  > print text=explicit-fail
