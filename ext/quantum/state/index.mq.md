---
markup: occlusion
title: ext/quantum/state
description: >-
  Quantum state skill — amplitudes, Kronecker products, fidelity, Schmidt.
import:
  cap: ext/quantum/_capability/index.mq.md
---

State

A pure quantum state on n qubits is a unit vector in a complex space of size
two to the n. In this package each amplitude is a map with re and im fields.
Qubit 0 is the rightmost bit in basis labels. Ideal simulation ignores teaching
noise; sampled trajectories live under the simulate skill.

States often arrive as the result of simulate tools on a circuit. This module
answers state-to-state questions: tensor products, fidelity, and Schmidt cuts.

## For AI agents

If you were given only this file, run state tools with marqdo call and --effects
plugin. Example fidelity:

Call example (plain text):
  marqdo call ext/quantum/state/index.mq.md fidelity a=$psi b=$phi --effects plugin --json

Tools: kron, fidelity, schmidt.

## kron

The Kronecker product of `a` and `b` builds a larger space from two smaller
ones. For state vectors this writes product states; for matrices it enlarges
operators.

| 参数 | 类型 | 说明 |
|------|------|------|
| a | any | left factor state or matrix |
| b | any | right factor state or matrix |

Pass handles the plugin already understands. Circuit and density objects may
participate through their registered ABI shapes when the simulator accepts them.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | tensor product |

> cap.load
*> quantum_kron a=`a` b=`b`*

## fidelity

Pure-state fidelity between `a` and `b` is the squared absolute overlap. It is
one when the states coincide and zero when they are orthogonal. You may pass a
circuit handle; the plugin uses the ideal simulated state. Teaching noise is
ignored.

| 参数 | 类型 | 说明 |
|------|------|------|
| a | any | first pure state or circuit |
| b | any | second pure state or circuit |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | number | fidelity in the unit interval |

> cap.load
*> quantum_fidelity a=`a` b=`b`*

## schmidt

Schmidt decomposition of `state` across a cut. Subsystem A owns the lowest
`cut`=1 qubits by default. Qubit 0 is the rightmost bit. Singular values show
entanglement across the bipartition.

| 参数 | 类型 | 说明 |
|------|------|------|
| state | any | pure state or circuit |
| cut | number | qubits in subsystem A low bits |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | singular values and factors |

> cap.load
*> quantum_schmidt state=`state` cut=`cut`*
