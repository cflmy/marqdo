---
title: linalg metrics smoke (L6)
description: lstsq, norm, cond, rank, complex fro, themed draw.
import la:ext/linalg/linalg.mq.md
import math:lib/math.mq.md
import json:lib/json.mq.md
---

# main

`A` =
$$
\begin{bmatrix}1&0\\1&1\\1&2\end{bmatrix}
$$

`b` =
$$
\begin{pmatrix}1\\2\\2\end{pmatrix}
$$

*`A` = > la.from_formula formula=`A`*
*`b` = > la.from_formula formula=`b`*
*`eps` = > math.div a=1 b=1000*

*`ls` = > la.lstsq a=`A` b=`b`*
*`x` = ls[^x]*
*`xd` = x[^data]*
*`r0` = > at value=`xd` index=0*
*`x0` = > at value=`r0` index=0*
*`e0` = > math.sub a=`x0` b=1.1666666667*
*`e0` = > math.abs value=`e0`*
1. `e0` < `eps`
  > print text=lstsq-ok
2. *
  > print text=lstsq-fail

*`nf` = > la.norm expr=`A` ord="fro"*
*`n0` = > math.sub a=`nf` b=2.8284271247*
*`n0` = > math.abs value=`n0`*
1. `n0` < `eps`
  > print text=norm-ok
2. *
  > print text=norm-fail

`D` =
$$
\begin{bmatrix}2&0\\0&1\end{bmatrix}
$$

*`D` = > la.from_formula formula=`D`*
*`c` = > la.cond expr=`D`*
*`cv` = c[^value]*
*`ce` = > math.sub a=`cv` b=2*
*`ce` = > math.abs value=`ce`*
1. `ce` < `eps`
  > print text=cond-ok
2. *
  > print text=cond-fail

*`rk` = > la.rank expr=`D`*
1. `rk` == 2
  > print text=rank-ok
2. *
  > print text=rank-fail

*`raw` = > json.parse text="[[[0,1],[1,0]],[[1,0],[0,-1]]]"*
*`Z` = > la.from_list data=`raw`*
*`dt` = Z[^dtype]*
1. `dt` == complex
  > print text=complex-dtype-ok
2. *
  > print text=complex-dtype-fail

*`zn` = > la.norm expr=`Z` ord="fro"*
*`ze` = > math.sub a=`zn` b=2*
*`ze` = > math.abs value=`ze`*
1. `ze` < `eps`
  > print text=complex-norm-ok
2. *
  > print text=complex-norm-fail

*`img` = > la.draw factor=`D` kind="heatmap" theme="dark" path="linalg-metrics-heat.svg"*
*`th` = img[^theme]*
1. `th` == dark
  > print text=theme-ok
2. *
  > print text=theme-fail

*`svg` = img[^svg]*
*`parts` = > split value=`svg` sep="data-theme=\"dark\""*
*`pn` = > len value=`parts`*
1. `pn` > 1
  > print text=theme-mark-ok
2. *
  > print text=theme-mark-fail
