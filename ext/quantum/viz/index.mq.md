---
markup: occlusion
title: ext/quantum/viz
description: >-
  Quantum visualization skill — circuit rails, probabilities, Bloch, density plots.
import:
  cap: ext/quantum/_capability/index.mq.md
---

Visualization

Drawings turn a circuit or density into SVG for view mode or a path on disk.
Pick a kind for the figure and a theme for colors. Themes dark (default), light,
and bw share geometry so lectures can switch style without rewriting the
circuit. Start with circuit and probs kinds when unsure.

## For AI agents

If you were given only this file, draw with marqdo call and --effects plugin.
You need a circuit or density handle from another module (toget).

Call example (plain text):
  marqdo call ext/quantum/viz/index.mq.md draw_probs circuit=$qc path="out.svg" --effects plugin --session s --json

## draw_circuit

Rail diagram of gates and barriers for `circuit`. Theme `theme`="dark" by
default. Optional `path` writes SVG for CLI or view embedding.

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | circuit to render |
| path | any | optional output path or None |
| theme | text | dark light or bw |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | drawing result |

> cap.load
*> quantum_draw_circuit circuit=`circuit` path=`path` kind="circuit" qubit=0 theme=`theme`*

## draw_probs

Probability bars for the ideal state of `circuit`. Same theme tokens as the
rail diagram. Uses `theme`="dark" when omitted. Optional `path` for file output.

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | circuit to render |
| path | any | optional output path or None |
| theme | text | dark light or bw |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | drawing result |

> cap.load
*> quantum_draw_circuit circuit=`circuit` path=`path` kind="probs" qubit=0 theme=`theme`*

## draw_bloch

Bloch sphere for one wire of `circuit`. Default `qubit`=0. Optional `path` and
`theme`="dark". Multi-qubit circuits still render one qubit reduced picture.

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | circuit source |
| path | any | optional output path or None |
| qubit | number | which wire |
| theme | text | dark light or bw |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | drawing result |

> cap.load
*> quantum_draw_circuit circuit=`circuit` path=`path` kind="bloch" qubit=`qubit` theme=`theme`*

## draw_density

Plot density operator `density`. Default `kind`="hinton". Optional `path`.
Choose city, density, or paulivec when the lecture uses those metaphors.

| 参数 | 类型 | 说明 |
|------|------|------|
| density | any | density operator |
| path | any | optional output path or None |
| kind | text | hinton city density or paulivec |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | drawing result |

> cap.load
*> quantum_density_draw density=`density` path=`path` kind=`kind`*

## draw_gate

Glyph or matrix heatmap for `gate`. Use `kind`="gate" by default, or matrix
when checking a custom unitary. Optional `path`.

| 参数 | 类型 | 说明 |
|------|------|------|
| gate | any | gate handle |
| path | any | optional output path or None |
| kind | text | gate or matrix |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | drawing result |

> cap.load
*> quantum_gate_draw gate=`gate` path=`path` kind=`kind`*
