---
title: GHZ state
description: 3-qubit GHZ; |000⟩/|111⟩ ≈ 1/2; probs + H matrix heatmap.
import quantum:ext/quantum/quantum.mq.md
---

# GHZ

H on qubit 0, CX(0→1), CX(0→2). Ideal: |000⟩ and |111⟩ each with probability 1/2.

# main

`steps` =

| step | gate | qubits |
|------|------|--------|
| 1 | H | 0 |
| 2 | CX | 0,1 |
| 3 | CX | 0,2 |

*`qc` = > quantum.circuit qubits=3 steps=`steps` *
*`p` = > `qc`.probabilities *
*`_` = > `qc`.draw path=ghz.svg *
*`_` = > `qc`.draw kind=probs path=ghz-probs.svg *
*`H` = > quantum.gate name=H *
*`_` = > `H`.draw kind=matrix path=ghz-h-matrix.svg *

> print text=`p`
