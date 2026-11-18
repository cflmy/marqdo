---
markup: occlusion
title: GHZ state
description: 3-qubit GHZ; |000⟩/|111⟩ ≈ 1/2; probs + H matrix heatmap.
import:
  quantum: ext/quantum/circuit/index.mq.md
  gate: ext/quantum/gate/index.mq.md
---

# GHZ


H on qubit 0, CX(0→1), CX(0→2). Ideal: |000⟩ and |111⟩ each with probability 1/2.

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

`steps` =

| step | gate | qubits |
|------|------|--------|
| 1 | H | 0 |
| 2 | CX | 0,1 |
| 3 | CX | 0,2 |

We first **qc = [Compute qc](quantum.circuit(qubits=3, steps=`steps`))** for the next step.
We first **p = [Compute p](qc.probabilities())** for the next step.
We first **_ = [Compute _](qc.draw(path="ghz.svg"))** for the next step.
We first **_ = [Compute _](qc.draw(kind="probs", path="ghz-probs.svg"))** for the next step.
We first **H = [Compute H](gate.gate(name="H"))** for the next step.
We first **_ = [Compute _](H.draw(kind="matrix", path="ghz-h-matrix.svg"))** for the next step.

We now **[Display this step's result](> print text=`p`)**.
