---
markup: occlusion
title: ext/quantum/gate
description: >-
  Quantum gate skill — unitaries as formulas, named gates, matrix checks.
import:
  cap: ext/quantum/_capability/index.mq.md
---

Gate

A gate is a unitary on one or more qubits. This skill writes the matrix in
lecture form, then builds a gate handle. Circuits apply gates; this page defines
what they are. Custom unitaries use the same path with a matrix argument from a
formula block or nested list.

## For AI agents
If you were given only this file, run tools with marqdo call and --effects plugin.
FUNCTION is the ## name. Example Hadamard gate handle:

Call example (plain text):
  marqdo call ext/quantum/gate/index.mq.md hadamard --effects plugin --json

Custom unitary: pass matrix= as JSON or a prior toget. matches checks a gate
against a matrix. Author programs may also use the gate object constructor on
this file.

## hadamard
The Hadamard gate creates equal superposition from computational basis states.
Its matrix in the computational basis is one over square-root two times the
matrix with ones on the first row and one then minus one on the second row.
So H on ket-zero yields an equal plus combination, and H on ket-one yields an
equal minus combination. This tool returns a named gate handle for later apply.

This tool takes no parameters.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Hadamard gate handle |

> cap.load
*> quantum_gate_new name="H" theta=None matrix=None*

## pauli_x
Pauli X is the bit-flip gate: a two-by-two matrix that swaps ket-zero and
ket-one. Use it when a lecture needs an explicit NOT on the computational basis.

This tool takes no parameters.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Pauli X handle |

> cap.load
*> quantum_gate_new name="X" theta=None matrix=None*

## pauli_z
Pauli Z applies a phase of minus one to ket-one while leaving ket-zero alone.
Its diagonal matrix is the standard computational-basis Z.

This tool takes no parameters.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Pauli Z handle |

> cap.load
*> quantum_gate_new name="Z" theta=None matrix=None*

## cnot
The controlled-NOT on two qubits flips the target when the control is ket-one.
Wire order and bit-endianness follow the package convention where qubit 0 is the
rightmost label bit. Prefer applying via circuit tools with explicit control and
target indices when building lectures step by step.

This tool takes no parameters.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | CNOT gate handle |

> cap.load
*> quantum_gate_new name="CX" theta=None matrix=None*

## custom
Build a gate from an explicit unitary given as `matrix`. Authors usually bind a
formula matrix first, then pass that binding. Optional label `name`="U" marks
drawings when the caller omits a name.

| 参数 | 类型 | 说明 |
|------|------|------|
| matrix | any | unitary matrix formula or list |
| name | text | optional label defaulting to U |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | custom gate handle |

> cap.load
*> quantum_gate_new name=`name` theta=None matrix=`matrix`*

## matches
Teaching check: does `gate` match expected `matrix` within absolute tolerance
`tol`=0.000000001? Use after defining a formula so the lecture and the simulator
cannot drift apart.

| 参数 | 类型 | 说明 |
|------|------|------|
| gate | any | gate handle to test |
| matrix | any | expected unitary |
| tol | number | absolute tolerance |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | bool | True when matrices match |

> cap.load
*> quantum_gate_matches_matrix gate=`gate` matrix=`matrix` tol=`tol`*

# gate
    + `name`=None
    + `theta`=None
    + `matrix`=None

Named built-in gate (name=H) or a custom unitary defined by matrix= (nested lists or a formula matrix fence). Custom gates may set name= (default U). Authors and call-style tools share this object surface.

> cap.load
*> quantum_gate_new name=`name` theta=`theta` matrix=`matrix`*

## matrix

Return the explicit unitary matrix of this gate handle. Use it to inspect entries or to feed matches_matrix after constructing a gate from a formula.

*> quantum_gate_matrix gate=`self`*

## matches_matrix
    + `matrix`
    + `tol`=0.000000001

Check whether this gate matches the given matrix within absolute tolerance tol. Typical after building a custom gate from a Markdown formula fence in a lecture notebook.

*> quantum_gate_matches_matrix gate=`self` matrix=`matrix` tol=`tol`*

## draw
    + `path`=None
    + `kind`=gate

Draw a gate symbol or a complex-matrix heatmap (kind=gate or matrix). Optional path writes SVG to disk; plots also enter view/CLI through record_plot for the session.

*> quantum_gate_draw gate=`self` path=`path` kind=`kind`*
