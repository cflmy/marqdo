---
markup: occlusion
title: Entanglement lab
description: Bell state — density, partial trace, Schmidt, themed SVG (Q7/Q8).
import:
  quantum: ext/quantum/circuit/index.mq.md
  math: lib/math.mq.md
---

# Entanglement from a runnable document


Build |Φ⁺⟩ with H then CX. Full-state purity stays 1; tracing out one qubit yields a mixed state (purity 1/2). Schmidt coefficients and entropy quantify the bipartition. SVG plots (hinton / qsphere / multibloch) make the same facts visible.

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

Prepare a two-qubit Bell state by applying H and then controlled-X. The full density matrix is pure, while tracing out one qubit leaves a mixed reduced state; these are different questions about the same state. The ZZ expectation and Schmidt entropy give complementary checks, not additional measurements of an evolving physical device.

The five SVG outputs are real artifacts written through the quantum drawing API, so run in a disposable filesystem root with its required plugin and write effects. Their different views share the calculated state. Keep both numeric observations and generated paths when verifying the example; a successful file write alone does not establish the entanglement values.

`steps` =

| step | gate | qubits |
|------|------|--------|
| 1 | H | 0 |
| 2 | CX | 0,1 |

We first **qc = [Compute qc](quantum.circuit(qubits=2, steps=`steps`))** for the next step.
We first **p = [Compute p](> `qc`.probabilities)** for the next step.

We first **rho = [Compute rho](> `qc`.density)** for the next step.
We first **pur = [Compute pur](> `rho`.purity)** for the next step.
We first **red = [Compute red](> `rho`.partial_trace keep=0)** for the next step.
We first **rpur = [Compute rpur](> `red`.purity)** for the next step.

We first **zz = "ZZ"** for the next step.
We first **ezz = [Compute ezz](> `qc`.expect obs=`zz`)** for the next step.

We first **sch = [Compute sch](> `qc`.schmidt cut=1)** for the next step.
We first **ent = [entropy](access(`sch`, "entropy"))** for the next step.

We first **_ = [Compute _](> `qc`.draw path="entangle-circuit.svg" theme="dark")** for the next step.
We first **_ = [Compute _](> `qc`.draw kind="hinton" path="entangle-hinton.svg")** for the next step.
We first **_ = [Compute _](> `qc`.draw kind="qsphere" path="entangle-qsphere.svg")** for the next step.
We first **_ = [Compute _](> `qc`.draw kind="multibloch" path="entangle-multibloch.svg")** for the next step.
We first **_ = [Compute _](> `rho`.draw kind="city" path="entangle-city.svg")** for the next step.

We now **[Display this step's result](> print text=`p`)**.
We now **[Display this step's result](> print text=`pur`)**.
We now **[Display this step's result](> print text=`rpur`)**.
We now **[Display this step's result](> print text=`ezz`)**.
We now **[Display this step's result](> print text=`ent`)**.
