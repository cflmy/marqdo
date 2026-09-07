---
title: Linear algebra (ext/linalg)
description: Formula MatExpr, dense eval, factorize, structure SVG
import la:ext/linalg/linalg.mq.md
---

# main

Optional package (not stdlib). Install: `marqdo ext add linalg` (or build `marqdo_plugin_linalg`).

Formula-first: symbols stay symbolic until `explicit` / `factorize`. `(AB)^T` simplifies to `B^T*A^T` without dense data.

L4–L5: `factorize kind=eig|svd|qr|lu|chol`, `draw kind=eig|svd|heatmap|hinton`, examples `examples/linalg-svd/` · `examples/linalg-least-squares/`.

*`A` = > la.symbol name="A" rows=2 cols=2*
*`B` = > la.symbol name="B" rows=2 cols=2*
*`P` = > la.mul a=`A` b=`B`*
*`Pt` = > la.transpose expr=`P`*
*`s` = > la.simplify expr=`Pt`*
*`t` = > la.ascii expr=`s`*

`M` =
$$
\begin{bmatrix}2&0\\0&3\end{bmatrix}
$$

*`M` = > la.from_formula formula=`M`*
*`f` = > la.factorize matrix=`M` kind="eig"*
*`_` = > la.draw factor=`f` kind="eig"*

> print text=`t`
> print text=linalg-feature-ok
