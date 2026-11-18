---
markup: occlusion
title: ext/quantum/density
description: >-
  Density-operator skill — mixed states, partial trace, expectations.
import:
  cap: ext/quantum/_capability/index.mq.md
---

Density operators — inspect a subsystem or a mixed-state matrix.

Build rho from an ideal circuit/state with from_state, or from a square
Hermitian 2^n matrix with from_matrix. Density operations and Schmidt support
at most six qubits. The matrix constructor checks shape and Hermiticity; it
does not certify positive semidefiniteness or trace one. For physical inputs,
check the trace and eig eigenvalues yourself before interpreting purity.

An independent mixed-state check

This one-qubit maximally mixed matrix has trace one, purity 0.5 and Z
expectation zero. The complete example needs no prior circuit or saved handle.
For entanglement, start from a Bell circuit, call from_state, then partial_trace
with keep=[0]; purity of that reduced density is also 0.5.

```sh
marqdo call ext/quantum/density/index.mq.md from_matrix 'matrix=[[0.5,0],[0,0.5]]' --effects plugin --session handbook --toget rho --json
marqdo call ext/quantum/density/index.mq.md purity 'density=$rho' --effects plugin --session handbook --json
marqdo call ext/quantum/density/index.mq.md expect 'density=$rho' 'obs="Z"' --effects plugin --session handbook --json
marqdo call ext/quantum/density/index.mq.md eig 'density=$rho' --effects plugin --session handbook --json
```

partial_trace keeps the listed zero-based wires and discards the others.
Pauli strings put the highest bit on the left; ZZ measures correlation on two
wires. matrix exports nested complex entries, and eig returns eigenvalues and
eigenvectors. Use [viz.draw_density](../viz/index.mq.md) for hinton, city,
density or paulivec plots. from_state ignores attached teaching noise, just
like the ideal simulator; it is not a noisy density evolution engine.

Development route

Add mixed-state mathematics to plugins/quantum/src/linalg.rs, export it through
src/lib.rs and update this file plus its Chinese mirror. Keep physical input
validation claims aligned with the constructor. Validate pure-state purity
one, Bell reduced purity one-half and observable expectations against
independent matrices in tests/ext/quantum-linalg-smoke.mq.md.

## from_state

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| state | any | required | pure state or circuit |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | density handle |

Use `state` as the inputs for this operation.

Form rho as the outer product of a pure `state` (or the ideal state of a
circuit). Use this when you trust the pure-state picture and want mixed-state
tools without writing a matrix by hand.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Form rho as the outer product of a pure state (or the ideal state of a circuit)](quantum_density_from_state(state=`state`))*.

## from_matrix

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| matrix | any | required | Hermitian matrix |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | density handle |

Use `matrix` as the inputs for this operation.

Build rho from an explicit Hermitian `matrix`. Pair with formula blocks in the
lecture so the matrix on the page is the matrix in the simulator. Size is
limited by the plugin.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Build rho from an explicit Hermitian matrix](quantum_density_from_matrix(matrix=`matrix`))*.

## purity

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| density | any | required | density operator |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | number | purity |

Use `density` as the inputs for this operation.

Compute the trace of rho squared for `density`. Values near one mean nearly
pure; lower values mean classical mixing or traced-out entanglement.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Compute the trace of rho squared for density. Values near one mean nearly pure; lower values mean classical mixing or traced-out entanglement.](quantum_density_purity(density=`density`))*.

## partial_trace

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| keep | any | required | qubit index list to keep |
| density | any | required | joint density |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | reduced density |

Use `keep`, `density` as the inputs for this operation.

Keep listed qubits in `keep` and trace out the rest of `density`. Indices use
LSB equals zero. The result is generally mixed even when the joint state was
pure.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Keep listed qubits in keep and trace out the rest of density](quantum_density_partial_trace(density=`density`, keep=`keep`))*.

## expect

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| obs | any | required | Pauli string or matrix |
| density | any | required | density operator |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | number | expectation value |

Use `obs`, `density` as the inputs for this operation.

Expectation of observable `obs` on `density`. Pauli strings use left equals
high bit. Pass a matrix when the observable is not a Pauli product.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Expectation of observable obs on density. Pauli strings use left equals high bit. Pass a matrix when the observable is not a Pauli product.](quantum_density_expect(density=`density`, obs=`obs`))*.

## eig

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| density | any | required | Hermitian density |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | eigenvalues and eigenvectors |

Use `density` as the inputs for this operation.

Hermitian spectrum of `density` as eigenvalues and eigenvectors. Useful when a
lecture needs the eigenbasis of rho.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Hermitian spectrum of density as eigenvalues and eigenvectors. Useful when a lecture needs the eigenbasis of rho.](quantum_density_eig(density=`density`))*.

## matrix

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| density | any | required | Density handle / 密度句柄 |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Nested complex entries with re and im |

Use `density` as the inputs for this operation.

Export the explicit nested complex matrix of `density`. Reading entries does not
change the density operator. Use the exported entries for an independent trace
check or further linear algebra; the caller remains responsible for physicality.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Export the explicit nested complex matrix of density](quantum_density_matrix(density=`density`))*.

# density

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| state | any | None | Default None |
| matrix | any | None | Default None |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Returned value |

Use `state`=None, `matrix`=None as the inputs for this operation.

Construct a density operator from a pure state or circuit (state=) or from an explicit Hermitian matrix=. Density operations support at most six qubits. Pure and mixed states share this object surface for purity, partial trace, and expectation.

We **[Load the native plugin required by this operation](cap.load())**.
1. `matrix`
  Return *[Construct a density operator from a pure state or circuit (state=) or from an explicit Hermitian matrix=](quantum_density_from_matrix(matrix=`matrix`))*.
2. *
  Return *[Construct a density operator from a pure state or circuit (state=) or from an explicit Hermitian matrix=](quantum_density_from_state(state=`state`))*.

## matrix

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Nested complex entries with re and im |

Return the explicit matrix of this density operator so authors can inspect entries or hand the array to further linear-algebra steps outside the quantum package.

Return *[Return the explicit matrix of this density operator so authors can inspect entries or hand the array to further linear-algebra steps outside the quantum package](quantum_density_matrix(density=`self`))*.

## purity

| 返回 | 类型 | 说明 |
|------|------|------|
| result | number | Scalar observation |

Compute Tr(ρ²). Values near one mean a pure state; mixed states are strictly less than one. Use this as a quick entanglement or decoherence check after a partial trace.

Return *[Compute Tr(ρ²)](quantum_density_purity(density=`self`))*.

## partial_trace

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| keep | any | required | Required |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Density operator handle |

Use `keep` as the inputs for this operation.

Trace out every qubit except those listed in keep (LSB index 0). Returns the reduced density on the kept subsystem for entanglement lectures and purity checks.

Return *[Trace out every qubit except those listed in keep (LSB index 0). Returns the reduced density on the kept subsystem for entanglement lectures and purity checks.](quantum_density_partial_trace(density=`self`, keep=`keep`))*.

## eig

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | eigenvalues and eigenvectors |

Compute the Hermitian spectrum of this density operator as eigenvalues and eigenvectors. Handy for lecture plots and for verifying that a constructed rho is physical.

Return *[Compute the Hermitian spectrum of this density operator as eigenvalues and eigenvectors](quantum_density_eig(density=`self`))*.

## expect

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| obs | any | required | Required |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | number | Scalar observation |

Use `obs` as the inputs for this operation.

Expectation value of observable obs on this density. obs may be a Pauli string with the left end as the high bit, or an explicit matrix matching the density dimension.

Return *[Expectation value of observable obs on this density](quantum_density_expect(density=`self`, obs=`obs`))*.

## draw

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | any | None | Default None |
| kind | any | "hinton" | Default hinton |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | quantum_svg with kind and svg |

Use `path`=None, `kind`="hinton" as the inputs for this operation.

Draw a Hinton diagram, city plot, density grid, or Pauli vector for this density. Optional path writes SVG; kind selects which visual metaphor the lecture uses.

Return *[Draw a Hinton diagram, city plot, density grid, or Pauli vector for this density](quantum_density_draw(density=`self`, path=`path`, kind=`kind`))*.
