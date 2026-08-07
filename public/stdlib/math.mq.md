---
title: lib/math — numbers, formulas, plots
description: High-school math stdlib (English)
> lib/math.mq.md
---

# main

Import lib/math.mq.md. Runtime types: num (float) and formula (symbolic tree). Bind a formula with an empty-RHS assign followed by a `$$…$$` fence (same shape as table assigns). In view, Structure renders that fence with KaTeX.

`f` =
$$
x^2 - 2
$$

Numeric helpers: pi, e, num, add/sub/mul/div/pow, sin/cos/tan, sqrt/abs/ln/exp, floor/ceil, min/max. Random: seed, random, random_int (reproducible after seed).

Symbolic: formula(text), simplify, expand, diff, subs, eval, solve.

Plotting (already in the library): plot / plot_points / plot_conic return SVG text. Axes include arrows and tick labels; **grid is on by default** (pass grid=False / grid=假 to hide). Optional path= writes a file under the sandbox. With no path, CLI auto-writes stem-plot-n.svg and prints plot: …; view embeds the SVG under Execution.

diff / simplify / expand / partial subs return formula values so you can chain (e.g. differentiate twice, then plot or eval).

*`ty` = > type `f` *

> print text=`ty`

*`df` = > math.diff formula=`f` var=x *

> print text=`df`

*`roots` = > math.solve formula=`f` var=x *

> print text=`roots`

*`_svg` = > math.plot formula=`f` var=x min=-3 max=3 *

> print text=plot-ok
