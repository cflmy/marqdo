---
markup: occlusion
title: SVD structure demo
description: Diagonal SVD + structure SVG via ext/linalg.
import:
  la: ext/linalg/linalg.mq.md
---

# SVD from a runnable document


A diagonal matrix has singular values on the diagonal. Prefer matrix values and methods; `factorize` / `draw` stay explicit for teaching SVGs.

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

`M` =
$$
\begin{bmatrix}4&0\\0&2\end{bmatrix}
$$

We first **M = [Compute M](la.from_formula(formula=`M`))** for the next step.
We first **f = [Compute f](la.factorize(matrix=`M`, kind="svd"))** for the next step.
We first **S = [S](access(`f`, "S"))** for the next step.
We first **_ = [Compute _](la.draw(factor=`f`, kind="svd", theme="light", path="svd-structure.svg"))** for the next step.
We first **hm = [Compute hm](la.draw(factor=`M`, kind="hinton", path="svd-hinton.svg"))** for the next step.

We now **[Display this step's result](> print text=`S`)**.
We now **[Display this step's result](> print text="svd-ok")**.
