---
markup: occlusion
title: ext/quantum/state
description: >-
  Quantum state skill — amplitudes, Kronecker products, fidelity, Schmidt.
import:
  cap: ext/quantum/_capability/index.mq.md
---

Pure states — compare, combine or partition a quantum state.

Obtain a state with simulate.state. A state is a map with qubits, dim and a
list of amplitudes, each carrying re and im. A raw amplitude list from
simulate.simulate is not a state handle. fidelity and schmidt also accept a
circuit and evolve its ideal state. kron requires two state handles for the
state-vector branch; convert circuits with simulate.state first.

A self-contained tensor experiment

Two zero states make a two-qubit zero state. Its fidelity with itself is one.
The first factor is the high-bit subsystem of the tensor product; the second
factor occupies the low bits. For Schmidt, cut counts the low bits in A.

```sh
marqdo call ext/quantum/circuit/index.mq.md new qubits=1 --effects plugin --session handbook --toget qc --json
marqdo call ext/quantum/simulate/index.mq.md state 'circuit=$qc' --effects plugin --session handbook --toget psi --json
marqdo call ext/quantum/state/index.mq.md kron 'a=$psi' 'b=$psi' --effects plugin --session handbook --toget product --json
marqdo call ext/quantum/state/index.mq.md fidelity 'a=$product' 'b=$product' --effects plugin --session handbook --json
marqdo call ext/quantum/state/index.mq.md schmidt 'state=$product' cut=1 --effects plugin --session handbook --json
```

fidelity is squared absolute overlap of equal-size pure states. schmidt returns
coeffs, ua, ub, entropy, cut and qubits. Entropy uses natural logarithms (nats);
a Bell pair has two coefficients near 1/√2 and entropy ln(2). Schmidt currently
shares the density solver's six-qubit limit and requires 0 < cut < qubits.
kron also accepts matrix factors and returns a matrix-bearing handle; this is
an operator product, not an automatic conversion into a normalized pure state.

Development route

Keep pure-state algebra here and mixed-state operations in
[density](../density/index.mq.md). Mathematical implementations and independent
numerical tests belong in plugins/quantum/src/linalg.rs, with exports in
src/lib.rs. Add both language contracts and preserve tests/ext/quantum-linalg-smoke.mq.md.
Use known orthogonal/product/entangled states as checks instead of comparing a
new implementation only with itself. Draw a single wire through
[viz.draw_bloch](../viz/index.mq.md) using its source circuit.

## kron

| 参数 | 类型 | 说明 |
|------|------|------|
| a | any | left factor state or matrix |
| b | any | right factor state or matrix |

The Kronecker product of `a` and `b` builds a larger space from two smaller
ones. For state vectors this writes product states; for matrices it enlarges
operators.

Pass handles the plugin already understands. For a pure-state product, convert circuits with simulate.state first. Matrix
factors take the operator branch; raw amplitude lists are not accepted as states.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | tensor product |

> cap.load
*> quantum_kron a=`a` b=`b`*

## fidelity

| 参数 | 类型 | 说明 |
|------|------|------|
| a | any | first pure state or circuit |
| b | any | second pure state or circuit |

Pure-state fidelity between `a` and `b` is the squared absolute overlap. It is
one when the states coincide and zero when they are orthogonal. You may pass a
circuit handle; the plugin uses the ideal simulated state. Teaching noise is
ignored.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | number | fidelity in the unit interval |

> cap.load
*> quantum_fidelity a=`a` b=`b`*

## schmidt

| 参数 | 类型 | 说明 |
|------|------|------|
| state | any | pure state or circuit |
| cut | number | qubits in subsystem A low bits |

Schmidt decomposition of `state` across a cut. Subsystem A owns the lowest
`cut`=1 qubits by default. Qubit 0 is the rightmost bit. Singular values show
entanglement across the bipartition.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | singular values and factors |

> cap.load
*> quantum_schmidt state=`state` cut=`cut`*
