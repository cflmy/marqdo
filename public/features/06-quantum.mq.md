---
title: Quantum circuits (ext/quantum)
description: Official quantum simulator — tables, gates, Q7 density and advanced SVG
import quantum:ext/quantum/quantum.mq.md
---

# main

Optional package (not stdlib). Install: `marqdo ext add quantum` (or build `marqdo_plugin_quantum`).

GHZ (3 qubits): H on q0, then CX 0→1 and 0→2. Ideal |000⟩ / |111⟩ each ≈ 1/2.

Q7: Bell density, partial trace (mixed reduced state), Schmidt entropy, and `draw kind=hinton` / qsphere. See `examples/quantum-entanglement/`.

`steps` =

| step | gate | qubits |
|------|------|--------|
| 1 | H | 0 |
| 2 | CX | 0,1 |
| 3 | CX | 0,2 |

*`qc` = > quantum.circuit qubits=3 steps=`steps`*
*`p` = > `qc`.probabilities*
*`_` = > `qc`.draw kind="probs"*
*`H` = > quantum.gate name="H"*
*`__` = > `H`.draw kind="matrix"*

> print text=`p`
> print text=quantum-feature-ok
