---
markup: occlusion
title: ext/quantum/gate
description: >-
  Quantum gate skill — unitaries as formulas, named gates, matrix checks.
import:
  cap: ext/quantum/_capability/index.mq.md
---

Gates — define the operator before choosing the wires it acts on.

Use named for I, X, Y, Z, H, S, T, Rx, Ry, Rz, CX, CZ or SWAP. Rotations
require theta in radians. A custom gate is a square 2^k matrix satisfying
U†U = I (construction tolerance 1e-6). Cells may be real numbers or maps with
re and im fields. JSON and a bound Markdown formula reach the same constructor.

A complete custom-gate experiment

The matrix below swaps zero and one. matches must return true, and the final
probabilities must put all weight on basis one. This uses the
[circuit module](../circuit/index.mq.md) to apply the operator; constructing a
gate alone never changes a circuit. All references belong to the same session.

```sh
marqdo call ext/quantum/gate/index.mq.md custom 'matrix=[[0,1],[1,0]]' 'name="X-demo"' --effects plugin --session handbook --toget u --json
marqdo call ext/quantum/gate/index.mq.md matches 'gate=$u' 'matrix=[[0,1],[1,0]]' --effects plugin --session handbook --json
marqdo call ext/quantum/circuit/index.mq.md new qubits=1 --effects plugin --session handbook --toget qc --json
marqdo call ext/quantum/circuit/index.mq.md apply 'circuit=$qc' 'gate=$u' qubits=0 --effects plugin --session handbook --toget qc --json
marqdo call ext/quantum/simulate/index.mq.md probabilities 'circuit=$qc' --effects plugin --session handbook --json
```

Choose hadamard, pauli_x, pauli_z or cnot for a familiar constant; named covers
the other built-ins, custom accepts a formula matrix, matrix exports entries,
and matches checks that the explanation agrees with the implemented operator.
Use [viz.draw_gate](../viz/index.mq.md) for a glyph or matrix heatmap. Prefer
circuit.cnot with explicit control and target for an entangling operation.

Development route

For a new user-defined unitary, use custom and an independent expected matrix;
no Rust or ABI change is necessary. For a new built-in, update
plugins/quantum/src/sim.rs::named_gate_matrix and its numerical tests before
exposing it here and in the Chinese mirror. Preserve tests/ext/quantum-custom-gate-smoke.mq.md,
which binds the Hadamard formula directly into the author gate constructor.

## hadamard

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Hadamard gate handle |

The Hadamard gate creates equal superposition from computational basis states.
Its matrix in the computational basis is one over square-root two times the
matrix with ones on the first row and one then minus one on the second row.
So H on ket-zero yields an equal plus combination, and H on ket-one yields an
equal minus combination. This tool returns a named gate handle for later apply.

This tool takes no parameters.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[The Hadamard gate creates equal superposition from computational basis states](quantum_gate_new(name="H", theta=None, matrix=None))*.

## pauli_x

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Pauli X handle |

Pauli X is the bit-flip gate: a two-by-two matrix that swaps ket-zero and
ket-one. Use it when a lecture needs an explicit NOT on the computational basis.

This tool takes no parameters.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Pauli X is the bit-flip gate: a two-by-two matrix that swaps ket-zero and ket-one](quantum_gate_new(name="X", theta=None, matrix=None))*.

## pauli_z

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Pauli Z handle |

Pauli Z applies a phase of minus one to ket-one while leaving ket-zero alone.
Its diagonal matrix is the standard computational-basis Z.

This tool takes no parameters.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Pauli Z applies a phase of minus one to ket-one while leaving ket-zero alone](quantum_gate_new(name="Z", theta=None, matrix=None))*.

## cnot

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | CNOT gate handle |

The controlled-NOT on two qubits flips the target when the control is ket-one.
Wire order and bit-endianness follow the package convention where qubit 0 is the
rightmost label bit. Prefer applying via circuit tools with explicit control and
target indices when building lectures step by step.

This tool takes no parameters.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[The controlled-NOT on two qubits flips the target when the control is ket-one](quantum_gate_new(name="CX", theta=None, matrix=None))*.

## custom

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| matrix | any | required | unitary matrix formula or list |
| name | text | "U" | optional label defaulting to U |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | custom gate handle |

Use `matrix`, `name`="U" as the inputs for this operation.

Build a gate from an explicit unitary given as `matrix`. Authors usually bind a
formula matrix first, then pass that binding. Optional label `name`="U" marks
drawings when the caller omits a name.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Build a gate from an explicit unitary given as matrix](quantum_gate_new(name=`name`, theta=None, matrix=`matrix`))*.

## matches

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| gate | any | required | gate handle to test |
| matrix | any | required | expected unitary |
| tol | number | 0.000000001 | absolute tolerance |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | bool | True when matrices match |

Use `gate`, `matrix`, `tol`=0.000000001 as the inputs for this operation.

Teaching check: does `gate` match expected `matrix` within absolute tolerance

`tol`=0.000000001 ? Use after defining a formula so the lecture and the simulator
cannot drift apart.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Teaching check: does gate match expected matrix within absolute tolerance cannot drift apart.](quantum_gate_matches_matrix(gate=`gate`, matrix=`matrix`, tol=`tol`))*.

## named

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| name | text | "H" | Built-in gate name |
| theta | any | None | Rotation angle or None |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Gate handle for circuit.apply |

Use `name`="H", `theta`=None as the inputs for this operation.

Construct built-in `name`="H" with optional `theta`=None . Rx, Ry and Rz require
a numeric angle in radians. The returned gate can be passed to circuit.apply;
creating it never changes a circuit. Supported names are listed above.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Construct built-in name="H" with optional theta=None ](quantum_gate_new(name=`name`, theta=`theta`, matrix=None))*.

## matrix

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| gate | any | required | Gate handle |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Nested complex entries with re and im |

Use `gate` as the inputs for this operation.

Export the explicit unitary of `gate`. Each complex entry has re and im fields.
Use this to compare a formula with independent expected entries. Reading a gate
matrix neither modifies its handle nor applies the operator to any circuit.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Export the explicit unitary of gate](quantum_gate_matrix(gate=`gate`))*.

# gate

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| name | any | None | Default None |
| theta | number | None | Default None |
| matrix | any | None | Default None |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Gate handle for circuit.apply |

Use `name`=None, `theta`=None, `matrix`=None as the inputs for this operation.

Named built-in gate (name=H) or a custom unitary defined by matrix= (nested lists or a formula matrix fence). Custom gates may set name= (default U). Authors and call-style tools share this object surface.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Named built-in gate (name=H) or a custom unitary defined by matrix= (nested lists or a formula matrix fence)](quantum_gate_new(name=`name`, theta=`theta`, matrix=`matrix`))*.

## matrix

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Nested complex entries with re and im |

Return the explicit unitary matrix of this gate handle. Use it to inspect entries or to feed matches_matrix after constructing a gate from a formula.

Return *[Return the explicit unitary matrix of this gate handle. Use it to inspect entries or to feed matches_matrix after constructing a gate from a formula.](quantum_gate_matrix(gate=`self`))*.

## matches_matrix

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| matrix | any | required | Required |
| tol | any | 0.000000001 | Default 0.000000001 |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | bool | Match within absolute tolerance |

Use `matrix`, `tol`=0.000000001 as the inputs for this operation.

Check whether this gate matches the given matrix within absolute tolerance tol. Typical after building a custom gate from a Markdown formula fence in a lecture notebook.

Return *[Check whether this gate matches the given matrix within absolute tolerance tol](quantum_gate_matches_matrix(gate=`self`, matrix=`matrix`, tol=`tol`))*.

## draw

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | any | None | Default None |
| kind | any | "gate" | Default gate |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | quantum_svg with kind and svg |

Use `path`=None, `kind`="gate" as the inputs for this operation.

Draw a gate symbol or a complex-matrix heatmap (kind=gate or matrix). Optional path writes SVG to disk; plots also enter view/CLI through record_plot for the session.

Return *[Draw a gate symbol or a complex-matrix heatmap (kind=gate or matrix)](quantum_gate_draw(gate=`self`, path=`path`, kind=`kind`))*.
