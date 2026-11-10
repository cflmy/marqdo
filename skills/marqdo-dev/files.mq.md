---
markup: occlusion
title: Native repository file operations
import:
  fs: lib/fs.mq.md
  re: lib/re.mq.md
  hash: lib/hash.mq.md
  table: lib/table.mq.md
  result: lib/result.mq.md
  engineering: ext/engineering/index.mq.md
---

Repository development reads and writes through Marqdo's own filesystem tools.
These functions are the same operations used by the handbooks, not shell text
that another agent must regenerate. Paths stay under the granted filesystem
root; pass --fs-root . at the checkout root. read/search need fs.read, while
write/patch need fs.write. Writes imply reads but grant no process or network IO.

A write is an explicit edit requested by the caller. Read the current file and
review the intended change before writing; these functions do not decide which
files belong to a task. Patch is useful for a unique bounded change; write is
useful for an explicitly reviewed complete artifact. Neither creates parent
directories implicitly, and neither promises a transaction across multiple files.

## read

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | UTF-8 source path under --fs-root |

| 返回 | 类型 | 说明 |
|---|---|---|
| source | text | Exact UTF-8 file contents without interpretation |

Use `path` to read a source document before editing it. The returned text is
data: embedded bold, returns, imports and shell-looking content are not run.
A missing path, invalid UTF-8 or root escape produces the native filesystem
diagnostic. For binary data use lib/fs.read_bytes or read_chunk instead.

After selecting the source we return *[Read the actual UTF-8 artifact](fs.read_text(path=`path`))*.

## write

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Destination under --fs-root; parent must exist |
| source | text | required | Reviewed complete UTF-8 content, including intentional newlines |

| 返回 | 类型 | 说明 |
|---|---|---|
| sha256 | text | SHA256 of the bytes actually written |

Use `path` and `source` for an explicit authored artifact. JSON argument input
carries quotes, Unicode, literal dollar signs and newlines without generating
a program in another language. Existing files are replaced only because the
caller selected write. A single-file atomic replacement does not turn a batch
of writes into a transaction or erase unrelated work on failure.

We **written = [Atomically replace this selected UTF-8 artifact](fs.atomic_write(path=`path`, text=`source`))**.
After writing, return *[Hash the actual resulting file](fs.file_hash(path=`path`))*.

## patch

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Existing UTF-8 file under --fs-root |
| find | text | required | Exact nonempty fragment that occurs once |
| replace | text | required | Reviewed replacement fragment |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Native patch result; missing/ambiguous matches are diagnostics |

Use `path`, `find` and `replace` when changing one known part of a file.
The exact-match requirement catches drift between reading and editing, though
it is not a concurrent kernel compare-and-swap. A whole-file replacement belongs
to write. This helper never falls back to a model-generated search-and-replace
program when the old fragment no longer matches.

Once the bounded change is reviewed, return *[Apply one exact native text patch](fs.text_patch(path=`path`, find=`find`, replace=`replace`))*.

## search

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | "." | Selected source directory under --fs-root |
| pattern | text | required | Unicode regex applied to each source line |
| glob | text | "**/*.mq.md" | File pattern; choose **/*.rs for runtime code |

| 返回 | 类型 | 说明 |
|---|---|---|
| matches | list | Records with path, one-based line and original text |

Use `path`=".", `pattern` and `glob`="**/*.mq.md" to search actual sources.
The native walker does not traverse symlinks and prunes generated/cache trees.
Reading remains UTF-8 and bounded by the filesystem implementation. Each result
retains its source path and line so an agent can read that artifact rather than
guessing from a detached snippet. This is a source query, not a code execution.

`excluded` =

| pattern |
|---|
| .git/** |
| **/target/** |
| **/node_modules/** |
| **/.marqdo/** |
| **/vendor/** |
| **/__pycache__/** |

We **paths = [Find source files through the native filesystem](fs.find(path=`path`, pattern=`glob`, exclude=`excluded`))** and
**matches = [Start the source-location result list](table.empty_list())**.
- [Read each selected text source](for paths as file)
  We **actual_path = path + "/" + file** and
  **lines = [Read its UTF-8 lines](fs.read_lines(path=`actual_path`))** and **number = 0**.
  - [Test the requested expression against each line](for lines as source)
    We **number = number + 1** to retain one-based source coordinates.
    1. [This line matches the requested source query](re.is_match(text=`source`, pattern=`pattern`))

      `hit` =

      | path | line | text |
      |---|---|---|
      | `actual_path` | `number` | `source` |

      We **matches = [Retain this source location](table.append(list=`matches`, item=`hit`))**.

When the selected sources are exhausted we return *matches*.
## read_many

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| paths | list | required | Explicit UTF-8 source paths selected for one review batch |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Sources and hashes for the exact requested files |

Use `paths` when reviewing a small related batch. Native filesystem reads
preserve the full UTF-8 text; hashes identify the exact returned source used to prepare a patch. The digest is
computed from that text, so a later filesystem change cannot pair new bytes with
an older source in the same record.
Reading does not run imports, main or embedded code. Choose a bounded batch so
the returned source remains useful to the agent rather than dumping a whole
repository into one response. Any missing or escaping path is a native error.
The same path/hash pair can be passed to apply for a preflight drift check.

We **records = [Start the source review batch](table.empty_list())**.
- [Read only the selected artifacts](for paths as path)
  We **source = [Read the exact UTF-8 source](fs.read_text(path=`path`))** and
  **sha256 = [Identify the bytes used for this review](hash.sha256(text=`source`))**.


  `record` =

  | path | source | sha256 |
  |---|---|---|
  | `path` | `source` | `sha256` |

  We **records = [Retain the source and its observed identity](table.append(list=`records`, item=`record`))**.
Return *records* after reading the selected batch.

## inventory

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| paths | list | required | Explicit source paths to inspect without executing |
| include_body | bool | False | Include compiler AST debug evidence for a migration |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Source, hash and actual compiler analysis for each selected artifact |

Use `paths` and `include_body`=False for an authored source review.
Marqdo reads each file through lib/fs, then sends its text to the compiler-backed
extension analyzer. The analyzer parses the same language, supplies real
parameters/defaults/contracts and measures each function's narrative ratio.
Imports are not executed or followed by this text inspection; full program checks
still use marqdo check. It needs fs.read and the engineering plugin, and makes no
network request or edit. A parse failure stays attached to that file as a Result
instead of making the rest of the inventory disappear.

We **records = [Start the executable source inventory](table.empty_list())**.
- [Inspect the selected source with its actual compiler](for paths as path)
  We **source = [Read the original source through the native filesystem](fs.read_text(path=`path`))** and
  **sha256 = [Retain its observed byte identity](hash.sha256(text=`source`))** and
  **analysis = [Parse contracts and statements without running them](engineering.analyze(source=`source`, include_body=`include_body`))**.

  `record` =

  | path | source | sha256 | analysis |
  |---|---|---|---|
  | `path` | `source` | `sha256` | `analysis` |

  We **records = [Keep the actual per-file evidence](table.append(list=`records`, item=`record`))**.
Return *records* after the requested source inventory.

## apply

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| updates | list | required | Reviewed records containing path/source and optional expected_sha256 |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result containing paths and SHA256 values actually written |

Use `updates` to apply a reviewed batch through native file IO. Each record
selects its path and complete source; expected_sha256 optionally ties that edit
to the previously read version. All supplied digests and duplicate destinations
are checked before the first write, preventing an ordinary stale review from
partially changing the batch. Path access checks also happen before writing.

This is still not a multi-file transaction or a kernel CAS. Each file replacement
is atomic on its own; a later disk failure can leave earlier reviewed files
written. Parent directories must already exist. A deliberate new file can omit
expected_sha256, while updating an observed file should retain its hash.
The result lists actual file hashes rather than claiming compliance with a style
rule. Call the static and behavioral checks after applying the batch.

We **seen = [Start the destination preflight list](table.empty_list())**.
- [Preflight every reviewed destination before writing](for updates as update)
  We **path = [Read the selected destination](access(update, "path"))** and
  **exists = [Apply the native root policy before any write](fs.exists(path=`path`))** and
  **expected = [Read its optional reviewed source identity](table.get(map=`update`, key="expected_sha256"))**.
  1. [This destination occurs twice](table.contains(list=`seen`, item=`path`))
    Return *[Reject duplicate writes in one batch](result.error(code="authoring.duplicate", message=`path`, operation="files.apply"))*.
  1. `expected` != None
    1. not `exists`
      Return *[Keep a missing reviewed artifact visible](result.error(code="authoring.missing", message=`path`, operation="files.apply"))*.
    1. fs.file_hash(path=`path`) != `expected`
      Return *[Refuse a stale reviewed update](result.error(code="authoring.changed", message=`path`, operation="files.apply"))*.
  We **seen = [Retain this checked destination](table.append(list=`seen`, item=`path`))**.
We **written = [Start the actual write evidence list](table.empty_list())**.
- [Write each explicit reviewed artifact through Marqdo](for updates as update)
  We **path = [Read its checked destination](access(update, "path"))** and
  **saved = [Atomically replace the selected UTF-8 content](fs.atomic_write(path=`path`, text=access(update, "source")))** and
  **sha256 = [Identify the bytes that were actually written](fs.file_hash(path=`path`))**.

  `evidence` =

  | path | sha256 |
  |---|---|
  | `path` | `sha256` |

  We **written = [Retain this actual file result](table.append(list=`written`, item=`evidence`))**.
Return *[The completed native edit batch](result.ok(value=`written`))*.

## analyze_updates

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| updates | list | required | Proposed path/source records; no filesystem mutation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Compiler metadata for each proposed artifact |

Use `updates` before applying an authored batch. Each proposal is analyzed
as text by the real compiler without replacing the current file or executing
the proposal. The path is an identity label for the report; no untrusted path
is opened here. Parameter order/defaults and statement projections allow a
reviewer to distinguish documentation changes from behavioral changes.

The projection deliberately omits source locations and occlusion labels from
executable statements; it is review evidence, not a proof of all runtime behavior.
Keep real golden calls and backend tests for observable behavior. Parse failures
remain per-file Results so another proposal's valid syntax cannot mask them.

We **reports = [Start the proposed-source review](table.empty_list())**.
- [Analyze one proposed artifact without writing it](for updates as update)
  We **path = [Retain its intended destination label](access(update, "path"))** and
  **source = [Read the proposed UTF-8 value](access(update, "source"))** and
  **analysis = [Use the actual parser and contract checker](engineering.analyze(source=`source`))**.

  `report` =

  | path | analysis |
  |---|---|
  | `path` | `analysis` |

  We **reports = [Retain the actual compile evidence](table.append(list=`reports`, item=`report`))**.
Return *reports* after analyzing all proposals.


## _missing_contracts

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| functions | list | required | Compiler-projected function tree from engineering.analyze |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Executable functions whose return contract is missing |

Use `functions` from the compiler projection; never infer functions by scanning
hash-prefixed source lines. Content-only headings have no executable statements
and are omitted, so a prose section does not acquire a fictional return table.
Nested functions are inspected recursively and retain compiler source lines.

This is a pure structural check. The compiler separately checks declared
parameter names and types. A return table's presence is not proof that prose is
accurate or that an effectful function behaves correctly. Keep runtime tests for
those properties. The source tree is ordinary data and no imports are executed.

We **missing = [Start the actual missing-contract findings](table.empty_list())**.
- [Inspect real callable units and their nested definitions](for functions as function)
  We **contract = [Read its parsed public contract](access(function, "contract"))** and
  **active = [Distinguish executable units from content-only headings](len(access(function, "semantic")) > 0)**.
  We **missing_return = True** until an actual return contract is observed.
  1. `contract` != None
    We **missing_return = [Inspect the existing contract without indexing None](access(contract, "returns") == None)**.
  1. `active` and `missing_return`
    We **name = [Keep the exact callable name](access(function, "name"))** and **line = [Keep its source location](access(function, "line"))**.

    `finding` =

    | name | line | issue |
    |---|---|---|
    | `name` | `line` | "missing return contract" |

    We **missing = [Retain the source-based finding](table.append(list=`missing`, item=`finding`))**.
  We **nested = [Inspect child definitions by the same rule](_missing_contracts(functions=access(function, "children")))** and
  **missing = [Combine findings without executing a function](table.concat(a=`missing`, b=`nested`))**.
Return *missing* for the caller's report.

## _active_lines

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| functions | list | required | Compiler-projected function tree |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Source heading lines belonging to executable units |

Use `functions` from the compiler projection. An executable statement or a
declared parameter makes a unit part of the callable audit; a plain content
heading is not a zero-length program. This distinction also covers field-only
object schemas, which use field contracts rather than fictional return tables.

The walk includes nested definitions and retains their actual heading lines.
It is a pure projection, not a source-text pattern or an import traversal.
An explicit return of None and an empty-payload assignment are still statements,
so intentionally unfinished callable code stays visible to the authoring checks.
The compiler's narrative measurements are used unchanged for every selected unit.

We **lines = [Start the actual executable heading locations](table.empty_list())**.
- [Inspect real declarations and their nested definitions](for functions as function)
  1. len(access(`function`, "semantic")) > 0 or len(access(`function`, "params")) > 0
    We **lines = [Retain this executable source unit](table.append(list=`lines`, item=access(function, "line")))**.
  We **nested = [Inspect nested source units by the same rule](_active_lines(functions=access(function, "children")))** and
  **lines = [Keep the combined executable locations](table.concat(a=`lines`, b=`nested`))**.
Return *lines* for matching the unchanged narrative measurements.

## audit

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| paths | list | required | Explicit maintained source paths; exclude intentional invalid fixtures |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result with compact per-file parser, narrative and contract findings |

Use `paths` for a selected maintained source batch. The same compiler that
runs Marqdo parses each source, checks declared contracts and measures narrative
per function. This task reads through the native filesystem and gives the plugin
only source text. It neither executes imports nor writes files. Requires
fs.read and plugin; it performs no network request.

A ratio at or below one half is a finding, including a data-heavy callable.
Do not solve findings by adding repeated filler or inventing return values for
pure content. Reorganize large code/data into useful modules and explain actual
inputs, decisions and limits. This structural audit is an editing aid; run check
for dependency validation and real cases for behavioral and visual results.

Pure content headings are excluded from callable ratios. Compiler warnings stay
in diagnostics for review; errors, parser failures and authoring gaps make the
batch incomplete. This allows a documented empty-payload demonstration to retain
its warning without replacing the example with fake business logic.

The result retains all selected file findings even if one fails parsing. A
complete batch returns ok=true. An incomplete batch returns ok=false with the
same evidence in value, so --require-ok can stop automation without discarding
the review report. Missing files or root violations remain native diagnostics.
Newline-bearing paths have no special meaning here; they are ordinary input data.

We **reports = [Start the compact source review](table.empty_list())** and **complete = True**.
- [Audit every explicitly selected source without running it](for paths as path)
  We **source = [Read the source using native Marqdo IO](fs.read_text(path=`path`))** and
  **analysis = [Use the actual compiler projection](engineering.analyze(source=`source`))** and
  **parse_ok = [Retain whether parsing succeeded](access(analysis, "ok"))** and
  **diagnostics = [Start this file diagnostic list](table.empty_list())** and
  **ratio_gaps = [Start this file narrative findings](table.empty_list())** and
  **missing_contracts = [Start this file contract findings](table.empty_list())**.
  1. `parse_ok`
    We **data = [Read the successful source projection](access(analysis, "value"))** and
    **diagnostics = [Retain genuine contract diagnostics](access(data, "diagnostics"))** and
    **missing_contracts = [Inspect executable units recursively](_missing_contracts(functions=access(data, "functions")))**.
    We **ratios = [Read the measured function units](access(data, "ratios"))** and **active_lines = [Identify actual executable units](_active_lines(functions=access(data, "functions")))**.
    - [Review the narrative measure of each unit](for ratios as ratio)
      We **line = [Read the measured source heading location](access(ratio, "line"))**.
      1. [This is an executable source unit](table.contains(list=`active_lines`, item=`line`)) and access(`ratio`, "ratio") <= 0.5
        We **ratio_gaps = [Retain this function and its actual measured ratio](table.append(list=`ratio_gaps`, item=`ratio`))**.
  1. not `parse_ok`
    We **diagnostics = [Retain the real parser failure](table.append(list=`diagnostics`, item=access(analysis, "error")))**.
  We **errors = 0** before examining the diagnostic severities.
  - [Keep errors separate from expected illustrative warnings](for diagnostics as diagnostic)
    We **severity = [Read a diagnostic severity when provided](table.get(map=`diagnostic`, key="severity"))**.
    1. `severity` == "error"
      We **errors = errors + 1**.
  We **ok = [Require all structural checks for this file](parse_ok and errors == 0 and len(ratio_gaps) == 0 and len(missing_contracts) == 0)**.
  1. not `ok`
    We **complete = False** to retain the incomplete batch state.

  `report` =

  | path | ok | diagnostics | ratio_gaps | missing_contracts |
  |---|---|---|---|---|
  | `path` | `ok` | `diagnostics` | `ratio_gaps` | `missing_contracts` |

  We **reports = [Retain this file evidence without its full source dump](table.append(list=`reports`, item=`report`))**.
We **count = [Retain the actual number of selected sources](len(`paths`))**.

`evidence` =

| complete | count | reports |
|---|---|---|
| `complete` | `count` | `reports` |

1. `complete`
  Return *[The complete structural evidence](result.ok(value=`evidence`))*.
We **failed = [Retain the incomplete authoring result](result.error(code="authoring.incomplete", message="inspect the per-file findings in value", operation="files.audit"))**.
Return *[Keep the report available on failure](table.put(in=`failed`, at="value", value=`evidence`))*.


## excerpt

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | UTF-8 artifact under --fs-root |
| start | number | 1 | First one-based line to inspect |
| count | number | 40 | Number of lines, from 1 through 200 |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | A record with total_lines and selected {line,text} records; each line is clipped to 2000 characters |

Use `path`, `start`=1 and `count`=40 when a generated HTML line or a large source file would overflow the agent's output window. Line numbers refer to the original file, so the excerpt can guide a precise later read or patch. This is a bounded inspection view: clipping does not alter the file or turn a truncated excerpt into suitable replacement source. Start and count must be positive integral numbers; selection past the end returns no lines. The native reader still validates UTF-8 and the granted filesystem root.

We **lines = [Read UTF-8 lines through the native filesystem](fs.read_lines(path=`path`))**.
1. `start` < 1 or `count` < 1 or `count` > 200
  Return *[Explain the supported inspection window](result.error(code="files.excerpt_window", message="start >= 1; count must be 1..200"))*.
We **selected = [Select only the requested line range](table.slice(list=`lines`, start=(`start` - 1), end=(`start` - 1 + `count`)))** and **records = [Start the source-location list](table.empty_list())**.
We **number = [Keep the original first line number](start)**.
- [Inspect each selected line](for selected as line)
  We **visible = [Limit one generated line to 2000 characters](re.replace(text=`line`, pattern="(?s)\\A(.{0,2000}).*\\z", with="$1"))**.

  `record` =

  | line | text |
  |---|---|
  | `number` | `visible` |

  We **records = [Retain the original location and readable text](table.append(list=`records`, item=`record`))**.
  We **number = [Advance the original line number](number + 1)**.
We **total = [Count the actual source lines](len(lines))**.

`window` =

| total_lines | lines |
|---|---|
| `total` | `records` |

After selecting this window, return *window*.
