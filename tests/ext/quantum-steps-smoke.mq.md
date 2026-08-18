---
title: quantum steps + draw smoke
description: steps= Bell table + draw SVG via record_plot.
import quantum:ext/quantum/quantum.mq.md
import math:lib/math.mq.md
---

# main

`steps` =

| step | gate | qubits |
|------|------|--------|
| 1 | H | 0 |
| 2 | CX | 0,1 |

*qc = > quantum.circuit qubits=2 steps=`steps`*
*p = > `qc`.probabilities*
*p00 = p[^00]*
*lo = > math.div a=2 b=5*
*hi = > math.div a=3 b=5*

1. `p00` > `lo`
  1. `p00` < `hi`
    > print text=steps-ok
  2. *
    > print text=steps-fail
2. *
  > print text=steps-fail

*img = > `qc`.draw path="quantum-steps-draw.svg"*
*kind = img[^kind]*

1. `kind` == circuit
  > print text=draw-ok
2. *
  > print text=draw-fail
