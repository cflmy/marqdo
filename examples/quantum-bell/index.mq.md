---
markup: occlusion
title: Bell state
description: H then CX; probabilities ≈ 1/2; circuit SVG.
import:
  quantum: ext/quantum/circuit/index.mq.md
  math: lib/math.mq.md
  json: lib/json.mq.md
---

# Bell |Φ+⟩


H on qubit 0, then CNOT(0→1). Ideal measurement: |00⟩ and |11⟩ each with probability 1/2.

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

`steps` =

| step | gate | qubits |
|------|------|--------|
| 1 | H | 0 |
| 2 | CX | 0,1 |

We first **qc = [Compute qc](quantum.circuit(qubits=2, steps=`steps`))** for the next step.
We first **p = [Compute p](qc.probabilities())** for the next step.
We first **_ = [Compute _](qc.draw(path="bell.svg"))** for the next step.
We first **_ = [Compute _](qc.draw(kind="probs", path="bell-probs.svg"))** for the next step.

We now **[Display this step's result](> print text=`p`)**.
