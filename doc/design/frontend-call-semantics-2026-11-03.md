# Executable call and integer semantics

This describes the implementation contract exercised by
`src/frontend_elaboration_tests.rs`; it is not a claim of a verified parser.

All explicit actual arguments evaluate once in authored order, including extra
named arguments whose values the callee does not read. Binding then permutes
already computed values into parameter slots. Named arguments take precedence
over positional filling; duplicate names and excess positional arguments fail.
A duplicate-actual binding fault occurs only when its call is reached, after
all explicit actuals have evaluated; it cannot reject an unused function at
compile time or erase an earlier argument effect.
Local calls and imported calls use the same order. Builtins and host primitives
must not reorder expression evaluation when selecting their parameter values.

All supplied parameters are available before defaults. Missing defaults execute
in parameter declaration order, once per call, in the target definition's
module and lexical scope. A default can see supplied parameters, earlier
computed defaults, and an explicitly bound method receiver. It cannot capture
the caller's locals. A still omitted later parameter is unbound. Explicit None
is a supplied value and never triggers a default. Required-argument validation
precedes default execution. Default expressions participate in receiver
classification, dependency analysis and transitive effect inference.

A dynamic method's signature belongs to the resolved receiver and selected
method; a different object's same-named method cannot supply its parameter
list. Imported definitions retain their module scope. A compiled document's
entry is selected only from its own module; an imported `main` is not an entry.

The bytecode retains argument names and explicit presence bits. Callee
prologues evaluate defaults in their own frame. Local slots distinguish an
unbound slot from a slot holding None. Local, constant, call-layout and jump
indices use container-sized indices rather than unchecked 8/16-bit casts.
Scratch slots are reused across sequential calls while pending outer actuals
remain live. Scalar constant interning uses a hash table; floating constants
are keyed by their exact bits.

Host integers are signed 64-bit values. Add, subtract, multiply, divide and
negation report a language error on overflow in every build profile; division
by zero is a separate error. Division truncates toward zero. The negative
endpoint has a valid signed literal. Float-to-int conversion rejects nonfinite
or out-of-range inputs instead of silently saturating. This local integer
domain is distinct from the narrower continuous foreign-copy domain.

The Lean call-combinator proof preserves complete interaction programs for
arbitrary argument evaluators, including failure and reply-dependent requests.
The integer model specifies exact arithmetic followed by representability
checking. Native documents, independently derived Python expectations, a Lean
arithmetic consumer and isolated source mutations validate the implementation
links. These do not prove the entire Rust parser, binder, object representation
or adapter. The research coverage ledger must keep those obligations explicit.

## Module identity, entry inputs and source-bound invocation

Instances retain the existing map representation and safe-copy domain. The
reserved `_type_module` field is a list of import binding names, paired with
`_type`. Class selection uses that module instance within the active import
tree; inherited methods resolve their bases in the defining module. Equal
class names in different imports remain distinct. These fields are ordinary,
forgeable data, not capabilities or globally portable nominal identities.
Round trips through Python/Node preserve these descriptors in the same module
context. Opaque session handles remain outside the copy domain.

Entry metadata is passed to `Vm::run`, never compiled into `main`'s body.
Explicit named/recursive invocations of `main` therefore keep their own inputs.
Defaults run with the invocation's actual registry; a session-valued default
must not be evaluated in a separate interpreter and then injected into a VM.
Reads appearing only in explicit/prose defaults participate in input inference
and effect analysis. Default failures use the parameter declaration's span.
Source-bound APIs preserve the selected declaration's source identity and
span, even when two declarations have the same dotted path. Textual path calls
retain the existing first-declaration rule. Duplicate parameter names are
rejected because they cannot define distinct binding slots.

Both backends retain parameter, return and local field contracts. Bytecode
checks supplied/defaulted inputs before the body, explicit returns before
returning, table bindings before writing a local and direct footnote accesses
before/after lookup. Unknown fields report the contract error before a generic
missing-key error. Entry/raw invocations also check their completed inputs.
Contract diagnostics retain the declaration quote and the failure location.

## Refinement obligations and current evidence

| Link | Mechanized result or implementation evidence | Remaining obligation |
|---|---|---|
| Raw Markdown to executable AST | Native explicit/prose defaults, source snapshots, parameter spans, inference and duplicate-parameter diagnostics | Prove the actual Rust classifier/parser and extraction rules; the older abstract grammar is not that proof |
| Loaded module tree to flattened declarations | Independent inventories of source digests, module instances, lexical parents, class/method ownership and resolved bases | Connect the actual resolver to the Lean namespace lookup model, including failed/ambiguous resolution |
| Effectful actual evaluation to parameter values | `FrontendCalls` proves interaction-program equality for arbitrary expression evaluators; native authored-order/unused-actual/builtin/foreign probes | Prove Rust scratch allocation, every expression case and its stack invariant against that combinator |
| Default evaluation and invocation inputs | Native callee-scope, presence, receiver, entry/raw/metadata, inference, error-span and stateful-session cases | A complete default-binding semantics and universal compiler refinement, including errors and host context |
| Module-qualified object dispatch | `ModularObjects` proves namespace flattening lookup, resolved-parent dispatch and safe descriptor round trips | Connect every real object constructor, resolver and copied descriptor to the model; justify Rust module-root lifetimes |
| Machine integers | `MachineInts` exact signed-64 model, checked boundary implementation, independent Python and Lean consumers | Connect Rust checked arithmetic and every conversion to the mathematical evaluator; floats remain an extension |
| Progressive contracts | Four real source boundary failures preserve codes, anchors and failure spans; valid contract execution | Universal contract-preservation theorem and imported declaration/call-site attribution |
| Source-bound external calls | Two same-path declarations execute their separately selected source lines on both backends | Full external argument validation, effect-policy and environment-injection refinement |

The ledger deliberately does not turn a new model lemma, a finite passing
corpus, or independent reconstruction into a complete frontend theorem. Those
remaining proof links are required to discharge the full frontend objective.

## Default prologue proof and imported contract attribution (November 4)

`DefaultPrologue.default_prologue_refines` relates source default binding, which
rechecks the current environment for omission, to a guarded block machine with
an immutable initial presence mask. It proves whole interaction-program equality
for arbitrary effectful evaluators, assuming distinct parameter slots and an
initially faithful presence mask. Earlier default stores preserve that invariant
for later parameters. The duplicate-slot counterexample issues two requests in
the machine versus one in the source, demonstrating why slot uniqueness matters.
Explicit None skips and immediate failure stopping are separately checked.
This is a model of guard/evaluate/store blocks, not a proof of the Rust emitter,
flat instruction stepping, actual expression compilation or parser.

The generated implementation family crosses all 27 omitted/int/None states for
three parameters with all six declaration orders. An independent environment
model checks 162 documents and 324 executions, including 34 faults when a default
reads a later omitted parameter. Source text is reconstructed independently;
expected traces are not derived from either backend. This finite family does
not prove arbitrary defaults. Imported argument-contract diagnostics now retain
the caller's failure span and source revision while pointing the declaration
anchor and suggestion at the actual library. Supplied and defaulted invalid
arguments both refuse before entering the callee body.

## Concrete name and flat-code interfaces

The compiler now borrows each original declaration and owning module while
flattening. Bare-call selection uses the same source `lookup_local` operation
as Tree, then maps the selected declaration identity to its flat index. This
removes a second, subtly different lookup algorithm: repeated top-level names
previously selected the first declaration in Tree but failed compilation in VM.
It does not prove the source lookup operation itself.

`NameSlots` proves reads, writes and the complete named-argument binding fold
commute with an injective local-name vector. Unknown names and repeated raw
API names are covered; repeated raw names retain last-write semantics. A
presence mask computed from initialized slots is faithful by construction.
The Rust raw entry and argument binder now derive their mask from the actual
initial logical slots. Method receiver aliases are overlaid only for default
evaluation, and restored into the body frame after logical argument checks.
`ReceiverDefaults` supplies the additional environment reasoning for explicitly
declared self/自 parameters; the slot theorem alone does not cover that case.

Every compiled Program is structurally verified before return, and the VM
verifies even publicly constructed/mutated Programs before any instruction.
All current opcode variants are handled exhaustively. Checks include operand
bounds, parameter/local prefix and injectivity, declaration and inheritance
forests, call-layout and function references, spans, reachable stack underflow,
branch/loop join heights and absence of leaked stack values on return. Missing
variable values remain language faults; this checker does not prohibit them.

Rust exports the computed per-PC heights alongside the real emitted opcode
text. `CheckFlatCertificate.lean` independently decodes that text and applies
an executable checker whose soundness is proved in `FlatCertificate`. Accepted
certificates preserve heights along every finite path and guarantee a valid
fetch with sufficient operands on each reachable PC, including loop prefixes.
The independent reader rejects unmodeled opcodes. Rust debug serialization and
opcode-to-concrete-VM-step correspondence remain interfaces to justify; a
structural certificate does not establish source/bytecode functional equality.

Neither this ordinary bytecode verification technique nor reuse of a resolver
is an originality claim. Complete functional lowering, parser semantics and the
Rust implementation correspondence remain open proof obligations.

## Functional expression checks and receiver views (November 4)

The emitter now records source-directed expression rules before emission, with
child order and exact instruction intervals. The mandatory native verifier
checks literal identity (including floating-point bits), local names, unary and
binary operators, short-circuit targets, interpolation, collections, indexing
and occlusion. Calls delegate staging and selection checks to `bytecode/call.rs`;
resolution and dispatch semantics remain explicit boundaries.
All seven statement forms have source-directed templates; a separate forest
check covers the entire default prologue, statement body and implicit return.
These checks detect stack-preserving semantic corruption, including Add to Sub,
and refuse it before host effects. A forged source witness remains outside this
check's guarantee; the recorder and actual AST correspondence need justification.

`ExpressionLowering` proves postfix expression compilation preserves complete
interaction programs for arbitrary effectful leaves and operator interpretations,
including faults and an arbitrary caller stack and continuation. `ShortCircuit`
proves the exact absolute-PC AND/OR macros under arbitrary effectful child
evaluation. `CheckExpressionLowering.lean` independently reads actual emitted
operators, constants and local names, and applies the proved core checker.
It counts call, collection, index and interpolation boundaries explicitly.
Child inlining, Rust VM stepping, leaf interpretation and source-witness fidelity
remain obligations; none of these results is whole-frontend functional refinement.

`ReceiverDefaults` proves that the source's separately maintained default
environment equals the machine's receiver view after every successful prefix.
The mask describes logical argument slots, while omitted self/自 defaults may
replace the evaluation view. Supplied None remains present. The real receiver
is restored before entering the body; argument contracts check both the logical
arguments at the call site and the restored frame at the declaration, preserving
source identity and diagnostic frames. The theorem composes this view invariant
with `DefaultPrologue` for arbitrary effectful defaults and distinct slots.

Concrete repairs also share scalar/unary semantics across evaluators, stringify
every interpolation part, allocate fresh loop temporaries, restrict dynamic
calls to their defining module scope, and enforce the existing one-million
truthy-iteration limit in VM as well as Tree. Regression evidence records each
repair separately from proof claims. These compiler techniques are not new
originality claims.

## Actual call staging and bounded control-flow macros

Every emitted call now has a separate source obligation captured before actuals
are replaced with scratch reads. The checker covers authored actual order,
evaluate/store/pop triples, source-local and loop-slot separation, nested-call
scratch separation, argument reloads, layouts, builtin selection/defaults, host
primitive identity, plugin name/pair ordering, method name/receiver and resolved
user target/tag. Every call expression must have exactly one obligation. The
reserved source-local prefix is established before any synthetic slots are
allocated. Disjoint nested calls may reuse scratch slots after their lifetimes.

`CallStaging` proves the flat evaluate/save/pop program restores an arbitrary
caller stack and has the same effects and failures as authored-order collection.
It proves outside slots unchanged and unique staged slots read back the exact
actual vector. The dispatch continuation is arbitrary and effectful, so binding,
defaults and receiver effects can follow collection without commuting effects.
`CheckCallStaging.lean` decodes real stores and reloads, applies the proved stage
checker and independently checks the complete selection/dispatch template.
Source-expression callback correspondence, name resolution and the semantics of
each dispatch continuation remain proof interfaces. Host parameter metadata is
an exported registry interface, not an independently verified registry theorem.

`WhileLowering` proves the condition/JF/check/body/backedge macro preserves the
complete interaction program for a bounded loop, including a final false
condition, condition/body faults, the truthy-iteration limit and early returns.
It relates an increasing native counter to a decreasing model budget and proves
admitted increments below one million fit u32. Model machine-step fuel is
distinct from this language-visible iteration budget.
`CheckStatementLowering.lean` checks actual default prologues, all seven statement
templates, forests and implicit returns, including the exact while macro layout.
The while theorem uses condition/body callbacks; concrete child inlining and the
full Rust state machine are not discharged by checking those layouts. Branches
and foreach currently have independent template evidence, not new functional
refinement theorems. No ordinary compiler-checking technique is a novelty claim.

## Loaded executable AST and coherent witness corruption

`SourceFunction` now captures parameters, defaults, executable bodies and complete
contract data directly from the loaded declaration before emission. The native
source checker walks that snapshot separately from the rule recorder, checks
ordered expression/statement correspondence and complete witness ownership,
and accounts for the generated scratch reloads explicitly. It also compares
full contracts with structural Rust equality, including types, nullable fields,
descriptions, source lines and declaration quotes. Contract presence alone would
not detect a Number-to-Any weakening.

The evidence exports the executable AST separately from the original module and
from the compiled snapshot. `CheckSourceWitness.lean` matches declaration
identities and applies `SourceWitness`'s proved ordered-tree reconstruction
checker. Interpolation parts, index labels, code payloads and formula structure
are retained; all formula floats and matrix entries retain their exact bits.
The old Debug formula text was insufficient for NaN payload identity. Inert
occlusion prose and pre-inference parser metadata are not execution-tree nodes.

Ten native coordinated-corruption cases change code together with its witness,
including operators, constants, variables, assignments, callees, NaN payloads,
index keys, interpolation, contract metadata and an orphan expression record.
Eight independent source-witness negatives include changing the compiled snapshot
as well. A coordinated wrong operator is a positive acceptance control for all
four earlier independent consumers and a rejection control for the AST checker.
This distinguishes trusted source correspondence from agreement among emitted
metadata fields.

The theorem proves exact correspondence of the supplied executable trees, hence
preservation under any interpretation of those trees. The native and JSON AST
adapters, normalization helpers and snapshot capture remain trusted interfaces.
No theorem here verifies the actual Markdown classifier/parser/inference or
connects every concrete Rust operation to the semantic models. A caller who
changes both the reference AST and its program has supplied a different
translation-validation input; this is not source authentication.

## Runtime argument-preparation faults

The expanded AST audit exposed a concrete acceptance mismatch: Tree can execute
main while an unused function contains a conflicting bracket modifier, whereas
VM compilation previously rejected the entire module. Modifier preparation now
lowers its failure to `RejectArgumentExpansion` before actual evaluation. A
provided default parameter, including explicit None, skips that fault; reaching
it preserves prior output and stops before actual effects or a suffix. The
error's message, span, source revision and diagnostic frames match Tree.

`CallExpr::for_evaluation` normalizes bare aliases before modifier expansion,
matching Tree's existing order. The expression checker validates an effect-free
fault leaf and the independent decoder handles its actual terminal opcode.
The AST adapter explicitly represents that executable fault without children;
unexecuted actual expressions are not falsely counted as evaluated code.
Two independent fault-opcode/message negatives and a positive AST-identity-only
control keep representation checking distinct from instruction interpretation.

The independent Python oracle now compares JSON values with exact recursive
types and requires integer arithmetic inputs to be actual integers. Python's
True==1 and 1==1.0 equalities must not erase language-visible value distinctions.
Four type-corruption controls are added. These improvements do not discharge
other call-error timing, parser, resolver, dispatch or Rust-step proof obligations.

## Native binding and declaration graph validation

Builtins, host primitives and plugins retain authored labels in native bytecode.
The shared value-independent binding plan selects named parameters before
positionals, preserves omitted optional bindings and unknown named extras, and
reports duplicate, missing and surplus positional arguments after actual effects.
Plugin lookup precedes signature binding. Eighteen reached/unreached error cases
check explicit expected output, messages, spans, source revisions and frames.
A registered C ABI echo probe checks mixed and extra named arguments. Execution
uses the native VM and plugin bridge throughout.

The source and flat declaration views share ordered resolution. A loaded snapshot
captures ordinal declaration paths, levels, child/root counts, imports and ordered
use aliases before emission. Verification checks identity, parent/child/root order,
module scopes and object/method names, then derives call targets independently of
emitted target records. Two-component paths check runtime receiver presence before
static fallback; an unbound reserved slot differs from a supplied None. Imports
retain priority. Resolution failures are native terminal instructions after actuals.

`NameResolution` proves first-match, path and lexical commutation under an
ordered embedding. Its rank checker proves sufficient depth and connects a
well-founded total source policy to flat lookup. Rust uses the same parent-chain
policy; parent-view extraction remains a trusted adapter. The finite checkers consume actual AST/flat evidence in
`CheckNameResolution.lean`. Nine native corruptions and seven independent graph
negatives test this boundary. A coordinated wrong target remains a positive control
for the older stack, staging and AST-shape consumers. Ordinary compiler correctness
ingredients are not novelty claims. Snapshot/JSON adapters, primitive registries,
parent-view extraction and concrete Rust
instruction refinement remain explicit obligations; universal frontend verification
and overall paper readiness are not established by these results.

`ArgumentBinding` separately models per-parameter omission permission and proves
value substitution preserves selections and faults. A successful index plan has
a finite-index witness. `CheckArgumentBinding.lean` checks actual static call
plans and an independently enumerated mask/label corpus. User VM calls now use
the shared planner with the callee default mask, correcting their surplus-count
diagnostic and preserving missing-required-before-default priority. Default
evaluation, contracts, dynamic signature selection and concrete Rust/JSON steps
remain separate interfaces. The mathematical planner is not a proof of the
entire native binding implementation.
