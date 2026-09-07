---
title: Least squares via QR
description: Overdetermined Ax≈b with ext/linalg QR + heatmap.
import la:ext/linalg/linalg.mq.md
import math:lib/math.mq.md
---

# Least squares from a runnable document

Fit a line through three points with an overdetermined `A x ≈ b`. Factor `A = QR`, form `Qᵀ b`, then solve the square upper-triangular `R`. A heatmap makes the design matrix visible in `view`.

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

*`_` = > la.draw factor=`A` kind="heatmap" path="ls-heatmap.svg"*

*`qr` = > la.factorize matrix=`A` kind="qr"*
*`Q` = qr[^Q]*
*`R` = qr[^R]*
*`Qt` = > la.transpose expr=`Q`*
*`Qtb` = > la.matmul a=`Qt` b=`b`*
*`sol` = > la.solve a=`R` b=`Qtb`*
*`x` = sol[^x]*

*`Ax` = > la.matmul a=`A` b=`x`*
*`r0` = > at value=Ax[^data] index=0*
*`ax0` = > at value=`r0` index=0*
*`diff` = > math.sub a=`ax0` b=1*
*`diff` = > math.abs value=`diff`*

> print text=`x`
> print text=`diff`
> print text=ls-ok
