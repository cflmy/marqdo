---
title: Least squares via lstsq
description: Overdetermined Ax≈b with ext/linalg lstsq + heatmap.
import la:ext/linalg/linalg.mq.md
import math:lib/math.mq.md
---

# Least squares from a runnable document

Fit a line through three points with an overdetermined `A x ≈ b` using `lstsq` (QR path). A heatmap makes the design matrix visible in `view`.

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

*`_` = > la.draw factor=`A` kind="heatmap" theme="light" path="ls-heatmap.svg"*

*`ls` = > la.lstsq a=`A` b=`b`*
*`x` = ls[^x]*
*`resid` = ls[^residual_fro]*

> print text=`x`
> print text=`resid`
> print text=ls-ok
