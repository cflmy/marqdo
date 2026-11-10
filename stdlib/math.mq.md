---
markup: occlusion
title: lib/math — numbers, formulas, plots
description: High-school math stdlib (English)
import:
  math: lib/math.mq.md
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

Import lib/math.mq.md. Runtime types: num (float) and formula (symbolic tree). Bind a formula with an empty-RHS assign followed by a `$$…$$` fence (same shape as table assigns). In view, Structure renders that fence with KaTeX.

`f` =
$$
x^2 - 2
$$

Numeric helpers: pi, e, num, add/sub/mul/div/pow, sin/cos/tan, sqrt/abs/ln/exp, floor/ceil, min/max. Random: seed, random, random_int (reproducible after seed).

Symbolic: formula(text), simplify, expand, diff, subs, eval, solve.

Plotting (already in the library): plot / plot_points / plot_conic return SVG text. Axes include arrows and tick labels; grid is on by default (pass grid=False / grid=假 to hide). Optional path= writes a file under the sandbox. With no path, CLI auto-writes stem-plot-n.svg and prints plot: …; view embeds the SVG under Execution.

diff / simplify / expand / partial subs return formula values so you can chain (e.g. differentiate twice, then plot or eval).

We first **ty = [Compute ty](type(`f`))** for the next step.

We now **[Display this step's result](> print text=`ty`)**.

We first **df = [Compute df](math.diff(formula=`f`, var="x"))** for the next step.

We now **[Display this step's result](> print text=`df`)**.

We first **roots = [Compute roots](math.solve(formula=`f`, var="x"))** for the next step.

We now **[Display this step's result](> print text=`roots`)**.

We first **_svg = [Compute _svg](math.plot(formula=`f`, var="x", min=-3, max=3))** for the next step.

We now **[Display this step's result](> print text="plot-ok")**.
