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

Build Phi-plus with domain skills, then print ideal probabilities. Prefer the
same steps via marqdo call with --effects plugin and toget when an agent drives
the session.

**qc = > circuit.new qubits=2 steps=None**
**qc = > circuit.hadamard circuit=`qc` qubit=0**
**qc = > circuit.cnot circuit=`qc` control=0 target=1**
**p = > simulate.probabilities circuit=`qc`**
> print text=`p`
