---
markup: occlusion
title: ext/quantum
description: >-
  Quantum package root skill. Read this file first. Every tool runs with marqdo call.
import:
  state: ext/quantum/state/index.mq.md
  gate: ext/quantum/gate/index.mq.md
  circuit: ext/quantum/circuit/index.mq.md
  simulate: ext/quantum/simulate/index.mq.md
  density: ext/quantum/density/index.mq.md
  viz: ext/quantum/viz/index.mq.md
---

This file is the root skill for the Marqdo quantum package. If you are an AI
agent (or any program) with no other Marqdo context, follow Choose a path, then
How to execute any tool. Do not invent MCP tools, Python wrappers, or a second
API. The Markdown ## headings in these files are the tools.

## Choose a path

Pick one row. Do not mix English and Chinese tool names or file trees. (Do not
write Marqdo branch lines here; this table is narration only.)

| If you need | Do this |
|-------------|---------|
| Only a Bell result | Run `bell_phi_plus` on this file (command in the next section). Stop. |
| Circuit handles to continue (draw, noise, more gates) | Copy the stepwise circuit then simulate calls under circuit module below. Keep one `--session` and `--toget qc`. |
| Chinese tool names | Open `ext/quantum/量子/索引.mq.md` instead and stay in that tree. |

English paths stay under `ext/quantum/` (this tree). Agents should use `##`
tools via `marqdo call`. The `# circuit` / `# gate` objects on module files are
for author programs that import the module, not the default agent path.

*None*

## How to execute any tool

Marqdo runs one function from a Markdown file with this shell command shape.
Copy it literally. Replace FILE, FUNCTION, and arguments.

Call example (plain text):
  marqdo call FILE FUNCTION name=value name=value --effects plugin --json

FILE is always a path under ext/quantum/, usually ending in index.mq.md.
FUNCTION is the exact ## heading name in that file (for example new, hadamard,
probabilities). Nested ### headings use a dotted path like parent.child.
Always pass --effects plugin because the simulator is a native plugin; without
this flag marqdo call refuses the tool. Pass --json when you need the envelope
with ok and result fields. To keep a return value for the next command, add
--session some-id and --toget name (or put toget name at the end). Later
arguments may use circuit=$name. Plugin handles are rebuilt from the session
journal. Argument values may be numbers, quoted text, True, False, None, JSON
starting with [ or {, or $name session refs. Do not call ensure_plugin or any
host-star API; domain files load the plugin themselves.

Minimal first command to verify the package (no tool arguments):

Call example (plain text):
  marqdo call ext/quantum/index.mq.md bell_phi_plus --effects plugin --json

*None*

## Package layout

Open only the module you need. Each module file is also a complete skill and
repeats how to marqdo call its tools.

The path ext/quantum/index.mq.md is this root map and the Bell demo tool.
The path ext/quantum/circuit/index.mq.md builds wires, applies H and CNOT, and
defines the author object circuit.
The path ext/quantum/gate/index.mq.md holds gate matrices as formulas and named
or custom gates.
The path ext/quantum/simulate/index.mq.md returns amplitudes, probabilities, and
noisy shots.
The path ext/quantum/state/index.mq.md covers kron, fidelity, and Schmidt.
The path ext/quantum/density/index.mq.md covers density operators and partial
trace.
The path ext/quantum/viz/index.mq.md draws SVG figures.
The path ext/quantum/_capability/index.mq.md only loads the plugin for diagnostics.

Author programs import the same paths in frontmatter, then call module.member.
Example: import circuit from ext/quantum/circuit/index.mq.md, then use
circuit.circuit for the object constructor, or circuit.new for the call-style
tool that agents use with marqdo call.

*None*

## circuit module

File path: ext/quantum/circuit/index.mq.md

Use this when you need a quantum circuit handle. Typical agent sequence for a
two-qubit Bell preparation (copy each line as its own shell command):

Call example (plain text):
  marqdo call ext/quantum/circuit/index.mq.md new qubits=2 --effects plugin --session s1 --toget qc --json
  marqdo call ext/quantum/circuit/index.mq.md hadamard circuit=$qc qubit=0 --effects plugin --session s1 --toget qc --json
  marqdo call ext/quantum/circuit/index.mq.md cnot circuit=$qc control=0 target=1 --effects plugin --session s1 --toget qc --json

Main ## tools on that file: new, hadamard, cnot, barrier, measure_mark. The same
file also defines the author object circuit with methods such as h and cx.

*None*

## gate module

File path: ext/quantum/gate/index.mq.md

Use this when you need a gate handle or a formula matrix check.

Call example (plain text):
  marqdo call ext/quantum/gate/index.mq.md hadamard --effects plugin --json
  marqdo call ext/quantum/gate/index.mq.md cnot --effects plugin --json

Main ## tools: hadamard, pauli_x, pauli_z, cnot, custom, matches.

*None*

## simulate module

File path: ext/quantum/simulate/index.mq.md

Use this when you already have a circuit handle and want amplitudes,
probabilities, or sampled shots.

Call example (plain text):
  marqdo call ext/quantum/simulate/index.mq.md probabilities circuit=$qc --effects plugin --session s1 --json

Main ## tools: simulate, state, probabilities, run, noise. Ideal probabilities
ignore teaching noise; run applies noise when set.

*None*

## state module

File path: ext/quantum/state/index.mq.md

Use this for Kronecker product, fidelity, and Schmidt decomposition.

Call example (plain text):
  marqdo call ext/quantum/state/index.mq.md fidelity a=$psi b=$phi --effects plugin --json

Main ## tools: kron, fidelity, schmidt.

*None*

## density module

File path: ext/quantum/density/index.mq.md

Use this for mixed states.

Call example (plain text):
  marqdo call ext/quantum/density/index.mq.md from_state state=$qc --effects plugin --toget rho --json
  marqdo call ext/quantum/density/index.mq.md purity density=$rho --effects plugin --json

*None*

## viz module

File path: ext/quantum/viz/index.mq.md

Use this to draw SVG. Pass a circuit or density handle and optional path.

Call example (plain text):
  marqdo call ext/quantum/viz/index.mq.md draw_circuit circuit=$qc path="bell.svg" --effects plugin --session s1 --json

Main ## tools: draw_circuit, draw_probs, draw_bloch, draw_density, draw_gate.

*None*

## bell_phi_plus

One-shot orchestration tool on this index file. It takes no parameters. It builds
the Bell Phi-plus state with Hadamard on qubit 0 and CNOT from 0 to 1, then
returns ideal basis probabilities. Use this when you only need the demo result.
Use the stepwise circuit and simulate calls above when you need intermediate
handles.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | map of basis label to probability |

Exact command to run this tool:

Call example (plain text):
  marqdo call ext/quantum/index.mq.md bell_phi_plus --effects plugin --json

**qc = > circuit.new qubits=2 steps=None**
**qc = > circuit.hadamard circuit=`qc` qubit=0**
**qc = > circuit.cnot circuit=`qc` control=0 target=1**
*> simulate.probabilities circuit=`qc`*
