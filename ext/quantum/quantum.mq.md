---
title: ext/quantum/quantum
description: Official quantum circuit simulator (English). Gates + circuits; ABI plugin.
> lib/plugin.mq.md
> lib/sys.mq.md
> lib/json.mq.md
---

## ensure_plugin

Load the ABI v2 `quantum` plugin once.

*`p` = > plugin.native_path name=quantum *
1. `p`
  > plugin.load path=`p`
2. *
  > print text=ext/quantum: native quantum plugin not found (build marqdo_plugin_quantum or marqdo ext add quantum)
  > sys.exit code=1
****

# circuit
    + `qubits`=1

State-vector circuit (default |0…0⟩). Methods append gates and return an updated handle.

> ensure_plugin
**> quantum_circuit_new qubits=`qubits`**

## h
    + `qubit`=0

**> quantum_h circuit=`self` qubit=`qubit`**

## x
    + `qubit`=0

**> quantum_x circuit=`self` qubit=`qubit`**

## i
    + `qubit`=0

**> quantum_i circuit=`self` qubit=`qubit`**

## cx
    + `control`=0
    + `target`=1

**> quantum_cx circuit=`self` control=`control` target=`target`**

## simulate

Return state vector amplitudes `{re,im}`.

**> quantum_simulate circuit=`self`**

## probabilities

Return map of basis label → probability (qubit 0 = rightmost bit).

**> quantum_probabilities circuit=`self`**
