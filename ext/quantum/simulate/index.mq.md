---
markup: occlusion
title: ext/quantum/simulate
description: >-
  Quantum simulation skill — amplitudes, probabilities, noisy shots.
import:
  cap: ext/quantum/_capability/index.mq.md
---

Simulate

Simulation answers what a circuit does. Ideal paths (simulate, state,
probabilities) ignore teaching noise. The run tool draws computational-basis
shots and applies circuit noise when noise was attached. The simulator refuses
circuits wider than the plugin limit instead of silently truncating.

## For AI agents

If you were given only this file, you need a circuit handle from
ext/quantum/circuit/index.mq.md (usually via --toget). Then:

Call example (plain text):
  marqdo call ext/quantum/simulate/index.mq.md probabilities circuit=$qc --effects plugin --session s --json

Other tools: simulate, state, run, noise. Always --effects plugin.

## simulate

Return ideal state-vector amplitudes for `circuit` as re/im entries. Noise
channels attached to the circuit are ignored here.

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | circuit to evolve |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | amplitude list |

> cap.load
*> quantum_simulate circuit=`circuit`*

## state

Same ideal amplitudes as simulate for `circuit`, packaged as the plugin state
object. Prefer this when later fidelity or density tools expect a state handle.

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | circuit to evolve |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | ideal state |

> cap.load
*> quantum_state circuit=`circuit`*

## probabilities

Map basis label to probability for the ideal state of `circuit`. Labels use
qubit 0 as the rightmost bit. Noise is ignored.

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | circuit to read |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | probability map |

> cap.load
*> quantum_probabilities circuit=`circuit`*

## run

Sample computational-basis outcomes from `circuit`. Defaults are `shots`=1024
and `seed`=1 so replays are deterministic. Teaching noise is applied when set.

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | circuit to sample |
| shots | number | sample count |
| seed | number | RNG seed |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | counts or shot result |

> cap.load
*> quantum_run circuit=`circuit` shots=`shots` seed=`seed`*

## noise

Attach teaching noise of kind `kind` with strength `p` on `circuit`. Ideal
simulate and probability tools ignore it. Keep p small unless demonstrating
decoherence. Supported kinds include bitflip, depolarizing, and amplitude_damping.

| 参数 | 类型 | 说明 |
|------|------|------|
| circuit | any | circuit to annotate |
| kind | text | noise family name |
| p | number | strength in the unit interval |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | circuit with noise metadata |

> cap.load
*> quantum_noise circuit=`circuit` kind=`kind` p=`p`*
