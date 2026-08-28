---
title: ext/quantum/quantum
description: Official quantum circuit simulator (English). Gates + circuits; ABI plugin.
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
import json:lib/json.mq.md
---

## ensure_plugin

Load the ABI v2 `quantum` plugin once.

*p = > plugin.native_path name="quantum"*
1. `p`
  > plugin.load path=`p`
2. *
  > print text=ext/quantum: native quantum plugin not found (build marqdo_plugin_quantum or marqdo ext add quantum)
  > sys.exit code=1
****

## kron
    + `a`
    + `b`

Tensor product of two states or square matrices.

> ensure_plugin
**> quantum_kron a=`a` b=`b`**

## schmidt
    + `state`
    + `cut`=1

Schmidt decomposition of a pure state / circuit across `cut` (subsystem A = low qubits).

> ensure_plugin
**> quantum_schmidt state=`state` cut=`cut`**

## fidelity
    + `a`
    + `b`

Pure-state fidelity |⟨a|b⟩|².

> ensure_plugin
**> quantum_fidelity a=`a` b=`b`**

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
    + `theme`=dark

Circuit rail / probability bars / Bloch / advanced state plots (`kind=circuit|probs|bloch|hinton|city|density|paulivec|qsphere|multibloch`). Theme `dark` (default tech lab) / `light` / `bw`. Records into view/CLI plots via host `record_plot`.

**> quantum_draw_circuit circuit=`self` path=`path` kind=`kind` qubit=`qubit` theme=`theme`**

## density

Pure-state density matrix ρ=|ψ⟩⟨ψ| after ideal simulate.

**> density state=`self`**

## expect
    + `obs`

Pauli string (left = high bit) or matrix expectation on the ideal state.

**> quantum_density_expect density=`self` obs=`obs`**

## schmidt
    + `cut`=1

Schmidt / singular values across a bipartition (`cut` = qubits in subsystem A, low bits).

**> quantum_schmidt state=`self` cut=`cut`**

# gate
    + `name`=None
    + `theta`=None
    + `matrix`=None

Named built-in gate (`name=H`) or custom unitary from `matrix=` (nested list or formula matrix fence). Optional `name=` label for custom gates (default `U`).

> ensure_plugin
**> quantum_gate_new name=`name` theta=`theta` matrix=`matrix`**

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

# density
    + `state`=None
    + `matrix`=None

Density matrix from a state/circuit (`state=`) or an explicit Hermitian `matrix=` (≤6 qubits).

> ensure_plugin
1. `matrix`
  **> quantum_density_from_matrix matrix=`matrix`**
2. *
  **> quantum_density_from_state state=`state`**

## matrix

**> quantum_density_matrix density=`self`**

## purity

Tr(ρ²).

**> quantum_density_purity density=`self`**

## partial_trace
    + `keep`

Keep listed qubits (LSB=0); trace out the rest.

**> quantum_density_partial_trace density=`self` keep=`keep`**

## eig

Hermitian spectrum → `{eigenvalues, eigenvectors}`.

**> quantum_density_eig density=`self`**

## expect
    + `obs`

**> quantum_density_expect density=`self` obs=`obs`**

## draw
    + `path`=None
    + `kind`=hinton

Hinton / city / density-cells / Pauli vector SVG (`kind=hinton` or city or density or paulivec).

**> quantum_density_draw density=`self` path=`path` kind=`kind`**
