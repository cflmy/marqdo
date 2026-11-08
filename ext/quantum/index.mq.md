---
markup: occlusion
title: ext/quantum
description: Executable quantum handbook and experiment tools.
import:
  state: ext/quantum/state/index.mq.md
  circuit: ext/quantum/circuit/index.mq.md
  simulate: ext/quantum/simulate/index.mq.md
  density: ext/quantum/density/index.mq.md
---

Quantum handbook — read this index, execute an experiment, then open only the
module responsible for the next question. This is the package's actual program
and developer guide. The callable headings below are the tools; the surrounding
prose explains how and why to use them. No separate skill manifest is required.

Start with a verifiable result

These commands are independent and need no session. The Bell probabilities have
00 and 11 near 0.5. bell_analysis adds ZZ correlation near one, reduced purity
near one-half and Schmidt entropy near ln(2). Compare numbers within 1e-9,
rather than comparing a floating-point JSON string with literal 0.5.

```sh
marqdo call ext/quantum/index.mq.md bell_phi_plus --effects plugin --json
marqdo call ext/quantum/index.mq.md bell_analysis --effects plugin --json
```

Choose the next question

For a circuit to continue editing, call bell_circuit on this file and save its
return with --session and --toget. For a different initial program open
[circuit](circuit/index.mq.md): new, x/y/z/s/t/i, hadamard, rx/ry/rz, cnot/cz/swap,
apply, append, barrier and measure_mark cover construction and composition.

For a named or formula-defined operator open [gate](gate/index.mq.md). It
constructs gates and exposes matrix comparison. For exact amplitudes, states,
probabilities or noisy shots open [simulate](simulate/index.mq.md).

For tensor products, fidelity and Schmidt cuts open [state](state/index.mq.md).
For mixed matrices, partial trace, expectations and spectra open
[density](density/index.mq.md). For SVG rails, probabilities, Bloch, qsphere, multibloch, matrix and
density figures open [viz](viz/index.mq.md). Each module starts with a complete
experiment, input/output conventions and a concrete development route.

Use [the Chinese index](量子/索引.mq.md) for Chinese tool and parameter names.
The two trees share one native plugin and are tested against the same numerical
expectations. [_capability](_capability/index.mq.md) is the plugin diagnostic
boundary, not a prerequisite for every experiment.

Install and locate the source

These commands assume the package source tree is visible from the current
working directory. For an installed package replace the first FILE argument by
its actual installed path (by default ~/.marqdo/ext/quantum/index.mq.md).
Imports inside the package resolve through the extension search roots.

```text
marqdo ext add quantum
marqdo ext list
```

For development in this repository, build the local native plugin first. If
several versions are installed, set MARQDO_QUANTUM_PLUGIN to the absolute path
of target/debug/libquantum.so (or the platform's library filename). Never run
ext add with MARQDO_EXT pointing at the repository ext directory: installation
and source editing are different operations.

```text
source scripts/dev-env.sh
cargo build --locked -p marqdo_plugin_quantum
```

What a call means

The command shape is marqdo call FILE FUNCTION name=value --effects plugin
--json. FUNCTION is the exact callable heading; nested tools use parent.child.
A heading on an author object that reads self is an instance method and needs
an object receiver. Use this package's free tools for command-line work; import
the very same modules when writing an author program.

Arguments accept numbers, text, True/False/None, JSON lists/maps, or session
references. Protect text literals, JSON and every dollar reference with shell single quotes:
'name="H"' preserves the text quotes for the CLI value parser;
'circuit=$qc' sends a reference to Marqdo. Unquoted circuit=$qc would let the
shell expand an unrelated environment variable first. With --json, inspect both
the process exit status and ok before reading result. A failed call is not a
valid handle, even when stdout contains an error description.

A complete persistent experiment

All commands below use session handbook and can be executed in order. A call
stores only its return value under --toget; apply operations preserve the old
value, so save the new circuit after each step. Reading without --toget leaves
the binding unchanged. The final result must be near 0.5: a Bell pair is pure,
but either one-qubit subsystem is maximally mixed.

```sh
marqdo call ext/quantum/index.mq.md bell_circuit --effects plugin --session handbook --toget qc --json
marqdo call ext/quantum/simulate/index.mq.md probabilities 'circuit=$qc' --effects plugin --session handbook --json
marqdo call ext/quantum/density/index.mq.md from_state 'state=$qc' --effects plugin --session handbook --toget rho --json
marqdo call ext/quantum/density/index.mq.md partial_trace 'density=$rho' 'keep=[0]' --effects plugin --session handbook --toget reduced --json
marqdo call ext/quantum/density/index.mq.md purity 'density=$reduced' --effects plugin --session handbook --json
```

Session data lives under .marqdo/call-sessions unless MARQDO_CALL_SESSION_DIR
is set. Plugin calls are journaled and replayed across processes. Keep a session
short and use a new session id after changing source or plugin versions; replay
can repeat plugin effects such as plot output. Do not reconstruct a handle by
copying an implementation-specific JSON object into a new session.

Scientific conventions and limits

Qubit zero is the least significant, rightmost basis bit; Pauli strings put the
highest bit on the left. Angles are radians. State-vector width defaults to 12,
with an environment-adjustable hard cap of 16. Density and Schmidt stop at six
qubits. Schmidt A owns the low cut bits and entropy is in natural-log nats.

Exact simulation, probabilities, fidelity and density-from-circuit ignore
teaching noise. Only sampled run uses the attached bitflip, depolarizing or
amplitude_damping channel. Measurement marks choose sampled wires without
collapsing the ideal state. Mixed-matrix construction verifies Hermiticity and
shape, not positivity or trace one. These boundaries determine which scientific
claims an experiment can support; module intros give concrete counterchecks.

Author a program from the same knowledge

Save the following as bell.mq.md and run marqdo run bell.mq.md. The steps table
is the circuit specification; frontmatter imports the existing module. The two printed weights should be near 0.5. GFM
record tables begin with @; a comma-separated wire cell names control then target.
A formula can be bound before gate.custom or gate.gate in the same way.

```marqdo
---
markup: occlusion
import:
  circuit: ext/quantum/circuit/index.mq.md
---

# main

`steps` =

| @ | gate | qubits |
|---|------|--------|
| 1 | H | 0 |
| 2 | CX | "0,1" |

**qc = circuit.circuit qubits=2 steps=`steps`**
**p = > `qc`.probabilities**
**p00 = access(p, "00")**
**p11 = access(p, "11")**
> print text=`p00`
> print text=`p11`
```

Develop without creating another API

A new experiment normally belongs in this index as a small orchestration of
existing modules. State its physical question, inputs, expected values and
limitations beside the executable steps, as bell_analysis does below. A new
unitary normally belongs in gate.custom; it needs no new native export.

A genuinely new primitive belongs in its domain module. Read that module's
development route, add its free tool, preserve the existing author method, and
update the Chinese mirror. Native evolution is in plugins/quantum/src/sim.rs,
algebra in linalg.rs, SVG in draw.rs, and ABI registration in src/lib.rs there.
Keep loading confined to the capability boundary. If adding a new file, update
the package file list in src/ext_cli.rs so installation includes it.

Use prose declarations for parameter defaults, a parameter contract immediately
after the heading, and a return contract before any executable statement.
Current extract_function_contract reads both only before the first statement;
a return table placed after cap.load is documentation, not an enforced contract.
Use number/text/bool/any from the current type vocabulary; describe handle shape
and origin in prose instead of inventing a type checker. Interleave intent and
operations. Do not pad the narrative ratio with repeated generic sentences.

Ordinary section titles must stay prose: ## creates an actual callable symbol.
Only executable tools and object methods should appear in the tool tree. For
navigation use source links and the existing knowledge inspect/resolve commands;
stable symbols are compiler output, not a handwritten tool catalog.

```text
marqdo knowledge ext/quantum --json inspect bell_analysis
marqdo check --narrative-ratio ext/quantum/index.mq.md
cargo test --locked --test quantum_handbook
cargo test --locked --test call_cli quantum_
cargo test --locked --test gold ext_quantum -- --test-threads=1
```

The handbook test executes the sh blocks and author examples directly from both
language trees, compares independent scientific expectations, and checks both
Tree and Bytecode. An example that cannot be copied and run is a broken public
interface even if the narration ratio is high. Existing quantum golds preserve
noise, formula, SVG and author-object behavior. No network or LLM is needed.
This implements the small call-based core described in next/010–012; richer
self-modifying agents and persistent interactive branches remain future work,
not extra promises of this package.

Failure recovery

A missing plugin: inspect _capability, build/install quantum, then retry the
small Bell command. An unknown session binding: keep one session id and rerun
its producer before its consumer. Unexpected ideal results under noise: use run
for noisy samples. A width/shape error: use the limits and matrix conventions
above instead of silently reducing the scientific problem. Never treat an
error envelope as an empty successful result.

## bell_circuit

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Two-qubit circuit for simulation, density or drawing |

Prepare a Bell Phi-plus circuit with no arguments, starting from 00.
First create equal superposition on the low wire, then use that wire to
control the other. The resulting amplitudes correlate 00 with 11.
Return the circuit so a caller can apply another gate, attach noise, draw it
or independently test entanglement. Every step keeps the returned new value.

We retain **`qc` = [New](circuit.new(qubits=2, steps=None))**.

Make two equal paths on the low wire, then propagate the path label to the other wire.

We retain **`qc` = [Hadamard](circuit.hadamard(circuit=`qc`, qubit=0))**.
Return *[Cnot](circuit.cnot(circuit=`qc`, control=0, target=1))*.

## bell_phi_plus

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Basis probabilities; 00 and 11 each near 0.5 |

This no-argument demonstration reuses the preparation on this page and
returns only ideal probabilities. Expect equal weights on 00 and 11 and zero
elsewhere. Computational-basis correlation alone does not prove entanglement;
use bell_analysis for reduced purity, ZZ expectation and Schmidt evidence.

We retain **`qc` = [Bell circuit](bell_circuit())**.
Return *[Probabilities](simulate.probabilities(circuit=`qc`))*.

## bell_analysis

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | probabilities, purity, reduced_purity, zz, schmidt; no file output |

This no-argument experiment cross-checks the joint system, one subsystem
and their correlation from the same prepared circuit. Both language entries
return the same scientific field names. Callers need no native export names
or persistent session for the intermediate observations. Expect joint purity
one, reduced purity one-half, ZZ one and Schmidt entropy ln(2), within 1e-9.
All paths here describe ideal evolution rather than noisy trajectory averages.

We retain **`qc` = [Bell circuit](bell_circuit())**.
We retain **`probs` = [Probabilities](simulate.probabilities(circuit=`qc`))**.

Build the joint density and compute purity. A pure joint state need not
have pure subsystems. Keep only the low wire and test the mixedness that
appears when information about the other wire is discarded.

We retain **`rho` = [From state](density.from_state(state=`qc`))**.
We retain **`whole` = [Purity](density.purity(density=`rho`))**.

`kept` =

| qubit |
|-------|
| 0 |

We retain **`reduced` = [Partial trace](density.partial_trace(density=`rho`, keep=`kept`))**.
We retain **`local` = [Purity](density.purity(density=`reduced`))**.

ZZ measures correlation and is not an entanglement witness by itself.
The Schmidt decomposition supplies pure-state coefficients and entropy.
Together these observations distinguish a Bell pure state from a classical
mixture with identical basis probabilities. Keep the measured values in the
return table rather than hiding them behind a pass label with an implicit threshold.

We retain **`correlation` = [Expect](density.expect(density=`rho`, obs="ZZ"))**.
We retain **`split` = [Schmidt](state.schmidt(state=`qc`, cut=1))**.

`report` =

| probabilities | purity | reduced_purity | zz | schmidt |
|---------------|--------|----------------|----|---------|
| `probs` | `whole` | `local` | `correlation` | `split` |

Return *report*.
