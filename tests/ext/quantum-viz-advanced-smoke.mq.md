---
title: quantum viz advanced smoke
description: Q7b hinton / qsphere / multibloch SVG markers
import quantum:ext/quantum/quantum.mq.md
---

# main

*qc = > quantum.circuit qubits=2*
*qc = > `qc`.h qubit=0*
*qc = > `qc`.cx control=0 target=1*

*h = > `qc`.draw kind="hinton" path="quantum-viz-hinton.svg"*
*q = > `qc`.draw kind="qsphere" path="quantum-viz-qsphere.svg"*
*m = > `qc`.draw kind="multibloch" path="quantum-viz-multibloch.svg"*

*rho = > `qc`.density*
*c = > `rho`.draw kind="city" path="quantum-viz-city.svg"*

> print text=viz-ok
