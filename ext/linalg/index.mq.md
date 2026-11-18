---
markup: occlusion
title: Linear algebra — readable matrix workflows
import:
  algebra: ext/linalg/linalg.mq.md
---

Start here for a complete matrix workflow, then read [the English API](linalg.mq.md) or
[the Chinese API](线性代数.mq.md). Both expose matrix construction, symbolic expressions,
dense evaluation, solving and reusable factorization. Author matrix methods require an actual
receiver; the free functions accept the expression or matrix explicitly.

Build the native dependency with cargo build -p marqdo_plugin_linalg. This offline workflow
needs plugin permission and no files, network, live services or guessed handles:

```sh
marqdo call ext/linalg/index.mq.md solve_example --effects plugin --json
```

The source matrix and right-hand side below are visible as native matrix formulas. A factorization is a native
structured result, not a string that should be parsed by callers. Reuse a factor only with
compatible dimensions. Rendering is separate: show/draw can write SVG when a path is supplied,
so production examples must state output ownership and cleanup.

Develop matrix semantics in the native linalg implementation and keep public wrappers readable.
Add a new free operation and its EN/ZH counterpart, then update object conveniences only where
useful. Verify dimensions, singular systems and the numeric tolerance appropriate to the method;
a successful call alone does not prove that a numerical result is accurate.

## solve_example

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Native solve result with x.data equal to [[2], [1]] for the displayed system |

This example solves two x plus y equals five, and x plus three y equals five. The matrix is
factored once as LU and then reused with the right-hand side. It demonstrates the real data
path without introducing a second formula parser or embedding an external language program.

`coefficients` =
$$
\begin{bmatrix}2&1\\1&3\end{bmatrix}
$$

`right_hand_side` =
$$
\begin{pmatrix}5\\5\end{pmatrix}
$$

We create **matrix = [Construct the coefficient matrix](algebra.from_formula(formula=`coefficients`))**
and **right = [Construct the right-hand-side column](algebra.from_formula(formula=`right_hand_side`))**.
Next **factor = [Factor the coefficient matrix once](algebra.factorize(matrix=`matrix`, kind="lu"))**
retains the native factor for this system. We return *[Solve using the prepared factor](algebra.solve(b=`right`, factor=`factor`))*.
The expected entries are two and one; use the solve result fields rather than interpreting a display string.
