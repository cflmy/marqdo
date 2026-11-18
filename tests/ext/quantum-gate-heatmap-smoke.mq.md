---
markup: occlusion
title: quantum gate matrix heatmap smoke
description: gate.draw kind=matrix writes SVG heatmap.
import:
  quantum: ext/quantum/circuit/index.mq.md
  gate: ext/quantum/gate/index.mq.md
---

# main

**H = > gate.gate name="H"**
**img = > `H`.draw kind="matrix" path="quantum-gate-heatmap.svg"**
**kind = [kind](access(img, "kind"))**

1. `kind` == matrix
  > print text=heatmap-ok
2. *
  > print text=heatmap-fail
