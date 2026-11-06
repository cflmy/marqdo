---
markup: occlusion
title: ext/quantum/circuit
description: >-
  Quantum circuit skill — wires, step tables, and applying gates.
import:
  cap: ext/quantum/_capability/index.mq.md
  density: ext/quantum/density/index.mq.md
---

Circuit

A circuit is an ordered list of gates on n wires, starting from the all-zero
ket. You can build it with method-style steps with author method chains on this file, or
call the tools here. A steps table is a readable program. Measurement rows mark
which bits simulate.run should report; they do not collapse the ideal state
vector used by probabilities.

## For AI agents
If you were given only this file, you still know how to run it. Every ## heading
below is a tool. Execute with the shell. Always include --effects plugin.

Create a two-qubit circuit and save it as qc:

Call example (plain text):
  marqdo call ext/quantum/circuit/index.mq.md new qubits=2 --effects plugin --session s --toget qc --json

Apply Hadamard on qubit 0:

Call example (plain text):
  marqdo call ext/quantum/circuit/index.mq.md hadamard circuit=$qc qubit=0 --effects plugin --session s --toget qc --json

Apply CNOT from 0 to 1:

Call example (plain text):
  marqdo call ext/quantum/circuit/index.mq.md cnot circuit=$qc control=0 target=1 --effects plugin --session s --toget qc --json

If you were given only this file, probabilities are not a ## tool here. After the
CNOT line, call simulate on the same session (do not invent a probabilities
tool on this file):

Call example (plain text):
  marqdo call ext/quantum/simulate/index.mq.md probabilities circuit=$qc --effects plugin --session s --json

Author programs may import this file and construct the circuit object named
circuit (method chain h, cx, …). Agents should prefer marqdo call on the ##
tools above, then simulate for amplitudes or probabilities.

*None*

## new
Create a state-vector circuit of width `qubits`=1 by default. Optional
`steps`=None means no preloaded gates; when a table is provided it is parsed
like the author API.

| 参数 | 类型 | 说明 |
|------|------|------|
| qubits | number | wire count |
| steps | any | optional gate table or None |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | new circuit handle |

> cap.load
*> quantum_circuit_new qubits=`qubits` steps=`steps`*

## hadamard
Apply Hadamard on wire `qubit`=0 of an existing `circuit` by default. Prefer
this when an agent already holds a circuit from new or toget.

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | existing circuit |
| qubit | number | target wire |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | updated circuit |

> cap.load
*> quantum_h circuit=`circuit` qubit=`qubit`*

## cnot
Controlled-NOT on `circuit` from `control`=0 to `target`=1 by default. Together
with hadamard this prepares a Bell pair on two wires.

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | existing circuit |
| control | number | control wire |
| target | number | target wire |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | updated circuit |

> cap.load
*> quantum_cx circuit=`circuit` control=`control` target=`target`*

## barrier
Insert a visual barrier on `circuit`. The state does not change; drawings show a
separator so lectures can mark logical layers.

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | existing circuit |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | circuit with barrier recorded |

> cap.load
*> quantum_barrier circuit=`circuit`*

## measure_mark
Mark computational-basis readout on `circuit` for later simulate.run. This does
not collapse the ideal state used by probabilities. Pass `qubits`=None to mark
every wire.

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | existing circuit |
| qubits | any | wire list or None for all |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | circuit with measure markers |

> cap.load
*> quantum_measure circuit=`circuit` qubits=`qubits`*

# circuit
    + `qubits`=1
    + `steps`=None

State-vector circuit on n wires (default all-zero ket). Optional steps= preloads a gate table. Methods append gates and return a fresh handle for chaining. Agents should prefer the ## call tools above; authors use this object for readable method chains in programs.

> cap.load
*> quantum_circuit_new qubits=`qubits` steps=`steps`*

## h
    + `qubit`=0

Apply the Hadamard gate on the chosen wire. This creates equal superposition from computational basis states and is the first step of Bell and GHZ recipes. Returns an updated circuit handle so authors can chain without in-place mutation.

*> quantum_h circuit=`self` qubit=`qubit`*

## x
    + `qubit`=0

Apply Pauli X (bit flip) on the chosen wire. Useful for preparing |1⟩ from |0⟩ or sandwiching phase gates. Returns the updated circuit handle for further chaining.

*> quantum_x circuit=`self` qubit=`qubit`*

## y
    + `qubit`=0

Apply Pauli Y on the chosen wire. Combines bit and phase flip; returns the updated circuit handle for further chaining in author programs.

*> quantum_y circuit=`self` qubit=`qubit`*

## z
    + `qubit`=0

Apply Pauli Z (phase flip) on the chosen wire. Often used in H–Z–H sandwiches that realize an effective X. Returns the updated circuit handle.

*> quantum_z circuit=`self` qubit=`qubit`*

## s
    + `qubit`=0

Apply the phase gate S (√Z) on the chosen wire. Returns the updated circuit handle for further chaining in lecture circuits.

*> quantum_s circuit=`self` qubit=`qubit`*

## t
    + `qubit`=0

Apply the phase gate T (π/8) on the chosen wire. Returns the updated circuit handle for further chaining in lecture circuits.

*> quantum_t circuit=`self` qubit=`qubit`*

## i
    + `qubit`=0

Apply the identity on the chosen wire. Acts as a no-op spacer in step tables or noise experiments while keeping the wire explicit. Returns the updated handle.

*> quantum_i circuit=`self` qubit=`qubit`*

## rx
    + `qubit`=0
    + `theta`

Rotate the chosen wire about the X axis by angle theta in radians. Continuous single-qubit control for variational and teaching demos. Returns the updated circuit handle.

*> quantum_rx circuit=`self` qubit=`qubit` theta=`theta`*

## ry
    + `qubit`=0
    + `theta`

Rotate the chosen wire about the Y axis by angle theta in radians. Continuous single-qubit control for variational and teaching demos. Returns the updated circuit handle.

*> quantum_ry circuit=`self` qubit=`qubit` theta=`theta`*

## rz
    + `qubit`=0
    + `theta`

Rotate the chosen wire about the Z axis by angle theta in radians. Continuous single-qubit control for variational and teaching demos. Returns the updated circuit handle.

*> quantum_rz circuit=`self` qubit=`qubit` theta=`theta`*

## cx
    + `control`=0
    + `target`=1

Apply CNOT from control to target. This is the standard entangling step in Bell and GHZ preparations. Returns the updated circuit handle for chaining.

*> quantum_cx circuit=`self` control=`control` target=`target`*

## cz
    + `control`=0
    + `target`=1

Apply controlled-Z between control and target. Creates phase entanglement without flipping bits. Returns the updated circuit handle for chaining.

*> quantum_cz circuit=`self` control=`control` target=`target`*

## swap
    + `a`=0
    + `b`=1

Exchange the two wires named by a and b. Useful when routing logical qubits in a teaching layout. Returns the updated circuit handle.

*> quantum_swap circuit=`self` a=`a` b=`b`*

## barrier

Record a visual barrier only. The ideal state vector is unchanged; use barriers to separate lecture stages when drawing circuit rails.

*> quantum_barrier circuit=`self`*

## measure
    + `qubits`=None

Mark computational-basis readout wires for later run sampling. This does not collapse the ideal state used by probabilities. Pass qubits=None to mark every wire.

*> quantum_measure circuit=`self` qubits=`qubits`*

## append
    + `op`

Append another circuit’s operation list, or a gate/qubits record table, onto this circuit. Returns the combined circuit handle so programs can compose fragments.

*> quantum_append circuit=`self` op=`op`*

## apply
    + `gate`
    + `qubits`=0

Apply a gate handle (built-in name or custom unitary matrix) onto qubits given as an int or a list. Returns the updated circuit handle for further chaining.

*> quantum_apply circuit=`self` gate=`gate` qubits=`qubits`*

## noise
    + `kind`
    + `p`

Attach teaching noise of the given kind and strength for run trajectories. Ideal simulate and probabilities paths ignore this channel by design.

*> quantum_noise circuit=`self` kind=`kind` p=`p`*

## simulate

Return ideal state-vector amplitudes as re/im pairs for the current circuit. Teaching noise attached to the circuit is ignored on this ideal path.

*> quantum_simulate circuit=`self`*

## state

Return the ideal state object with the same amplitudes as simulate. Prefer this when later fidelity or density tools expect a state handle rather than raw amplitudes.

*> quantum_state circuit=`self`*

## probabilities

Map basis labels to ideal probabilities. Labels treat qubit 0 as the rightmost bit. Teaching noise is ignored so results match the pure statevector story.

*> quantum_probabilities circuit=`self`*

## run
    + `shots`=1024
    + `seed`=1

Sample computational-basis shots from this circuit. Defaults are shots=1024 and seed=1 for reproducible replays. Applies teaching noise when a noise channel was attached.

*> quantum_run circuit=`self` shots=`shots` seed=`seed`*

## draw
    + `path`=None
    + `kind`=circuit
    + `qubit`=0
    + `theme`=dark

Render an SVG figure for circuit rails, probability bars, Bloch spheres, or advanced density kinds. Theme tokens are dark, light, and bw. Optional path writes the file for CLI or view embedding.

*> quantum_draw_circuit circuit=`self` path=`path` kind=`kind` qubit=`qubit` theme=`theme`*

## density

Build the pure-state density matrix ρ=|ψ⟩⟨ψ| after ideal simulation of this circuit. Use density-module tools for mixed states and partial traces.

*> quantum_density_from_state state=`self`*

## expect
    + `obs`

Compute the expectation value of a Pauli string (left end = high bit) or an explicit matrix on this circuit’s ideal density. Teaching noise is ignored.

*> quantum_density_expect density=`self` obs=`obs`*

## schmidt
    + `cut`=1

Perform a bipartite Schmidt decomposition of the ideal state. The cut argument is the number of qubits in subsystem A counted from the low bits.

*> quantum_schmidt state=`self` cut=`cut`*
