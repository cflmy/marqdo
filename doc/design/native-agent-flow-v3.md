# Marqdo native agent flow — implementation contract

Status: **incremental, draft**. The core orchestration now has a plugin-only
alternative to interpreter-stack suspension: a durable finite step state machine
with explicit safe commit boundaries. Do not claim arbitrary stack-level
continuation or exactly-once external writes based on a checkpoint file.

## Author surface (implemented in this branch)

| Module | Function | Contract |
|---|---|---|
| ext/ai/context.mq.md | scope | Select a bounded source excerpt using parsed Markdown headings; ambiguous names fail closed |
| ext/ai/context.mq.md | pack | Compact caller-provided file/class/function text via ContextPackV2 |
| ext/ai/flow.mq.md | ask/choose/parse_choice | One short decision, allowed-choice verification, at most one format repair |
| ext/ai/flow.mq.md | input/reply | Durable step-boundary user-input gate keyed by id + source revision and scope |
| ext/ai/flow.mq.md | start/wait/parallel | Existing Marqdo subtask scheduler, failure-preserving parallel result aggregation |
| ext/ai/flow.mq.md | enter | Resolve only author-supplied registered skill name, run via standard subtask path |
| ext/ai/recovery.mq.md | decide/save/load | Native error policy and CheckpointV2; no implicit unsafe re-execution |

The native agent plugin provides `agent_choice_parse`, `agent_context_scope`,
`agent_input_open`, `agent_input_resume` and `agent_parallel_report`.
Plugin errors must be visible, never converted into a fabricated successful answer.

## Invariants

- A model choice is untrusted data until verified against an explicit option list.
- An imported module is **not** automatically part of a model prompt.
- The requested heading scope, not the entire source file, is the prompt source.
- A pending user-input gate owns a persistent checkpoint, not a blocked interpreter thread.
- The same answer may be returned repeatedly; a competing answer or stale revision conflicts.
- A completed subtask wait is distinct from a successful child exit.
- Existing successful siblings remain visible when one subtask fails.
- Writes, executable effects and external writes remain under ToolSpec permissions,
  the EffectLedger and source-version checks.
- No component may silently broaden a tool's authorization on failure.

## Important limitations in this branch

The input/reply gate stores an explicit **step boundary**. It does not yet persist
an arbitrary bytecode interpreter frame. Applications must choose and re-enter
their next Marqdo function explicitly after receiving a verified input answer.
The current source selector relies on a Markdown parser plus Marqdo heading
conventions, not on the compiler's resolved module/class/function AST and symbols.
Host identity inputs are supplied by the caller; production applications must
attest them from their authenticated session, never use model-generated identity.
The parallel wrapper reuses `subtask.run_many` and does not add durable child
lease orchestration. Live provider tests and real 1B-model benchmarking remain open.

## Required VM/runtime completion: suspended execution

1. Introduce `WaitingInput` as a scheduler state, distinct from terminal failure.
   Save the logical run, program counter at a safe suspension boundary, bound
   inputs, variable snapshot, module and tool digests, and effect watermark.
2. Resume via a new attempt linked to the original logical run, but never
   automatically repeat committed effects. Preserve cancellation and budget.
3. The resume endpoint must be atomic (revision CAS), authenticate principal
   and workspace, verify source and tool schema revisions, and deduplicate
   repeated reply requests. Do not trust IDs inside model text.
4. Handle interrupted worker, partial snapshot, stale input, invalid choice and
   unknown external write as separate outcomes. Treat unknown writes as
   reconciliation-required; do not invent exactly-once guarantees.
5. Persist only serializable language values at declared safe points. If a
   foreign kernel, socket or native handle is live, explicitly refuse
   transparent frame recovery and preserve a safe step-level restart option.

## Required durable parallel execution

A parent dispatches child `.mq.md` functions using registered paths, grants
minimal capabilities, shares budget and rate limits, and persists a child task
manifest. Child completion records a status, value ref, output digest and ledger
watermark before the parent merges results. For same-resource writes, use
serialized revisions and explicit commit order. On parent cancellation, stop
new children and request child cancellation; do not assume remote writes rolled
back. On recovery, reconcile child attempts and reuse successful results.

## Release gates

| Test | Required observation |
|---|---|
| compiler/import check | All new `.mq.md` functions pass contract checks and execute via ABI |
| small model outputs | Exact, labeled, fenced and numeric options normalize; mixed options fail |
| input gate crash | After checkpoint, reply is committed once; alternate reply conflicts |
| source drift | Original gate cannot resume against changed source revision |
| context isolation | Class/function selectors reject ambiguity and ignore fenced headings |
| concurrent partial | Failed/uncertain child never discards successful siblings |
| unknown write | Neither retry nor recovered child duplicates an unconfirmed effect |
| long-run cancellation | Worker/child task termination preserves usable artifacts and receipts |
| compatibility | Existing `agent.step/plan/run` and workbooks remain callable |
| model evaluation | Tokens/decision, semantic validity, end-to-end completion, recovery overhead |

Do not merge the branch as a completed runtime refactor until the relevant
runtime gates, including crash injection and multi-worker concurrency, pass.


## Plugin-only workflow (November 18 revision)

The Agent ABI plugin implements `agent_workflow_open/next/commit/cancel`.
The small `ext/ai/workflow.mq.md` library wraps these primitives and provides
`tick` to execute exactly one safe decision or document task per call.
Source definitions have only five step kinds: `model`, `input`, `skill`,
`parallel` and `return`. Model text is verified against declared options.
`input` intentionally returns a waiting status; the UI supplies an answer
through `commit` on a later invocation. There is no interpreter thread to
hold and no need for a separate qdagent orchestration server.

The checkpoint records the source revision, caller-provided session and
principal scope, next step index and the completed outputs. The same step
cannot be silently committed twice. Mixed child outcomes preserve completed
siblings in the checkpoint and **block** further automatic execution until
the unknown/failed child operations are reconciled.

The state engine does not execute arbitrary unregistered tools directly:
`tick` delegates to existing Marqdo ABI-backed SKILL/subtask functions.
This is an implementation of **durable step-state recovery**, not a claim
of transparent recovery for a suspended foreign function or native call.
It provides the practical workflow needed for short qdagent SKILLs without
altering the Marqdo interpreter.

### Security/reliability boundary still required for production

- The host must attest source digest, principal, session, permissions and
  registry, rather than accepting values copied from an LLM response.
- Effectful child tools must use the existing EffectLedger and external
  receipt reconciliation; no automatic re-dispatch after an uncertain write.
- Cancelling a workflow checkpoint does not automatically cancel previously
  spawned external children; the supervisor must propagate cancellation.
- The current task runner is step-atomic, not a durable parallel child lease
  scheduler. A future plugin-only child manifest may use the host's existing
  `runtime` query API, but must not bypass process.exec or file-root grants.
- Cross-process locks need a lease/owner-aware stale-lock recovery path;
  a stale lock currently fails closed and requires operator review.
