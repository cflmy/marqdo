---
markup: occlusion
title: ext/quantum/density
description: >-
  Density-operator skill — mixed states, partial trace, expectations.
import:
  cap: ext/quantum/_capability/index.mq.md
---

Density

A density operator describes pure and mixed states together. Pure states have
purity one; mixing lowers purity. Build rho from a circuit or state, or from an
explicit Hermitian matrix. Partial trace discards subsystems. Expectation values
take Pauli strings or matrices.

## For AI agents
If you were given only this file, build or inspect density operators with marqdo
call and --effects plugin.

Call example (plain text):
  marqdo call ext/quantum/density/index.mq.md from_state state=$qc --effects plugin --toget rho --json
  marqdo call ext/quantum/density/index.mq.md purity density=$rho --effects plugin --json

Author programs may use the density object on this file.

## from_state
Form rho as the outer product of a pure `state` (or the ideal state of a
circuit). Use this when you trust the pure-state picture and want mixed-state
tools without writing a matrix by hand.

| 参数 | 类型 | 说明 |
|------|------|------|
| state | any | pure state or circuit |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | density handle |

> cap.load
*> quantum_density_from_state state=`state`*

## from_matrix
Build rho from an explicit Hermitian `matrix`. Pair with formula blocks in the
lecture so the matrix on the page is the matrix in the simulator. Size is
limited by the plugin.

| 参数 | 类型 | 说明 |
|------|------|------|
| matrix | any | Hermitian matrix |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | density handle |

> cap.load
*> quantum_density_from_matrix matrix=`matrix`*

## purity
Compute the trace of rho squared for `density`. Values near one mean nearly
pure; lower values mean classical mixing or traced-out entanglement.

| 参数 | 类型 | 说明 |
|------|------|------|
| density | any | density operator |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | number | purity |

> cap.load
*> quantum_density_purity density=`density`*

## partial_trace
Keep listed qubits in `keep` and trace out the rest of `density`. Indices use
LSB equals zero. The result is generally mixed even when the joint state was
pure.

| 参数 | 类型 | 说明 |
|------|------|------|
| density | any | joint density |
| keep | any | qubit index list to keep |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | reduced density |

> cap.load
*> quantum_density_partial_trace density=`density` keep=`keep`*

## expect
Expectation of observable `obs` on `density`. Pauli strings use left equals
high bit. Pass a matrix when the observable is not a Pauli product.

| 参数 | 类型 | 说明 |
|------|------|------|
| density | any | density operator |
| obs | any | Pauli string or matrix |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | number | expectation value |

> cap.load
*> quantum_density_expect density=`density` obs=`obs`*

## eig
Hermitian spectrum of `density` as eigenvalues and eigenvectors. Useful when a
lecture needs the eigenbasis of rho.

| 参数 | 类型 | 说明 |
|------|------|------|
| density | any | Hermitian density |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | eigenvalues and eigenvectors |

> cap.load
*> quantum_density_eig density=`density`*

# density
    + `state`=None
    + `matrix`=None

Construct a density operator from a pure state or circuit (state=) or from an explicit Hermitian matrix=. Density operations support at most six qubits. Pure and mixed states share this object surface for purity, partial trace, and expectation.

> cap.load
1. `matrix`
  *> quantum_density_from_matrix matrix=`matrix`*
2. *
  *> quantum_density_from_state state=`state`*

## matrix

Return the explicit matrix of this density operator so authors can inspect entries or hand the array to further linear-algebra steps outside the quantum package.

*> quantum_density_matrix density=`self`*

## purity

Compute Tr(ρ²). Values near one mean a pure state; mixed states are strictly less than one. Use this as a quick entanglement or decoherence check after a partial trace.

*> quantum_density_purity density=`self`*

## partial_trace
    + `keep`

Trace out every qubit except those listed in keep (LSB index 0). Returns the reduced density on the kept subsystem for entanglement lectures and purity checks.

*> quantum_density_partial_trace density=`self` keep=`keep`*

## eig

Compute the Hermitian spectrum of this density operator as eigenvalues and eigenvectors. Handy for lecture plots and for verifying that a constructed rho is physical.

*> quantum_density_eig density=`self`*

## expect
    + `obs`

Expectation value of observable obs on this density. obs may be a Pauli string with the left end as the high bit, or an explicit matrix matching the density dimension.

*> quantum_density_expect density=`self` obs=`obs`*

## draw
    + `path`=None
    + `kind`=hinton

Draw a Hinton diagram, city plot, density grid, or Pauli vector for this density. Optional path writes SVG; kind selects which visual metaphor the lecture uses.

*> quantum_density_draw density=`self` path=`path` kind=`kind`*
