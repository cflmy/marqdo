---
markup: occlusion
title: lib/subtask
description: Concurrent subtasks — file, function, or foreign code
---

Concurrent subtasks: file, function, or foreign code.

Run and observe concurrent work with explicit ownership.

Behavior and limits: File batches inherit parent caps and root. Each Result.ok reports completion of the wait protocol; inspect value.code for child exit success. wait_timeout keeps its handle unless cancel is requested. OS tasks can be killed/reaped; function-thread cancellation is logical only.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Relative IO paths use --fs-root, or the called file directory when omitted. Writes require explicit fs.write grants, subprocesses require process.exec, and pure transforms require no IO grant.

Implementation: [src/host/subtask.rs](../src/host/subtask.rs)、[src/host/task_ops.rs](../src/host/task_ops.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## spawn

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | any | None | Path relative to the granted filesystem root |
| fn | any | None | Marqdo function target used instead of a file task |
| args | any | None | Arguments supplied to the selected task |
| code | any | None | Bound code-fence value; supply lang and source for inline text |
| lang | any | None | Registered interpreter language identifier, such as python |
| source | any | None | Source text to run with the selected interpreter |
| stdin | any | None | Text written to the child process standard input |
| quiet | bool | True | Whether to suppress child output while retaining its result |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Start a subtask |

Use `path`=None, `fn`=None, `args`=None, `code`=None, `lang`=None, `source`=None, `stdin`=None, `quiet`=True as the inputs for this operation.

Effects: process.exec.

Start a subtask. File children are silent by default (quiet=True: piped capture). Set quiet=False to inherit parent stdout/stderr. File wait returns {code, value} plus optional stdout/stderr tails when quiet-captured. value is the child main return (not stdout).

Caller: [subtask.spawn] path="child.mq.md"

Return *[Start a subtask](> host_subtask_spawn path=`path` fn=`fn` args=`args` code=`code` lang=`lang` source=`source` stdin=`stdin` quiet=`quiet`)*.

## poll

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | any | required | Identifier returned by the corresponding create/spawn operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Non-blocking status for subtask id |

Use `id` as the inputs for this operation.

Effects: process.exec.

Non-blocking status for subtask id.

Return *[Non-blocking status for subtask id](> host_subtask_poll id=`id`)*.

## wait

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | any | required | Identifier returned by the corresponding create/spawn operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Block until subtask id finishes |

Use `id` as the inputs for this operation.

Effects: process.exec.

Block until subtask id finishes.

Return *[Block until subtask id finishes](> host_subtask_join id=`id`)*.

## kill

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | any | required | Identifier returned by the corresponding create/spawn operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Kill subtask id |

Use `id` as the inputs for this operation.

Effects: process.exec.

Kill subtask id.

Return *[Kill subtask id](> host_subtask_kill id=`id`)*.

## wait_all

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Wait until every spawned subtask finishes |

Effects: process.exec.

Wait until every spawned subtask finishes.

Return *[Wait until every spawned subtask finishes](host_subtask_wait_all())*.

## wait_timeout

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | number | required | Identifier returned by the corresponding create/spawn operation |
| timeout_ms | number | 30000 | Timeout in milliseconds; inspect the operation result after waiting |
| cancel | bool | False | Whether a timeout also cancels the outstanding task |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | map | Wait as Result; timeout retains the handle unless cancel=True |

Use `id`, `timeout_ms`=30000, `cancel`=False as the inputs for this operation.

Effects: process.exec.

Wait as Result; timeout retains the handle unless cancel=True. Cancellation reaps process tasks; function-thread cancellation is logical only.

Return *[Wait as Result; timeout retains the handle unless cancel=True](> host_subtask_wait_timeout id=`id` timeout_ms=`timeout_ms` cancel=`cancel`)*.

## cancel_all

| 返回 | 类型 | 说明 |
|------|------|------|
| result | number | Cancel all subtasks, reaping process tasks; return count |

Effects: process.exec.

Cancel all subtasks, reaping process tasks; return count. Function threads only support logical cancellation.

Return *[Cancel all subtasks, reaping process tasks; return count](> host_subtask_cancel_all )*.

## run_many

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| tasks | list | required | Ordered file task records containing path and optional args |
| limit | number | 4 | Maximum simultaneous file tasks, from 1 through 64 |
| timeout_ms | number | 30000 | Timeout in milliseconds; inspect the operation result after waiting |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Run path/args file tasks with bounded concurrency; return ordered Results |

Use `tasks`, `limit`=4, `timeout_ms`=30000 as the inputs for this operation.

Effects: process.exec.

Call this function directly:

```sh
marqdo call lib/subtask.mq.md run_many 'tasks=[{"path":"examples/stdlib-handbook/worker.mq.md"}]' limit=2 --effects process.exec --fs-root . --json
```

Run path/args file tasks with bounded concurrency; return ordered Results. Timed-out children are killed and reaped; at most 10000 tasks, 64 workers; inherit parent caps and filesystem root.

Return *[Run path/args file tasks with bounded concurrency; return ordered Results](> host_subtask_run_many tasks=`tasks` limit=`limit` timeout_ms=`timeout_ms`)*.
