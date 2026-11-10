---
markup: occlusion
title: How to write a linear-algebra formula document
description: ext/linalg formula-doc — declare, infix, run-implies-correct
import:
  la: ext/linalg/linalg.mq.md
---

# Formula documents with ext/linalg


Not stdlib. Install: `marqdo ext add linalg` (or build `marqdo_plugin_linalg`).

*Idea:* if the `.mq.md` runs and the asserted form matches, the identity holds under the library rules. Algebra reads like a lecture; `factorize` / `lstsq` / `draw` stay explicit calculation steps.

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

Declare symbols in a table, multiply, transpose — `T_ascii` is one-step display (auto-simplify).

`shapes` =

| @ | name | rows | cols |
|---|------|------|------|
| 1 | A | 2 | 3 |
| 2 | B | 3 | 2 |

We first **env = [Compute env](la.declare(table=`shapes`))** for the next step.
We first **A = [A](access(`env`, "A"))** for the next step.
We first **B = [B](access(`env`, "B"))** for the next step.
We first **P = (`A` * `B`)** for the next step.
We first **t = [Compute t](> `P`.T_ascii)** for the next step.
We now **[Display this step's result](> print text=`t`)**.
1. `t` == "B^T*A^T"
  We now **[Display this step's result](> print text="formula-doc-ok")**.
2. *
  We now **[Display this step's result](> print text="formula-doc-fail")**.

We now **[Display this step's result](> print text="see-examples-linalg-transpose")**.
