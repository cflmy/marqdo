---
markup: occlusion
title: ext/quantum/simulate
description: >-
  Quantum simulation skill — amplitudes, probabilities, noisy shots.
import:
  cap: ext/quantum/_capability/index.mq.md
---

Simulation — choose the observable before choosing the execution path.

What are the amplitudes? Use simulate (noise: no); returns list of maps with re, im.

Need a state for tensor/fidelity work? Use state (noise: no); returns quantum_state with qubits, dim, amplitudes.

What are the exact basis weights? Use probabilities (noise: no); returns basis label → probability.

What would repeated readout produce? Use run (noise: yes); returns qubits, shots, seed, counts, measure; noise when set.

Attach one teaching noise channel Use noise (noise: later in run); returns new circuit.

Noise is bitflip, depolarizing or amplitude_damping, with probability in [0,1].
It acts after each unitary on that operation's participating wires. Attaching
noise replaces the previous channel; it does not append a gate or change the
ideal state. This is a teaching trajectory model, not a hardware backend.
Measurement marks select sampled wires without collapsing ideal amplitudes.

A complete noise experiment

First create a circuit through the [circuit module](../circuit/index.mq.md).
The ideal state after X is one. Full bit-flip noise after X makes every sampled
outcome zero; exact probabilities still describe one. Repeating run with the
same circuit, seed and shots is deterministic for this simulator version.

```sh
marqdo call ext/quantum/circuit/index.mq.md new qubits=1 --effects plugin --session handbook --toget qc --json
marqdo call ext/quantum/circuit/index.mq.md x 'circuit=$qc' --effects plugin --session handbook --toget qc --json
marqdo call ext/quantum/simulate/index.mq.md noise 'circuit=$qc' 'kind="bitflip"' p=1 --effects plugin --session handbook --toget noisy --json
marqdo call ext/quantum/simulate/index.mq.md probabilities 'circuit=$noisy' --effects plugin --session handbook --json
marqdo call ext/quantum/simulate/index.mq.md run 'circuit=$noisy' shots=32 seed=7 --effects plugin --session handbook --json
```

Development route

Put new readouts here rather than duplicating circuit construction. Numerical
evolution and seeded sampling live in plugins/quantum/src/sim.rs; exports live
in src/lib.rs beside it. Preserve ideal/noisy separation in both language trees
and validate tests/ext/quantum-noise-smoke.mq.md plus amplitude-damping golds.
For entanglement go to [state](../state/index.mq.md); for mixed states use
[density](../density/index.mq.md), which currently describes ideal evolution.

## simulate

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | circuit to evolve |

Return ideal state-vector amplitudes for `circuit` as re/im entries. Noise
channels attached to the circuit are ignored here.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | amplitude list |

> cap.load
*> quantum_simulate circuit=`circuit`*

## state

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | circuit to evolve |

Same ideal amplitudes as simulate for `circuit`, packaged as the plugin state
object. Prefer this when later fidelity or density tools expect a state handle.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | ideal state |

> cap.load
*> quantum_state circuit=`circuit`*

## probabilities

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | circuit to read |

Map basis label to probability for the ideal state of `circuit`. Labels use
qubit 0 as the rightmost bit. Noise is ignored.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | probability map |

> cap.load
*> quantum_probabilities circuit=`circuit`*

## run

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | circuit to sample |
| shots | number | sample count |
| seed | number | RNG seed |

Sample computational-basis outcomes from `circuit`. Defaults are `shots`=1024
and `seed`=1 so replays are deterministic. Teaching noise is applied when set.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | counts or shot result |

> cap.load
*> quantum_run circuit=`circuit` shots=`shots` seed=`seed`*

## noise

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | circuit to annotate |
| kind | text | noise family name |
| p | number | strength in the unit interval |

Attach teaching noise of kind `kind` with strength `p` on `circuit`. Ideal
simulate and probability tools ignore it. Keep p small unless demonstrating
decoherence. Supported kinds include bitflip, depolarizing, and amplitude_damping.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | circuit with noise metadata |

> cap.load
*> quantum_noise circuit=`circuit` kind=`kind` p=`p`*
