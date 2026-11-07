---
markup: occlusion
title: ext/quantum/circuit
description: >-
  Quantum circuit skill — wires, step tables, and applying gates.
import:
  cap: ext/quantum/_capability/index.mq.md
  density: ext/quantum/density/index.mq.md
---

Circuit building — start here when you need a reusable quantum program.

A circuit begins in the all-zero state. Gates return a new circuit value; save
that return value to continue. Qubit zero is the rightmost basis bit. Gate
angles use radians. Width is 1–12 by default; MARQDO_QUANTUM_MAX_QUBITS can raise
or lower the limit, with a hard maximum of 16. State-vector memory grows as 2^n.

Read → execute → observe

This complete shell recipe creates a Bell pair and observes it through the
[simulation module](../simulate/index.mq.md). The two nonzero probabilities
should each be 0.5 within floating-point tolerance. Single quotes preserve the
session reference for Marqdo instead of letting the shell expand it.

```sh
marqdo call ext/quantum/circuit/index.mq.md new qubits=2 --effects plugin --session handbook --toget qc --json
marqdo call ext/quantum/circuit/index.mq.md hadamard 'circuit=$qc' qubit=0 --effects plugin --session handbook --toget qc --json
marqdo call ext/quantum/circuit/index.mq.md cnot 'circuit=$qc' control=0 target=1 --effects plugin --session handbook --toget qc --json
marqdo call ext/quantum/simulate/index.mq.md probabilities 'circuit=$qc' --effects plugin --session handbook --json
```

Which operation belongs here?

Wires or a declarative steps table: use new; then simulate for results.

Superposition or basis/phase gates: use hadamard, x, y, z, s, t, i; then simulate.probabilities.

Continuous control: use rx, ry, rz; then theta is in radians.

Entangle or route: use cnot, cz, swap; then state.schmidt.

Compose fragments or custom gates: use append, apply; then gate.custom or gate.named supplies a gate.

Mark teaching layers or readout: use barrier, measure_mark; then viz.draw_circuit or simulate.run.

The author object circuit below offers the same operations as methods. Its
simulation, density and drawing methods remain conveniences; their full call
contracts live in the corresponding modules. Do not pass an object method to
call without an explicit receiver; use the free tools above the object.

Development route

A new circuit recipe belongs in the [root index](../index.mq.md), composed from
these tools. Add a native gate only when composition and gate.custom cannot
express it: update plugins/quantum/src/sim.rs and src/lib.rs, then add the free
tool and author method here and in the Chinese mirror. Validate both backends
and the existing tests/ext/quantum-steps-smoke.mq.md author example.

## new

| 参数 | 类型 | 说明 |
|------|------|------|
| qubits | number | wire count |
| steps | any | optional gate table or None |

Create a state-vector circuit of width `qubits`=1 by default. Optional
`steps`=None means no preloaded gates; when a table is provided it is parsed
like the author API.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | new circuit handle |

> cap.load
*> quantum_circuit_new qubits=`qubits` steps=`steps`*

## hadamard

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | existing circuit |
| qubit | number | target wire |

Apply Hadamard on wire `qubit`=0 of an existing `circuit` by default. Prefer
this when an agent already holds a circuit from new or toget.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | updated circuit |

> cap.load
*> quantum_h circuit=`circuit` qubit=`qubit`*

## cnot

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | existing circuit |
| control | number | control wire |
| target | number | target wire |

Controlled-NOT on `circuit` from `control`=0 to `target`=1 by default. Together
with hadamard this prepares a Bell pair on two wires.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | updated circuit |

> cap.load
*> quantum_cx circuit=`circuit` control=`control` target=`target`*

## barrier

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | existing circuit |

Insert a visual barrier on `circuit`. The state does not change; drawings show a
separator so lectures can mark logical layers.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | circuit with barrier recorded |

> cap.load
*> quantum_barrier circuit=`circuit`*

## measure_mark

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | existing circuit |
| qubits | any | wire list or None for all |

Mark computational-basis readout on `circuit` for later simulate.run. This does
not collapse the ideal state used by probabilities. Pass `qubits`=None to mark
every wire.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | circuit with measure markers |

> cap.load
*> quantum_measure circuit=`circuit` qubits=`qubits`*

## x

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | Existing circuit; returns a new value |
| qubit | number | Default 0 (input) |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Updated circuit |

Inputs: `qubit`=0 (input).

Apply Pauli X (bit flip) on the chosen wire. Useful for preparing |1⟩ from |0⟩ or sandwiching phase gates. Returns the updated circuit handle for further chaining.

Input `circuit` is preserved; save the returned value with --toget qc in the same session to continue.

> cap.load
*> quantum_x circuit=`circuit` qubit=`qubit`*

## y

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | Existing circuit; returns a new value |
| qubit | number | Default 0 (input) |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Updated circuit |

Inputs: `qubit`=0 (input).

Apply Pauli Y on the chosen wire. Combines bit and phase flip; returns the updated circuit handle for further chaining in author programs.

Input `circuit` is preserved; save the returned value with --toget qc in the same session to continue.

> cap.load
*> quantum_y circuit=`circuit` qubit=`qubit`*

## z

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | Existing circuit; returns a new value |
| qubit | number | Default 0 (input) |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Updated circuit |

Inputs: `qubit`=0 (input).

Apply Pauli Z (phase flip) on the chosen wire. Often used in H–Z–H sandwiches that realize an effective X. Returns the updated circuit handle.

Input `circuit` is preserved; save the returned value with --toget qc in the same session to continue.

> cap.load
*> quantum_z circuit=`circuit` qubit=`qubit`*

## s

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | Existing circuit; returns a new value |
| qubit | number | Default 0 (input) |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Updated circuit |

Inputs: `qubit`=0 (input).

Apply the phase gate S (√Z) on the chosen wire. Returns the updated circuit handle for further chaining in lecture circuits.

Input `circuit` is preserved; save the returned value with --toget qc in the same session to continue.

> cap.load
*> quantum_s circuit=`circuit` qubit=`qubit`*

## t

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | Existing circuit; returns a new value |
| qubit | number | Default 0 (input) |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Updated circuit |

Inputs: `qubit`=0 (input).

Apply the phase gate T (π/8) on the chosen wire. Returns the updated circuit handle for further chaining in lecture circuits.

Input `circuit` is preserved; save the returned value with --toget qc in the same session to continue.

> cap.load
*> quantum_t circuit=`circuit` qubit=`qubit`*

## i

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | Existing circuit; returns a new value |
| qubit | number | Default 0 (input) |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Updated circuit |

Inputs: `qubit`=0 (input).

Apply the identity on the chosen wire. Acts as a no-op spacer in step tables or noise experiments while keeping the wire explicit. Returns the updated handle.

Input `circuit` is preserved; save the returned value with --toget qc in the same session to continue.

> cap.load
*> quantum_i circuit=`circuit` qubit=`qubit`*

## rx

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | Existing circuit; returns a new value |
| qubit | number | Default 0 |
| theta | number | Required |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Updated circuit |

Inputs: `qubit`=0, `theta` (input).

Rotate the chosen wire about the X axis by angle theta in radians. Continuous single-qubit control for variational and teaching demos. Returns the updated circuit handle.

Input `circuit` is preserved; save the returned value with --toget qc in the same session to continue.

> cap.load
*> quantum_rx circuit=`circuit` qubit=`qubit` theta=`theta`*

## ry

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | Existing circuit; returns a new value |
| qubit | number | Default 0 |
| theta | number | Required |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Updated circuit |

Inputs: `qubit`=0, `theta` (input).

Rotate the chosen wire about the Y axis by angle theta in radians. Continuous single-qubit control for variational and teaching demos. Returns the updated circuit handle.

Input `circuit` is preserved; save the returned value with --toget qc in the same session to continue.

> cap.load
*> quantum_ry circuit=`circuit` qubit=`qubit` theta=`theta`*

## rz

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | Existing circuit; returns a new value |
| qubit | number | Default 0 |
| theta | number | Required |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Updated circuit |

Inputs: `qubit`=0, `theta` (input).

Rotate the chosen wire about the Z axis by angle theta in radians. Continuous single-qubit control for variational and teaching demos. Returns the updated circuit handle.

Input `circuit` is preserved; save the returned value with --toget qc in the same session to continue.

> cap.load
*> quantum_rz circuit=`circuit` qubit=`qubit` theta=`theta`*

## cz

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | Existing circuit; returns a new value |
| control | number | Default 0 |
| target | number | Default 1 (input) |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Updated circuit |

Inputs: `control`=0, `target`=1 (input).

Apply controlled-Z between control and target. Creates phase entanglement without flipping bits. Returns the updated circuit handle for chaining.

Input `circuit` is preserved; save the returned value with --toget qc in the same session to continue.

> cap.load
*> quantum_cz circuit=`circuit` control=`control` target=`target`*

## swap

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | Existing circuit; returns a new value |
| a | number | Default 0 |
| b | number | Default 1 (input) |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Updated circuit |

Inputs: `a`=0, `b`=1 (input).

Exchange the two wires named by a and b. Useful when routing logical qubits in a teaching layout. Returns the updated circuit handle.

Input `circuit` is preserved; save the returned value with --toget qc in the same session to continue.

> cap.load
*> quantum_swap circuit=`circuit` a=`a` b=`b`*

## append

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | Existing circuit; returns a new value |
| op | any | Required |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Updated circuit |

Inputs: `op` (input).

Append another circuit’s operation list, or a gate/qubits record table, onto this circuit. Returns the combined circuit handle so programs can compose fragments.

Input `circuit` is preserved; save the returned value with --toget qc in the same session to continue.

> cap.load
*> quantum_append circuit=`circuit` op=`op`*

## apply

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | Existing circuit; returns a new value |
| gate | any | Required |
| qubits | any | Default 0 (input) |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Updated circuit |

Inputs: `gate`, `qubits`=0 (input).

Apply a gate handle (built-in name or custom unitary matrix) onto qubits given as an int or a list. Returns the updated circuit handle for further chaining.

Input `circuit` is preserved; save the returned value with --toget qc in the same session to continue.

> cap.load
*> quantum_apply circuit=`circuit` gate=`gate` qubits=`qubits`*

# circuit

| 参数 | 类型 | 说明 |
|------|------|------|
| qubits | number | Default 1 |
| steps | any | Default None |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | New circuit value; keep it to continue |

Inputs: `qubits`=1, `steps`=None (input).

State-vector circuit on n wires (default all-zero ket). Optional steps= preloads a gate table. Methods append gates and return a fresh handle for chaining. Agents should prefer the ## call tools above; authors use this object for readable method chains in programs.

> cap.load
*> quantum_circuit_new qubits=`qubits` steps=`steps`*

## h

| 参数 | 类型 | 说明 |
|------|------|------|
| qubit | number | Default 0 |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | New circuit value; keep it to continue |

Inputs: `qubit`=0 (input).

Apply the Hadamard gate on the chosen wire. This creates equal superposition from computational basis states and is the first step of Bell and GHZ recipes. Returns an updated circuit handle so authors can chain without in-place mutation.

*> quantum_h circuit=`self` qubit=`qubit`*

## x

| 参数 | 类型 | 说明 |
|------|------|------|
| qubit | number | Default 0 |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | New circuit value; keep it to continue |

Inputs: `qubit`=0 (input).

Apply Pauli X (bit flip) on the chosen wire. Useful for preparing |1⟩ from |0⟩ or sandwiching phase gates. Returns the updated circuit handle for further chaining.

*> quantum_x circuit=`self` qubit=`qubit`*

## y

| 参数 | 类型 | 说明 |
|------|------|------|
| qubit | number | Default 0 |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | New circuit value; keep it to continue |

Inputs: `qubit`=0 (input).

Apply Pauli Y on the chosen wire. Combines bit and phase flip; returns the updated circuit handle for further chaining in author programs.

*> quantum_y circuit=`self` qubit=`qubit`*

## z

| 参数 | 类型 | 说明 |
|------|------|------|
| qubit | number | Default 0 |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | New circuit value; keep it to continue |

Inputs: `qubit`=0 (input).

Apply Pauli Z (phase flip) on the chosen wire. Often used in H–Z–H sandwiches that realize an effective X. Returns the updated circuit handle.

*> quantum_z circuit=`self` qubit=`qubit`*

## s

| 参数 | 类型 | 说明 |
|------|------|------|
| qubit | number | Default 0 |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | New circuit value; keep it to continue |

Inputs: `qubit`=0 (input).

Apply the phase gate S (√Z) on the chosen wire. Returns the updated circuit handle for further chaining in lecture circuits.

*> quantum_s circuit=`self` qubit=`qubit`*

## t

| 参数 | 类型 | 说明 |
|------|------|------|
| qubit | number | Default 0 |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | New circuit value; keep it to continue |

Inputs: `qubit`=0 (input).

Apply the phase gate T (π/8) on the chosen wire. Returns the updated circuit handle for further chaining in lecture circuits.

*> quantum_t circuit=`self` qubit=`qubit`*

## i

| 参数 | 类型 | 说明 |
|------|------|------|
| qubit | number | Default 0 |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | New circuit value; keep it to continue |

Inputs: `qubit`=0 (input).

Apply the identity on the chosen wire. Acts as a no-op spacer in step tables or noise experiments while keeping the wire explicit. Returns the updated handle.

*> quantum_i circuit=`self` qubit=`qubit`*

## rx

| 参数 | 类型 | 说明 |
|------|------|------|
| qubit | number | Default 0 |
| theta | number | Required |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | New circuit value; keep it to continue |

Inputs: `qubit`=0, `theta` (input).

Rotate the chosen wire about the X axis by angle theta in radians. Continuous single-qubit control for variational and teaching demos. Returns the updated circuit handle.

*> quantum_rx circuit=`self` qubit=`qubit` theta=`theta`*

## ry

| 参数 | 类型 | 说明 |
|------|------|------|
| qubit | number | Default 0 |
| theta | number | Required |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | New circuit value; keep it to continue |

Inputs: `qubit`=0, `theta` (input).

Rotate the chosen wire about the Y axis by angle theta in radians. Continuous single-qubit control for variational and teaching demos. Returns the updated circuit handle.

*> quantum_ry circuit=`self` qubit=`qubit` theta=`theta`*

## rz

| 参数 | 类型 | 说明 |
|------|------|------|
| qubit | number | Default 0 |
| theta | number | Required |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | New circuit value; keep it to continue |

Inputs: `qubit`=0, `theta` (input).

Rotate the chosen wire about the Z axis by angle theta in radians. Continuous single-qubit control for variational and teaching demos. Returns the updated circuit handle.

*> quantum_rz circuit=`self` qubit=`qubit` theta=`theta`*

## cx

| 参数 | 类型 | 说明 |
|------|------|------|
| control | number | Default 0 |
| target | number | Default 1 |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | New circuit value; keep it to continue |

Inputs: `control`=0, `target`=1 (input).

Apply CNOT from control to target. This is the standard entangling step in Bell and GHZ preparations. Returns the updated circuit handle for chaining.

*> quantum_cx circuit=`self` control=`control` target=`target`*

## cz

| 参数 | 类型 | 说明 |
|------|------|------|
| control | number | Default 0 |
| target | number | Default 1 |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | New circuit value; keep it to continue |

Inputs: `control`=0, `target`=1 (input).

Apply controlled-Z between control and target. Creates phase entanglement without flipping bits. Returns the updated circuit handle for chaining.

*> quantum_cz circuit=`self` control=`control` target=`target`*

## swap

| 参数 | 类型 | 说明 |
|------|------|------|
| a | number | Default 0 |
| b | number | Default 1 |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | New circuit value; keep it to continue |

Inputs: `a`=0, `b`=1 (input).

Exchange the two wires named by a and b. Useful when routing logical qubits in a teaching layout. Returns the updated circuit handle.

*> quantum_swap circuit=`self` a=`a` b=`b`*

## barrier

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | New circuit value; keep it to continue |

Record a visual barrier only. The ideal state vector is unchanged; use barriers to separate lecture stages when drawing circuit rails.

*> quantum_barrier circuit=`self`*

## measure

| 参数 | 类型 | 说明 |
|------|------|------|
| qubits | any | Default None |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | New circuit value; keep it to continue |

Inputs: `qubits`=None (input).

Mark computational-basis readout wires for later run sampling. This does not collapse the ideal state used by probabilities. Pass qubits=None to mark every wire.

*> quantum_measure circuit=`self` qubits=`qubits`*

## append

| 参数 | 类型 | 说明 |
|------|------|------|
| op | any | Required |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | New circuit value; keep it to continue |

Inputs: `op` (input).

Append another circuit’s operation list, or a gate/qubits record table, onto this circuit. Returns the combined circuit handle so programs can compose fragments.

*> quantum_append circuit=`self` op=`op`*

## apply

| 参数 | 类型 | 说明 |
|------|------|------|
| gate | any | Required |
| qubits | any | Default 0 |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | New circuit value; keep it to continue |

Inputs: `gate`, `qubits`=0 (input).

Apply a gate handle (built-in name or custom unitary matrix) onto qubits given as an int or a list. Returns the updated circuit handle for further chaining.

*> quantum_apply circuit=`self` gate=`gate` qubits=`qubits`*

## noise

| 参数 | 类型 | 说明 |
|------|------|------|
| kind | any | Required |
| p | any | Required |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | New circuit value; keep it to continue |

Inputs: `kind`, `p` (input).

Attach teaching noise of the given kind and strength for run trajectories. Ideal simulate and probabilities paths ignore this channel by design.

*> quantum_noise circuit=`self` kind=`kind` p=`p`*

## simulate

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | List of re/im amplitudes |

Return ideal state-vector amplitudes as re/im pairs for the current circuit. Teaching noise attached to the circuit is ignored on this ideal path.

*> quantum_simulate circuit=`self`*

## state

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | quantum_state: qubits, dim, amplitudes |

Return the ideal state object with the same amplitudes as simulate. Prefer this when later fidelity or density tools expect a state handle rather than raw amplitudes.

*> quantum_state circuit=`self`*

## probabilities

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Basis label to ideal probability |

Map basis labels to ideal probabilities. Labels treat qubit 0 as the rightmost bit. Teaching noise is ignored so results match the pure statevector story.

*> quantum_probabilities circuit=`self`*

## run

| 参数 | 类型 | 说明 |
|------|------|------|
| shots | number | Default 1024 |
| seed | number | Default 1 |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Shot metadata and counts |

Inputs: `shots`=1024, `seed`=1 (input).

Sample computational-basis shots from this circuit. Defaults are shots=1024 and seed=1 for reproducible replays. Applies teaching noise when a noise channel was attached.

*> quantum_run circuit=`self` shots=`shots` seed=`seed`*

## draw

| 参数 | 类型 | 说明 |
|------|------|------|
| path | any | Default None |
| kind | any | Default circuit |
| qubit | number | Default 0 |
| theme | any | Default dark |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | quantum_svg with kind and svg |

Inputs: `path`=None, `kind`="circuit", `qubit`=0, `theme`="dark" (input).

Render an SVG figure for circuit rails, probability bars, Bloch spheres, or advanced density kinds. Theme tokens are dark, light, and bw. Optional path writes the file for CLI or view embedding.

*> quantum_draw_circuit circuit=`self` path=`path` kind=`kind` qubit=`qubit` theme=`theme`*

## density

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Density operator handle |

Build the pure-state density matrix ρ=|ψ⟩⟨ψ| after ideal simulation of this circuit. Use density-module tools for mixed states and partial traces.

*> quantum_density_from_state state=`self`*

## expect

| 参数 | 类型 | 说明 |
|------|------|------|
| obs | any | Required |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | number | Scalar observation |

Inputs: `obs` (input).

Compute the expectation value of a Pauli string (left end = high bit) or an explicit matrix on this circuit’s ideal density. Teaching noise is ignored.

*> quantum_density_expect density=`self` obs=`obs`*

## schmidt

| 参数 | 类型 | 说明 |
|------|------|------|
| cut | number | Default 1 |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Coefficients, subsystem bases and entropy |

Inputs: `cut`=1 (input).

Perform a bipartite Schmidt decomposition of the ideal state. The cut argument is the number of qubits in subsystem A counted from the low bits.

*> quantum_schmidt state=`self` cut=`cut`*
