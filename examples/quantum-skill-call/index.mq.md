---
markup: occlusion
title: Quantum skill call demo
description: >-
  Agent-style flow via ext/quantum index and domain modules (Bell probabilities).
import:
  circuit: ext/quantum/circuit/index.mq.md
  simulate: ext/quantum/simulate/index.mq.md
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

Build Phi-plus with domain skills, then print ideal probabilities. Prefer the
same steps via marqdo call with --effects plugin and toget when an agent drives
the session.

We first **qc = [Compute qc](circuit.new(qubits=2, steps=None))** for the next step.
We first **qc = [Compute qc](circuit.hadamard(circuit=`qc`, qubit=0))** for the next step.
We first **qc = [Compute qc](circuit.cnot(circuit=`qc`, control=0, target=1))** for the next step.
We first **p = [Compute p](simulate.probabilities(circuit=`qc`))** for the next step.
We now **[Display this step's result](> print text=`p`)**.
