---
title: quantum gate matrix heatmap smoke
description: gate.draw kind=matrix writes SVG heatmap.
import quantum:ext/quantum/quantum.mq.md
---

# main

*H = > quantum.gate name=H*
*img = > `H`.draw kind=matrix path=quantum-gate-heatmap.svg*
*kind = img[^kind]*

1. `kind` == matrix
  > print text=heatmap-ok
2. *
  > print text=heatmap-fail
