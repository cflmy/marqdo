---
title: Entanglement lab
description: Bell state — density, partial trace, Schmidt, advanced SVG (Q7).
import quantum:ext/quantum/quantum.mq.md
import math:lib/math.mq.md
---

# Entanglement from a runnable document

Build |Φ⁺⟩ with H then CX. Full-state purity stays 1; tracing out one qubit yields a mixed state (purity 1/2). Schmidt coefficients and entropy quantify the bipartition. SVG plots (hinton / qsphere / multibloch) make the same facts visible.

# main

`steps` =

| step | gate | qubits |
|------|------|--------|
| 1 | H | 0 |
| 2 | CX | 0,1 |

*qc = > quantum.circuit qubits=2 steps=`steps`*
*p = > `qc`.probabilities*

*rho = > `qc`.density*
*pur = > `rho`.purity*
*red = > `rho`.partial_trace keep=0*
*rpur = > `red`.purity*

*zz = "ZZ"*
*ezz = > `qc`.expect obs=`zz`*

*sch = > `qc`.schmidt cut=1*
*ent = sch[^entropy]*

*_ = > `qc`.draw path="entangle-circuit.svg"*
*_ = > `qc`.draw kind="hinton" path="entangle-hinton.svg"*
*_ = > `qc`.draw kind="qsphere" path="entangle-qsphere.svg"*
*_ = > `qc`.draw kind="multibloch" path="entangle-multibloch.svg"*
*_ = > `rho`.draw kind="city" path="entangle-city.svg"*

> print text=`p`
> print text=`pur`
> print text=`rpur`
> print text=`ezz`
> print text=`ent`
