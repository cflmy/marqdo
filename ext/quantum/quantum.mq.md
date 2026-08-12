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
    + `steps`=None

State-vector circuit (default |0…0⟩). Optional `steps=` gate table (column or row records). Methods append gates and return an updated handle.

> ensure_plugin
**> quantum_circuit_new qubits=`qubits` steps=`steps`**

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

## barrier

Visual separator only (no state change).

**> quantum_barrier circuit=`self`**

## measure
    + `qubits`=None

Mark computational-basis readout (`None` / empty = all qubits). Affects which bits `run` reports.

**> quantum_measure circuit=`self` qubits=`qubits`**

## append
    + `op`

Append another circuit's ops or a `{gate,qubits}` op map.

**> quantum_append circuit=`self` op=`op`**

## apply
    + `gate`
    + `qubits`=0

Apply a gate handle (named or custom `matrix=`) on `qubits` (int or list).

**> quantum_apply circuit=`self` gate=`gate` qubits=`qubits`**

## noise
    + `kind`
    + `p`

Teaching noise for `run` trajectories (`bitflip` / `depolarizing` / `amplitude_damping`). Ignored by `simulate` / `probabilities`.

**> quantum_noise circuit=`self` kind=`kind` p=`p`**

## simulate

Return state vector amplitudes `{re,im}` (ideal; no noise).

**> quantum_simulate circuit=`self`**

## state

Ideal state vector (same amplitudes as `simulate`).

**> quantum_state circuit=`self`**

## probabilities

Return map of basis label → probability (qubit 0 = rightmost bit). Ideal; ignores noise.

**> quantum_probabilities circuit=`self`**

## run
    + `shots`=1024
    + `seed`=1

Sample computational-basis shots (deterministic with `seed=`). Applies circuit `noise` if set.

**> quantum_run circuit=`self` shots=`shots` seed=`seed`**

## draw
    + `path`=None
    + `kind`=circuit
    + `qubit`=0

Circuit rail / probability bars / Bloch sphere SVG (`kind=circuit|probs|bloch`). Records into view/CLI plots via host `record_plot`.

**> quantum_draw_circuit circuit=`self` path=`path` kind=`kind` qubit=`qubit`**

# gate
    + `name`=None
    + `theta`=None
    + `matrix`=None

Named built-in gate (`name=H`) or custom unitary from `matrix=` (nested list / `$$` matrix fence). Optional `name=` label for custom gates (default `U`).

> ensure_plugin
1. `matrix`
  **> quantum_gate_from_matrix matrix=`matrix` name=`name`**
2. *
  **> quantum_gate_new name=`name` theta=`theta`**

## matrix

**> quantum_gate_matrix gate=`self`**

## matches_matrix
    + `matrix`
    + `tol`=0.000000001

**> quantum_gate_matches_matrix gate=`self` matrix=`matrix` tol=`tol`**

## draw
    + `path`=None
    + `kind`=gate

Gate glyph or complex-matrix heatmap (`kind=gate|matrix`). Records via host `record_plot`.

**> quantum_gate_draw gate=`self` path=`path` kind=`kind`**
