---
markup: occlusion
title: ext/quantum/viz
description: >-
  Quantum visualization skill — circuit rails, probabilities, Bloch, density plots.
import:
  cap: ext/quantum/_capability/index.mq.md
---

Visualization — turn a scientific result into an inspectable SVG.

From circuit, use draw_circuit for gate rails.

From circuit, use draw_probs for ideal probability bars.

From circuit, use draw_bloch for one wire's reduced Bloch vector.

From density, use draw_density for hinton, city, density, paulivec.

From gate, use draw_gate for gate glyph or matrix heatmap.

From circuit, use draw_state for qsphere or multibloch.

Circuit figures accept dark, light and bw themes. Gate and density drawing
have no theme parameter on the current public API. Every tool returns a
quantum_svg map including svg and kind. Under call, use path=None to return
and record SVG; --effects plugin permits loading and drawing but the current
call host denies filesystem writes. To save a file, draw in an author program
executed with marqdo run.

A complete drawing experiment

Create the circuit first instead of assuming a handle already exists. The
probability result's svg field must contain an SVG document and kind must be
probs; the next results are qsphere and multibloch. Omitting path means None.
To save a file, append the following line to the root index's author example
before running it:

```text
**img = > `qc`.draw path="bell.svg"**
```

Relative paths resolve against the source document's directory. A call with a nonempty path fails under the current host policy.

```sh
marqdo call ext/quantum/circuit/index.mq.md new qubits=1 --effects plugin --session handbook --toget qc --json
marqdo call ext/quantum/circuit/index.mq.md hadamard 'circuit=$qc' --effects plugin --session handbook --toget qc --json
marqdo call ext/quantum/viz/index.mq.md draw_probs 'circuit=$qc' 'theme="light"' --effects plugin --session handbook --json
marqdo call ext/quantum/viz/index.mq.md draw_state 'circuit=$qc' path=None --effects plugin --session handbook --json
marqdo call ext/quantum/viz/index.mq.md draw_state 'circuit=$qc' 'kind="multibloch"' path=None --effects plugin --session handbook --json
```

Development route

Rendering belongs in plugins/quantum/src/draw.rs and the record_plot bridge
in src/lib.rs. Add a named choice to this page and its
Chinese mirror without mixing filesystem work into circuit algebra. Verify SVG
structure and numerical labels, not a screenshot alone. Preserve
 tests/ext/quantum-draw-smoke.mq.md and quantum-viz-advanced-smoke.mq.md.
For input preparation follow [circuit](../circuit/index.mq.md),
[gate](../gate/index.mq.md) or [density](../density/index.mq.md).

## draw_circuit

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| circuit | any | required | circuit to render |
| theme | text | "dark" | dark light or bw |
| path | any | None | optional output path or None |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | drawing result |

Use `circuit`, `theme`="dark", `path`=None as the inputs for this operation.

Render `circuit` as gate rails and barriers. Choose `theme`="dark" by default;
light and bw preserve the same geometry. Optional `path`=None returns SVG
without a file. A nonempty path is available to author programs under run.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Render circuit as gate rails and barriers](quantum_draw_circuit(circuit=`circuit`, path=`path`, kind="circuit", qubit=0, theme=`theme`))*.

## draw_probs

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| circuit | any | required | circuit to render |
| theme | text | "dark" | dark light or bw |
| path | any | None | optional output path or None |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | drawing result |

Use `circuit`, `theme`="dark", `path`=None as the inputs for this operation.

Plot ideal basis probabilities of `circuit`. Default `theme`="dark" supports
light and bw alternatives. Optional `path`=None returns SVG without a file.
The bars ignore teaching noise, just like simulate.probabilities.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Plot ideal basis probabilities of circuit](quantum_draw_circuit(circuit=`circuit`, path=`path`, kind="probs", qubit=0, theme=`theme`))*.

## draw_bloch

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| qubit | number | 0 | which wire |
| circuit | any | required | circuit source |
| theme | text | "dark" | dark light or bw |
| path | any | None | optional output path or None |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | drawing result |

Use `qubit`=0, `circuit`, `theme`="dark", `path`=None as the inputs for this operation.

Show the reduced Bloch sphere of wire `qubit`=0 from `circuit`. Default

`theme`="dark"; optional `path`=None returns SVG without a file. A multi-qubit
circuit still produces one reduced wire picture, not a joint-state sphere.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Show the reduced Bloch sphere of wire qubit=0 from circuit. Default circuit still produces one reduced wire picture, not a joint-state sphere.](quantum_draw_circuit(circuit=`circuit`, path=`path`, kind="bloch", qubit=`qubit`, theme=`theme`))*.

## draw_density

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| density | any | required | density operator |
| kind | text | "hinton" | hinton city density or paulivec |
| path | any | None | optional output path or None |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | drawing result |

Use `density`, `kind`="hinton", `path`=None as the inputs for this operation.

Plot `density` using `kind`="hinton" by default. Alternatives are city,
density and paulivec. Optional `path`=None returns SVG without a file.
Use the density module to construct the mixed-state input before drawing.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Plot density using kind="hinton" by default](quantum_density_draw(density=`density`, path=`path`, kind=`kind`))*.

## draw_gate

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| gate | any | required | gate handle |
| kind | text | "gate" | gate or matrix |
| path | any | None | optional output path or None |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | drawing result |

Use `gate`, `kind`="gate", `path`=None as the inputs for this operation.

Plot `gate` using `kind`="gate" by default. Choose matrix for a complex-entry
heatmap. Optional `path`=None returns SVG without a file. This observes the
operator rather than applying it or changing any circuit.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Plot gate using kind="gate" by default](quantum_gate_draw(gate=`gate`, path=`path`, kind=`kind`))*.

## draw_state

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| circuit | any | required | Circuit evolved ideally |
| kind | text | "qsphere" | qsphere or multibloch |
| path | any | None | SVG output path or None |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | quantum_svg with kind and svg |

Use `circuit`, `kind`="qsphere", `path`=None as the inputs for this operation.

Observe the ideal state of `circuit` as a basis-amplitude/phase sphere or one
reduced Bloch sphere per wire. Default `kind`="qsphere"; use multibloch for the
per-wire picture. Default `path`=None avoids explicit file output. A set of
local Bloch vectors alone cannot certify joint entanglement. Both advanced
figures currently use fixed styling; theme support remains Q8c work and is
not offered as a nonfunctional parameter here.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Observe the ideal state of circuit as a basis-amplitude/phase sphere or one reduced Bloch sphere per wire](quantum_draw_circuit(circuit=`circuit`, path=`path`, kind=`kind`, qubit=0, theme="dark"))*.
