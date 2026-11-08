---
markup: occlusion
title: lib/sys
description: English system wrappers (load_dotenv accepts optional path via host alias)
---

English system wrappers (load_dotenv accepts optional path via host alias)

Inspect execution metadata and interact with the host system.

Behavior and limits: Metadata describes the entry artifact. module_source is entry source, not the imported library. exec needs process.exec; environment operations declare separate effects.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Implementation: [src/host/sys.rs](../src/host/sys.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## env_get

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| name | text | required | Environment variable name |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Get environment variable name |

Use `name` as the inputs for this operation.

Effects: environment.read.

Get environment variable name.

Return *[Get environment variable name](> host_env_get name=`name`)*.

## env_set

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| name | text | required | Environment variable name |
| value | any | required | Value to transform or store, without mutating the input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Set environment variable name to value |

Use `name`, `value` as the inputs for this operation.

Effects: environment.write.

Set environment variable name to value.

Return *[Set environment variable name to value](> host_env_set name=`name` value=`value`)*.

## load_dotenv

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | any | None | Path relative to the granted filesystem root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Load `.env` (optional named arg `path=`) |

Use `path`=None as the inputs for this operation.

Effects: environment.write,fs.read.

Load `.env` (optional named arg `path=`). Existing process env is not overridden.

Return *[Load .env (optional named arg path=)](> host_dotenv_load path=`path`)*.

## args

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Process argv list |

Effects: pure.

Process argv list.

Return *[Process argv list](host_args())*.

## cwd

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Current working directory |

Effects: pure.

Current working directory.

Return *[Current working directory](host_cwd())*.

## meta

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Bound Artifact Metadata from the entry `.mq.md` (map) |

Effects: pure.

Call this function directly:

```sh
marqdo call lib/sys.mq.md meta  --effects none --json
```

Bound Artifact Metadata from the entry `.mq.md` (map). See `doc/design/binding.md`.

Return *[Bound Artifact Metadata from the entry .mq.md (map)](host_meta())*.

## meta_get

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| key | text | required | Field name or key required by this operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Lookup one metadata key; missing → None |

Use `key` as the inputs for this operation.

Effects: pure.

Lookup one metadata key; missing → None.

Return *[Lookup one metadata key; missing → None](> host_meta_get key=`key`)*.

## artifact_kind

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Classify entry Artifact Metadata: `web` / `endpoint` / `prompt` / `other`, or None if no `type`/`类型` |

Effects: pure.

Classify entry Artifact Metadata: `web` / `endpoint` / `prompt` / `other`, or None if no `type`/`类型`. ADR 0007.

Return *[Classify entry Artifact Metadata: web / endpoint / prompt / other, or None if no type/类型](host_artifact_kind())*.

## module_source

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Entry `.mq.md` source text (for document-as-prompt) |

Effects: pure.

Entry `.mq.md` source text (for document-as-prompt). Empty string if unset.

Return *[Entry .mq.md source text (for document-as-prompt)](host_module_source())*.

## exit

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| code | any | required | Process exit status; zero means success |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Exit process with code |

Use `code` as the inputs for this operation.

Effects: process.control.

Exit process with code.

Return *[Exit process with code](> host_exit code=`code`)*.

## exec

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| cmd | any | required | Executable command or interpreter override |
| args | any | None | Argument list passed to the called command or function |
| capture | any | False | Whether to return command output instead of forwarding it |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Run a process |

Use `cmd`, `args`=None, `capture`=False as the inputs for this operation.

Effects: process.exec.

Run a process. Default return is the exit code (int).

With `capture=True`, return a map `{code, stdout, stderr}` (stdout/stderr may be truncated for huge output).

Inside `**…**` prefer bare `args=args` (variable); `` args=`args` `` is also accepted.

Return *[Run a process](> host_exec cmd=`cmd` args=`args` capture=`capture`)*.

## stream_publish

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| event | any | required | Structured event to publish to the active stream |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Publish a stream event map onto the process EventBus (`marqdo view` SSE) |

Use `event` as the inputs for this operation.

Effects: output.write.

Publish a stream event map onto the process EventBus (`marqdo view` SSE). No-op when nothing is subscribed.

Return *[Publish a stream event map onto the process EventBus (marqdo view SSE)](> host_stream_publish event=`event`)*.
