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

## y
    + `qubit`=0

**> quantum_y circuit=`self` qubit=`qubit`**

## z
    + `qubit`=0

**> quantum_z circuit=`self` qubit=`qubit`**

## s
    + `qubit`=0

**> quantum_s circuit=`self` qubit=`qubit`**

## t
    + `qubit`=0

**> quantum_t circuit=`self` qubit=`qubit`**

## i
    + `qubit`=0

**> quantum_i circuit=`self` qubit=`qubit`**

## rx
    + `qubit`=0
    + `theta`

**> quantum_rx circuit=`self` qubit=`qubit` theta=`theta`**

## ry
    + `qubit`=0
    + `theta`

**> quantum_ry circuit=`self` qubit=`qubit` theta=`theta`**

## rz
    + `qubit`=0
    + `theta`

**> quantum_rz circuit=`self` qubit=`qubit` theta=`theta`**

## cx
    + `control`=0
    + `target`=1

**> quantum_cx circuit=`self` control=`control` target=`target`**

## cz
    + `control`=0
    + `target`=1

**> quantum_cz circuit=`self` control=`control` target=`target`**

## swap
    + `a`=0
    + `b`=1

**> quantum_swap circuit=`self` a=`a` b=`b`**

## simulate

Return state vector amplitudes `{re,im}`.

**> quantum_simulate circuit=`self`**

## probabilities

Return map of basis label → probability (qubit 0 = rightmost bit).

**> quantum_probabilities circuit=`self`**

## run
    + `shots`=1024
    + `seed`=1

Sample computational-basis shots (deterministic with `seed=`).

**> quantum_run circuit=`self` shots=`shots` seed=`seed`**
