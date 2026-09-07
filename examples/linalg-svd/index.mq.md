---
title: SVD structure demo
description: Diagonal SVD + structure SVG via ext/linalg.
import la:ext/linalg/linalg.mq.md
---

# SVD from a runnable document

A diagonal matrix has singular values on the diagonal. Prefer matrix values and methods; `factorize` / `draw` stay explicit for teaching SVGs.

# main

`M` =
$$
\begin{bmatrix}4&0\\0&2\end{bmatrix}
$$

*`M` = > la.from_formula formula=`M`*
*`f` = > la.factorize matrix=`M` kind="svd"*
*`S` = f[^S]*
*`_` = > la.draw factor=`f` kind="svd" theme="light" path="svd-structure.svg"*
*`hm` = > la.draw factor=`M` kind="hinton" path="svd-hinton.svg"*

> print text=`S`
> print text=svd-ok
