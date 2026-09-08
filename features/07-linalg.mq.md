---
title: How to write a linear-algebra formula document
description: ext/linalg formula-doc — declare, infix, run-implies-correct
import la:ext/linalg/linalg.mq.md
---

# Formula documents with ext/linalg

Not stdlib. Install: `marqdo ext add linalg` (or build `marqdo_plugin_linalg`).

**Idea:** if the `.mq.md` runs and the asserted form matches, the identity holds under the library rules. Algebra reads like a lecture; `factorize` / `lstsq` / `draw` stay explicit calculation steps.

# main

Declare symbols in a table, multiply, transpose — `T_ascii` is one-step display (auto-simplify).

`shapes` =

| @ | name | rows | cols |
|---|------|------|------|
| 1 | A | 2 | 3 |
| 2 | B | 3 | 2 |

*`env` = > la.declare table=`shapes`*
*`A` = env[^A]*
*`B` = env[^B]*
*`P` = `A` * `B`*
*`t` = > `P`.T_ascii*
> print text=`t`
1. `t` == "B^T*A^T"
  > print text=formula-doc-ok
2. *
  > print text=formula-doc-fail

> print text=see-examples-linalg-transpose
