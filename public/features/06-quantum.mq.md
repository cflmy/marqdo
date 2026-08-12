---
title: Quantum circuits (ext/quantum)
description: Official quantum simulator — tables, gates, probs/Bloch SVG
> ext/quantum/quantum.mq.md
---

# main

Optional package (not stdlib). Install: `marqdo ext add quantum` (or build `marqdo_plugin_quantum`).

Bell state: H then CX. Ideal |00⟩ / |11⟩ each ≈ 1/2. `draw kind=probs` embeds a probability bar chart in view.

`steps` =

| step | gate | qubits |
|------|------|--------|
| 1 | H | 0 |
| 2 | CX | 0,1 |

*`qc` = > quantum.circuit qubits=2 steps=`steps` *
*`p` = > `qc`.probabilities *
*`_` = > `qc`.draw kind=probs *

> print text=`p`
> print text=quantum-feature-ok
