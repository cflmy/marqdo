---
markup: occlusion
title: lib/foreign
description: Run bound code fences and inline sources via local interpreters
import:
  table: lib/table.mq.md
---

Run bound code fences and inline sources via local interpreters.

Bridge explicit values to external language processes and sessions.

Behavior and limits: Configured languages are not proof that interpreters are installed. invoke exposes exit/timeout/output limits. External execution requires process.exec; close owned persistent sessions.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Implementation: [src/host/foreign.rs](../src/host/foreign.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## set_cmd

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| lang | text | required | Registered interpreter language identifier, such as python |
| cmd | any | required | Executable command or interpreter override |
| args | any | None | Argument list passed to the called command or function |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Bind interpreter command cmd for language lang; optional extra argv |

Use `lang`, `cmd`, `args`=None as the inputs for this operation.

Effects: pure.

Bind interpreter command cmd for language lang; optional extra argv.

Return *[Bind interpreter command cmd for language lang; optional extra argv](> host_foreign_set_cmd lang=`lang` cmd=`cmd` args=`args`)*.

## run

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| code | any | required | Bound code-fence value; supply lang and source for inline text |
| stdin | any | None | Text written to the child process standard input |
| env | any | None | Environment variable/value record for the child process |
| cwd | any | None | Child process working directory |
| timeout | any | None | Execution timeout in seconds; None uses the host default |
| args | any | None | Argument list passed to the called command or function |
| max_output | any | None | Maximum captured output bytes; None uses the host default |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Run a bound code fence value; optional stdin, env map, cwd, timeout (seconds), extra argv and output cap |

Use `code`, `stdin`=None, `env`=None, `cwd`=None, `timeout`=None, `args`=None, `max_output`=None as the inputs for this operation.

Effects: process.exec.

Run a bound code fence value; optional stdin, env map, cwd, timeout (seconds),
extra argv and output cap. Returns stdout text; non-zero exit is an error.

Caller: [foreign.run] code=`block`

Return *[Run a bound code fence value; optional stdin, env map, cwd, timeout (seconds), extra argv and output cap](> host_foreign_run code=`code` stdin=`stdin` env=`env` cwd=`cwd` timeout=`timeout` args=`args` max_output=`max_output`)*.

## run_lang

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| lang | text | required | Registered interpreter language identifier, such as python |
| source | any | required | Source text to run with the selected interpreter |
| stdin | any | None | Text written to the child process standard input |
| env | any | None | Environment variable/value record for the child process |
| cwd | any | None | Child process working directory |
| timeout | any | None | Execution timeout in seconds; None uses the host default |
| args | any | None | Argument list passed to the called command or function |
| max_output | any | None | Maximum captured output bytes; None uses the host default |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Run source text with language lang; same options as run |

Use `lang`, `source`, `stdin`=None, `env`=None, `cwd`=None, `timeout`=None, `args`=None, `max_output`=None as the inputs for this operation.

Effects: process.exec.

Run source text with language lang; same options as run.

Return *[Run source text with language lang; same options as run](> host_foreign_run_lang lang=`lang` source=`source` stdin=`stdin` env=`env` cwd=`cwd` timeout=`timeout` args=`args` max_output=`max_output`)*.

## invoke

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| code | any | None | Bound code-fence value; supply lang and source for inline text |
| lang | any | None | Registered interpreter language identifier, such as python |
| source | any | None | Source text to run with the selected interpreter |
| stdin | any | None | Text written to the child process standard input |
| input | any | None | Structured value JSON-encoded on stdin; cannot combine with stdin |
| env | any | None | Environment variable/value record for the child process |
| cwd | any | None | Child process working directory |
| timeout | any | None | Execution timeout in seconds; None uses the host default |
| args | any | None | Argument list passed to the called command or function |
| output | any | "auto" | Result decoding mode: auto, text, json or lines |
| max_output | any | None | Maximum captured output bytes; None uses the host default |
| cmd | any | None | Executable command or interpreter override |
| keep | any | False | Whether to retain the generated script for inspection |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Run code or lang+source and return a record map instead of raising on process failure |

Use `code`=None, `lang`=None, `source`=None, `stdin`=None, `input`=None, `env`=None, `cwd`=None, `timeout`=None, `args`=None, `output`="auto", `max_output`=None, `cmd`=None, `keep`=False as the inputs for this operation.

Effects: process.exec.

Run code or lang+source and return a record map instead of raising on process
failure. Fields: ok, code, stdout, stderr, ms, lang, cmd, timed_out, truncated,
result, result_error, error, script, cached.

`input` is encoded as JSON on stdin (mutually exclusive with `stdin`). `output`
is auto | text | json | lines. `cmd` overrides the interpreter for this call.

Return *[Run code or lang+source and return a record map instead of raising on process failure](> host_foreign_invoke code=`code` lang=`lang` source=`source` stdin=`stdin` input=`input` env=`env` cwd=`cwd` timeout=`timeout` args=`args` output=`output` max_output=`max_output` cmd=`cmd` keep=`keep`)*.

## probe

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| lang | any | None | Registered interpreter language identifier, such as python |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Probe one language, or every known language when lang is omitted |

Use `lang`=None as the inputs for this operation.

Effects: process.exec.

Probe one language, or every known language when lang is omitted. Returns
{lang, cmd, path, ok, version, error} (a list when lang is omitted).

Return *[Probe one language, or every known language when lang is omitted](> host_foreign_probe lang=`lang`)*.

## register

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| lang | text | required | Registered interpreter language identifier, such as python |
| cmd | any | None | Executable command or interpreter override |
| args | any | None | Argument list passed to the called command or function |
| template | any | None | Interpreter launch template |
| ext | any | None | Temporary script filename extension |
| env | any | None | Environment variable/value record for the child process |
| version_args | any | None | Arguments used to query the interpreter version |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Register a language invocation |

Use `lang`, `cmd`=None, `args`=None, `template`=None, `ext`=None, `env`=None, `version_args`=None as the inputs for this operation.

Effects: pure.

Register a language invocation. Provide cmd (+args) or template (placeholders
{file}, {dir}, {base}, {inline}, {lang}, {cwd}); they are mutually exclusive.

Return *[Register a language invocation](> host_foreign_register lang=`lang` cmd=`cmd` args=`args` template=`template` ext=`ext` env=`env` version_args=`version_args`)*.

## unset

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| lang | text | required | Registered interpreter language identifier, such as python |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Remove a process-local registration |

Use `lang` as the inputs for this operation.

Effects: pure.

Remove a process-local registration. Presets/config/env still apply.

Return *[Remove a process-local registration](> host_foreign_unset lang=`lang`)*.

## load_config

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | any | None | Path relative to the granted filesystem root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Load .marqdo/foreign.toml (or an explicit path); returns the number of language tables read |

Use `path`=None as the inputs for this operation.

Effects: fs.read.

Load .marqdo/foreign.toml (or an explicit path); returns the number of language
tables read.

Return *[Load .marqdo/foreign.toml (or an explicit path); returns the number of language tables read](> host_foreign_load_config path=`path`)*.

## call

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| code | any | None | Bound code-fence value; supply lang and source for inline text |
| lang | any | None | Registered interpreter language identifier, such as python |
| source | any | None | Source text to run with the selected interpreter |
| input | any | None | Structured value JSON-encoded on stdin; cannot combine with stdin |
| stdin | any | None | Text written to the child process standard input |
| env | any | None | Environment variable/value record for the child process |
| cwd | any | None | Child process working directory |
| timeout | any | None | Execution timeout in seconds; None uses the host default |
| args | any | None | Argument list passed to the called command or function |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | JSON-in / JSON-out call: encode `input` on stdin, parse stdout JSON and return the value (empty stdout -> None) |

Use `code`=None, `lang`=None, `source`=None, `input`=None, `stdin`=None, `env`=None, `cwd`=None, `timeout`=None, `args`=None as the inputs for this operation.

Effects: process.exec.

JSON-in / JSON-out call: encode `input` on stdin, parse stdout JSON and return
the value (empty stdout -> None). Raises on process failure or invalid JSON.

Return *[JSON-in / JSON-out call: encode input on stdin, parse stdout JSON and return the value (empty stdout -> None)](> host_foreign_call code=`code` lang=`lang` source=`source` input=`input` stdin=`stdin` env=`env` cwd=`cwd` timeout=`timeout` args=`args`)*.

## run_many

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| runs | any | required | Ordered per-run code or language/source records |
| concurrency | any | 4 | Maximum number of concurrent operations |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Run a list of code values or invoke-option maps in parallel; returns records in input order |

Use `runs`, `concurrency`=4 as the inputs for this operation.

Effects: process.exec.

Run a list of code values or invoke-option maps in parallel; returns records in
input order. One failing item does not stop the others.

Return *[Run a list of code values or invoke-option maps in parallel; returns records in input order](> host_foreign_run_many runs=`runs` concurrency=`concurrency`)*.

## call_many

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| inputs | any | required | Input values for the batched calls |
| code | any | None | Bound code-fence value; supply lang and source for inline text |
| lang | any | None | Registered interpreter language identifier, such as python |
| source | any | None | Source text to run with the selected interpreter |
| cmd | any | None | Executable command or interpreter override |
| env | any | None | Environment variable/value record for the child process |
| cwd | any | None | Child process working directory |
| timeout | any | None | Execution timeout in seconds; None uses the host default |
| args | any | None | Argument list passed to the called command or function |
| max_output | any | None | Maximum captured output bytes; None uses the host default |
| concurrency | any | 4 | Maximum number of concurrent operations |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Apply one code/function to each input in parallel; returns parsed JSON results in input order |

Use `inputs`, `code`=None, `lang`=None, `source`=None, `cmd`=None, `env`=None, `cwd`=None, `timeout`=None, `args`=None, `max_output`=None, `concurrency`=4 as the inputs for this operation.

Effects: process.exec.

Apply one code/function to each input in parallel; returns parsed JSON results
in input order. Raises with the failing index after all workers finish.

Return *[Apply one code/function to each input in parallel; returns parsed JSON results in input order](> host_foreign_call_many inputs=`inputs` code=`code` lang=`lang` source=`source` cmd=`cmd` env=`env` cwd=`cwd` timeout=`timeout` args=`args` max_output=`max_output` concurrency=`concurrency`)*.

## session

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| kernels | any | None | Kernel/session specifications for persistent interpreters |
| cells | any | None | Code cells to evaluate in a persistent interpreter |
| values | any | None | Variable/value record to inject into a session |
| inputs | any | None | Input values for the batched calls |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Declare a document view session |

Use `kernels`=None, `cells`=None, `values`=None, `inputs`=None as the inputs for this operation.

Effects: pure.

Declare a document view session. `kernels` and `cells` are ordinary tables:
kernels are `|名称|语言|命令|` (ZH or EN), cells are `|单元|内核|`.
Returns a foreign session handle consumed by `view`; without this handle the
view has no persistent kernel state.

We retain **`p` = [Empty map](table.empty_map())**.
We retain **`p` = [Set the kind field in the new record](table.put(in=`p`, at="kind", value="foreign-session"))**.
We retain **`p` = [Set the kernels field in the new record](table.put(in=`p`, at="kernels", value=`kernels`))**.
We retain **`p` = [Set the cells field in the new record](table.put(in=`p`, at="cells", value=`cells`))**.
We retain **`p` = [Set the values field in the new record](table.put(in=`p`, at="values", value=`values`))**.
We retain **`p` = [Set the inputs field in the new record](table.put(in=`p`, at="inputs", value=`inputs`))**.
Return *p*.

## session_open

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| lang | text | required | Registered interpreter language identifier, such as python |
| cmd | any | None | Executable command or interpreter override |
| env | any | None | Environment variable/value record for the child process |
| cwd | any | None | Child process working directory |
| timeout | any | None | Execution timeout in seconds; None uses the host default |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Start a persistent Python/Node kernel; returns an opaque `handle` value |

Use `lang`, `cmd`=None, `env`=None, `cwd`=None, `timeout`=None as the inputs for this operation.

Effects: process.exec.

Start a persistent Python/Node kernel; returns an opaque `handle` value.
Text fabrications are refused by session ops; handles cannot cross the JSON
bridge. `timeout` is the default per-eval timeout.

Return *[Start a persistent Python/Node kernel; returns an opaque handle value](> host_foreign_session_open lang=`lang` cmd=`cmd` env=`env` cwd=`cwd` timeout=`timeout`)*.

## session_eval

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | any | required | Identifier returned by the corresponding create/spawn operation |
| code | any | None | Bound code-fence value; supply lang and source for inline text |
| source | any | None | Source text to run with the selected interpreter |
| input | any | None | Structured value JSON-encoded on stdin; cannot combine with stdin |
| stdin | any | None | Text written to the child process standard input |
| timeout | any | None | Execution timeout in seconds; None uses the host default |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Evaluate code (a code value) or source text in session id; returns a record with id/alive |

Use `id`, `code`=None, `source`=None, `input`=None, `stdin`=None, `timeout`=None as the inputs for this operation.

Effects: process.exec.

Evaluate code (a code value) or source text in session id; returns a record
with id/alive. `input` is JSON-encoded for the kernel; `stdin` is raw program
stdin for this evaluation.

Return *[Evaluate code (a code value) or source text in session id; returns a record with id/alive](> host_foreign_session_eval id=`id` code=`code` source=`source` input=`input` stdin=`stdin` timeout=`timeout`)*.

## session_set

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | any | required | Identifier returned by the corresponding create/spawn operation |
| name | text | required | Name to resolve, register or look up |
| value | any | required | Value to transform or store, without mutating the input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Set one kernel variable from a Marqdo value (JSON copy) |

Use `id`, `name`, `value` as the inputs for this operation.

Effects: process.exec.

Set one kernel variable from a Marqdo value (JSON copy).

Return *[Set one kernel variable from a Marqdo value (JSON copy)](> host_foreign_session_set id=`id` name=`name` value=`value`)*.

## session_get

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | any | required | Identifier returned by the corresponding create/spawn operation |
| name | text | required | Name to resolve, register or look up |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Read one kernel variable as a Marqdo value |

Use `id`, `name` as the inputs for this operation.

Effects: process.exec.

Read one kernel variable as a Marqdo value.

Return *[Read one kernel variable as a Marqdo value](> host_foreign_session_get id=`id` name=`name`)*.

## session_names

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | any | required | Identifier returned by the corresponding create/spawn operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | List kernel variable names |

Use `id` as the inputs for this operation.

Effects: process.exec.

List kernel variable names.

Return *[List kernel variable names](> host_foreign_session_names id=`id`)*.

## session_close

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | any | required | Identifier returned by the corresponding create/spawn operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Consume (affine close) a live session handle: kill and remove it |

Use `id` as the inputs for this operation.

Effects: process.exec.

Consume (affine close) a live session handle: kill and remove it. Returns
`True` on success. Closing an unknown or already-closed id is a fault
(`foreign session … not found`), matching use-after-close on eval/set/get.

Return *[Consume (affine close) a live session handle: kill and remove it](> host_foreign_session_close id=`id`)*.

## session_interrupt

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | any | required | Identifier returned by the corresponding create/spawn operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Interrupt the current evaluation of a persistent kernel |

Use `id` as the inputs for this operation.

Effects: process.exec.

Interrupt the current evaluation of a persistent kernel. The session remains
usable for the next evaluation when the worker supports interruption.

Return *[Interrupt the current evaluation of a persistent kernel](> host_foreign_session_interrupt id=`id`)*.

## session_list

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | List live sessions with id/lang/cmd/evals/alive/started_ms |

Effects: process.exec.

List live sessions with id/lang/cmd/evals/alive/started_ms.

Return *[List live sessions with id/lang/cmd/evals/alive/started_ms](host_foreign_session_list())*.

## langs

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | List configured foreign languages |

Effects: pure.

Call this function directly:

```sh
marqdo call lib/foreign.mq.md langs  --effects none --json
```

List configured foreign languages.

Return configured language definitions without launching interpreters. Use probe separately to check executable availability.

Return *[List configured foreign languages](host_foreign_langs())*.

## oracle_journal

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Return this document evaluation's admitted eval/get/decode observations as ordinary records with index, channel, value |

Effects: pure.

Return this document evaluation's admitted eval/get/decode observations as
ordinary records with index, channel, value. Opening, setting and closing do
not append observations.

Return *[Return this document evaluation's admitted eval/get/decode observations as ordinary records with index, channel, value](> host_oracle_journal )*.

## replay_check

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| expected | any | required | Expected observation record used for replay comparison |
| observed | any | required | Actual observation record used for replay comparison |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Compare two oracle journals without executing foreign code |

Use `expected`, `observed` as the inputs for this operation.

Effects: pure.

Compare two oracle journals without executing foreign code. Returns ok and
the least mismatching index, with the expected and observed records.

Return *[Compare two oracle journals without executing foreign code](> host_oracle_replay_check expected=`expected` observed=`observed`)*.
