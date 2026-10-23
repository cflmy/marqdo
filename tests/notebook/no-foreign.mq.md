---
title: View without foreign
description: A pure Marqdo document — formula, table, plot and prose, no external kernel
import:
  math: lib/math.mq.md
---

# main

This page has no foreign import and no ` ```lang ` code cell. It is still opened by the same `view` surface, with the same toolbar and layout. The cell list is simply empty.

The formula is ordinary Marqdo data:

$$
% marqdo: var=f
x^2 - 2
$$

**_plot = math.plot formula=`f` var="x" min=-4 max=4**

| region | orders | revenue |
|--------|--------|---------|
| 华东 | 45 | 56253 |
| 华南 | 45 | 67187 |
| 华北 | 45 | 65272 |
| 西部 | 45 | 66884 |

**total = 56253 + 67187 + 65272 + 66884**

> print text=`total`

This is the control page: no foreign, no cells, no separate mode. The same view shell is used.
