# Document oracle observations and strict replay

`foreign.oracle_journal` returns a copy of this evaluation's observations as
ordinary `index/channel/value` records. Successful admitted `session_eval`,
`session_get`, and document `json.parse` operations append `eval`, `get`, and
`decode`, respectively. Set/open/close do not append. An observation is
validated recursively before append; handles and code objects cannot enter.
Each new HostContext starts with an empty observation list.

`foreign.replay_check expected=J observed=J2` is a pure comparison, with no
foreign execution. It validates contiguous zero-based indices, channel tags
and crossing-domain membership. Records compare structurally by field name,
independent of field order. The result contains `ok`, the least mismatching
`index`, and both records (None for a missing entry). Two empty journals
match. Failure to provide a valid journal is an error, not a match.

The interactive `DocumentSession::replay_check(snapshot)` is a different layer:
it executes a recorded trace from fresh state. Version-2 snapshots contain the
full chronological oracle trace, recorded stdin per step, its declared count,
the plan and source identity. Preflight validates every step before any foreign
execution. Every step is compared; none is skipped. Repeated cell executions
are distinct observations. Execution timing logs and cell display summaries
are not comparison inputs. Matching input values in a supplied snapshot is
consistency checking, not cryptographic authentication of the recording.

Snapshots carry the SHA-256 of the loaded document text. Import checks it
before restoring foreign bindings, and preflights all crossing payloads before
mutation. Reload retains prior oracle observations with their original source
hashes, so a later checker can reject an incompatible trace.

Lean: DocCalc's independent `runForeignLive` cursor simulates `runForeignAcc`
for arbitrary prefixes, observation accumulators and code sequences, including
oracle failures. `ForeignTrace` composes the registry's open/eval/close
judgments using independent normative and interactive execution control;
`trace_simulation`, `trace_correspondence`, and `trace_replay` quantify arbitrary
multisession action sequences, also when a run faults. These supplement the
existing theorems without removing them.

Tests: `tests/oracle_journal.rs` runs the public document API on both evaluators;
`host::oracle::tests` covers journal equality and length localization;
`host::foreign_doc::tests` covers the live trace preflight and actual replay;
`host::foreign_session::tests` includes generated two-session model checks and
Python/Node boundary and process-failure experiments. The research runner is
`MarqdoThesis/PLDI/experiments/reviewer-v5/run.py`.

## Stateful requests and semantic projection

`HostContext.foreign_protocol_steps` records internal request attempts and
replies, including set values, copied eval input and raw stdin. It is separate
from the admitted language journal: rejected inbound responses belong to the
protocol evidence, while only admitted values belong to the oracle journal.
An input refused before send produces no protocol request. A response refused
after execution does not undo kernel effects; the post-state is retained.

Language observations are recorded at the document API boundary and by live
cell evaluation. Low-level get/names used during snapshot export do not append
language observations. A successful eval projects to its result. A failed eval
projects to a record of stable status, output and diagnostics. Process timing,
commands and handles are not semantic values. The exact source path allocated
for one-shot execution is alpha-renamed only in diagnostic fields; raw stderr
and stdout remain available unchanged, and ordinary paths/error differences
are still compared. Live oracle records carry this same projected `value`.

Python/Node replies mark serialization-domain failure with `domain_error`.
Error text is not used to infer this flag. Python refuses non-string object
keys; Node sanitizes objects with a null prototype to preserve special keys.
The common strict JSON decoder checks duplicate keys before materializing a
map, rather than losing them through overwrite.

`ForeignState.lean` refines the earlier eval-only fragment with opaque kernel
state, complete request histories, set/get/eval input/decode, monotonic handle
allocation and fault outcomes carrying the admitted prefix. An adaptive
Program has continuations on actual observations; its correspondence theorem
covers data-dependent next actions, not just a fixed schedule. Mapping actual
document elaboration and host bindings to this program is a further proof
obligation recorded in the research status file.

## Complete history, snapshot version 4 (2026-11-02)

Strict interactive replay now records operations, not just successful cells.
Each public execution/mutation boundary records typed arguments, success or
failure, admitted observations, cell-oracle records, and stable resulting
state. Nested implementation calls are grouped under the user's operation.
The history includes set/get, run/step/run-all, restart, reload, plan edits,
clear, snapshot inspection and imported values. Failed attempts remain in the
history; rejected handles and secrets have only non-capability/redacted tags.
Clearing display outputs cannot erase the history's oracle projection.

The snapshot stores the materialized initial plan and historical reload plans.
Entry text is parsed at its original path, retaining relative imports without
rewriting the user's document. Initial bindings are reapplied when a kernel is
reopened; unchanged reload kernels retain state, whereas changed language or
command retires the incompatible process. Loaded source/metadata and cached
cell outputs are updated consistently.

Preflight checks the entire operation stream, all source revisions, plan edits,
binding/cursor/generation transitions, and oracle projections before starting
any replay kernel. Executing the trace compares each result, admitted value,
cell result and stable state. The first mismatch stops its dependent suffix;
`executed_actions` and `remaining_actions` make this explicit. `matched` counts
admitted observations and `matched_actions` counts operations. No missing
record is silently accepted or skipped.

The HTTP replay-check endpoint performs an isolated comparison and retains an
existing warm session. It validates malformed input before starting even a
seed kernel; duplicate JSON fields are refused before overwrite. Verification
can still run recorded foreign work and its external effects: isolation is of
the document session, not a process sandbox. `replay_snapshot` remains the
separate legacy cell re-run API and makes no strict-history comparison claim.
Older version-2/3 snapshots may be used by legacy restoration/re-run surfaces;
they cannot prove a complete action history and strict checking requires v4.

`ForeignHistory.lean` models explicit command boundaries. A fault terminates
its command, a later command receives the actual retained registry, and the
history retains all command faults and admitted observations. It proves
command-history correspondence, journal membership and no alias resurrection
through arbitrary recovery histories. This extends the adaptive Program
results; the general source-elaboration refinement is a separate outstanding
proof obligation.

Experiment source inputs are frozen before running, checked again afterward,
and archived with their hashes. The current passing bundle is identified by
`MarqdoThesis/PLDI/experiments/reviewer-v5/LATEST-PASSING.json`.

Each operation also retains its internal protocol attempts/replies, including
refused inspection results. Those records are compared even when an inspector
could not return a value. They are control evidence and do not admit rejected
payloads into the language journal. An explicit document session_names request
adds its admitted array on the decode channel; internal names/get probes do not.
Reload records either a historical source/plan input or its input-read error;
replay consumes that captured version input and does not rewrite past files.
