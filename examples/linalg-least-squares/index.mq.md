---
markup: occlusion
title: Least squares via lstsq
description: Overdetermined Ax≈b with ext/linalg lstsq + heatmap.
import:
  la: ext/linalg/linalg.mq.md
  math: lib/math.mq.md
---

# Least squares from a runnable document


Fit a line through three points with an overdetermined `A x ≈ b` using `lstsq` (QR path). A heatmap makes the design matrix visible in `view`.

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

`A` =
$$
\begin{bmatrix}1&0\\1&1\\1&2\end{bmatrix}
$$

`b` =
$$
\begin{pmatrix}1\\2\\2\end{pmatrix}
$$

We first **A = [Compute A](la.from_formula(formula=`A`))** for the next step.
We first **b = [Compute b](la.from_formula(formula=`b`))** for the next step.

We first **_ = [Compute _](la.draw(factor=`A`, kind="heatmap", theme="light", path="ls-heatmap.svg"))** for the next step.

We first **ls = [Compute ls](la.lstsq(a=`A`, b=`b`))** for the next step.
We first **x = [x](access(`ls`, "x"))** for the next step.
We first **resid = [residual_fro](access(`ls`, "residual_fro"))** for the next step.

We now **[Display this step's result](> print text=`x`)**.
We now **[Display this step's result](> print text=`resid`)**.
We now **[Display this step's result](> print text="ls-ok")**.
