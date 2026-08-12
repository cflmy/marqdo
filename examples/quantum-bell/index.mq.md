---
title: Bell state
description: H then CX; probabilities ≈ 1/2; circuit SVG.
> ext/quantum/quantum.mq.md
> lib/math.mq.md
> lib/json.mq.md
---

# Bell |Φ+⟩

H on qubit 0, then CNOT(0→1). Ideal measurement: |00⟩ and |11⟩ each with probability 1/2.

# main

`steps` =

| step | gate | qubits |
|------|------|--------|
| 1 | H | 0 |
| 2 | CX | 0,1 |

*`qc` = > quantum.circuit qubits=2 steps=`steps` *
*`p` = > `qc`.probabilities *
*`_` = > `qc`.draw path=bell.svg *

> print text=`p`
