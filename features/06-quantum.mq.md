---
markup: occlusion
title: Quantum circuits (ext/quantum)
description: Official quantum simulator — tables, gates, Q7 density, Q8 themed SVG
import:
  quantum: ext/quantum/circuit/index.mq.md
  gate: ext/quantum/gate/index.mq.md
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

Optional package (not stdlib). Install: `marqdo ext add quantum` (or build `marqdo_plugin_quantum`).

GHZ (3 qubits): H on q0, then CX 0→1 and 0→2. Ideal |000⟩ / |111⟩ each ≈ 1/2.

Q7: Bell density, partial trace, Schmidt, `draw kind=hinton` / qsphere — `examples/quantum-entanglement/`.

Q8: `draw theme="dark"|"light"|"bw"` (default dark lab look) on circuit / probs / bloch. Rebuild plugin then `ext add` so view loads the new native lib.

`steps` =

| step | gate | qubits |
|------|------|--------|
| 1 | H | 0 |
| 2 | CX | 0,1 |
| 3 | CX | 0,2 |

We first **qc = [Compute qc](quantum.circuit(qubits=3, steps=`steps`))** for the next step.
We first **p = [Compute p](> `qc`.probabilities)** for the next step.
We first **_ = [Compute _](> `qc`.draw kind="probs" theme="dark")** for the next step.
We first **H = [Compute H](gate.gate(name="H"))** for the next step.
We first **__ = [Compute __](> `H`.draw kind="matrix")** for the next step.

We now **[Display this step's result](> print text=`p`)**.
We now **[Display this step's result](> print text="quantum-feature-ok")**.
