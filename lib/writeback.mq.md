---
markup: occlusion
title: lib/writeback
description: Persist run output in .mq.md (Jupyter-style writeback)
---

Persist run output in .mq.md (Jupyter-style writeback)

Read and maintain persisted execution-output slots.

Behavior and limits: record/get default to the current entry call site. Named slots isolate outputs. This API does not edit arbitrary document sections; use document for structural transforms.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Relative IO paths use --fs-root, or the called file directory when omitted. Writes require explicit fs.write grants, subprocesses require process.exec, and pure transforms require no IO grant.

Implementation: [src/host/writeback.rs](../src/host/writeback.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## record

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | any | required | Value to transform or store, without mutating the input |
| at_end | bool | False | Whether to append to the end of the target block |
| key | any | None | Field name or key required by this operation |
| line | any | None | Text line to add to the managed writeback block |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Write `value` into the entry file |

Use `value`, `at_end`=False, `key`=None, `line`=None as the inputs for this operation.

Effects: fs.write.

Write `value` into the entry file. Image-like values (`.png` / `.svg` / … or `data:image/…`) become `![marqdo-out](…)` (or `![marqdo-out key](…)`); other values use `<!-- marqdo-out … -->`. Default: insert or replace the block immediately below the call; `at_end=true` replaces the unkeyed EOF block. With `key=` (e.g. `ok` / `error`), replace or create that named slot only — slots do not overwrite each other. Optional `line=` anchors the block under that source line (needed when writing from a subtask).

Return *[Write value into the entry file](> host_writeback_record value=`value` at_end=`at_end` key=`key` line=`line`)*.

## get

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| at_end | bool | False | Whether to append to the end of the target block |
| key | any | None | Field name or key required by this operation |
| line | any | None | Text line to add to the managed writeback block |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Read persisted output for the current call site (adjacent unkeyed block), the unkeyed EOF block when `at_end=true`, or a named slot when `key=` is set |

Use `at_end`=False, `key`=None, `line`=None as the inputs for this operation.

Effects: fs.read.

Read persisted output for the current call site (adjacent unkeyed block), the unkeyed EOF block when `at_end=true`, or a named slot when `key=` is set.

Return *[Read persisted output for the current call site (adjacent unkeyed block), the unkeyed EOF block when at_end=true, or a named slot when key= is set](> host_writeback_get at_end=`at_end` key=`key` line=`line`)*.

## clear

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| at_end | bool | False | Whether to append to the end of the target block |
| key | any | None | Field name or key required by this operation |
| line | any | None | Text line to add to the managed writeback block |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Remove the adjacent, EOF, or named output block |

Use `at_end`=False, `key`=None, `line`=None as the inputs for this operation.

Effects: fs.write.

Remove the adjacent, EOF, or named output block.

Return *[Remove the adjacent, EOF, or named output block](> host_writeback_clear at_end=`at_end` key=`key` line=`line`)*.

## ensure

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| key | text | required | Field name or key required by this operation |
| placeholder | any | "pending" | Placeholder used when creating the managed block |
| line | any | None | Text line to add to the managed writeback block |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Create a named slot with `placeholder` if missing |

Use `key`, `placeholder`="pending", `line`=None as the inputs for this operation.

Effects: fs.write.

Create a named slot with `placeholder` if missing. Never overwrites an existing slot body.

Return *[Create a named slot with placeholder if missing](> host_writeback_ensure key=`key` placeholder=`placeholder` line=`line`)*.

## list

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | List managed output slots from the current entry source without writing or reading another file |

Effects: fs.read.

List all output blocks in the entry file (includes `key` when named).

List managed output slots from the current entry source without writing or reading another file.

Return *[List managed output slots from the current entry source without writing or reading another file](host_writeback_list())*.

## scan_path

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path relative to the granted filesystem root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | List writeback slots in any file (HTML comment and `![marqdo-out](…)` image forms; workbook inspection) |

Use `path` as the inputs for this operation.

Effects: fs.read.

Call this function directly:

```sh
marqdo call lib/writeback.mq.md scan_path 'path="lib/writeback.mq.md"' --effects fs.read --fs-root . --json
```

List writeback slots in any file (HTML comment and `![marqdo-out](…)` image forms; workbook inspection).

Return *[List writeback slots in any file (HTML comment and !marqdo-out(…) image forms; workbook inspection)](> host_writeback_scan_path path=`path`)*.
