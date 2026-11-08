---
markup: occlusion
title: lib/path
description: Path algebra (Mid M2). No I/O; OS separators.
---

Path algebra only. No I/O; separators follow the host OS.

Manipulate paths without accessing the filesystem.

Behavior and limits: Normalization is lexical, not an authorization check. IO paths are checked by the host filesystem root.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Implementation: [src/host/path.rs](../src/host/path.rs)、[src/host/fs_ops.rs](../src/host/fs_ops.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## join

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| a | any | required | Base path |
| b | any | required | Path segment appended to the base |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Join path segments a and b |

Use `a`, `b` as the inputs for this operation.

Effects: pure.

Join path segments a and b.

Caller: [path.join] a="a" b="b"

Return *[Join path segments a and b](> host_path_join a=`a` b=`b`)*.

## split

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path relative to the granted filesystem root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Split path into parts |

Use `path` as the inputs for this operation.

Effects: pure.

Split path into parts.

Return *[Split path into parts](> host_path_split path=`path`)*.

## file_name

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path relative to the granted filesystem root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Final file name of path |

Use `path` as the inputs for this operation.

Effects: pure.

Final file name of path.

Return *[Final file name of path](> host_path_file_name path=`path`)*.

## parent

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path relative to the granted filesystem root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Parent directory of path |

Use `path` as the inputs for this operation.

Effects: pure.

Parent directory of path.

Return *[Parent directory of path](> host_path_parent path=`path`)*.

## extension

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path relative to the granted filesystem root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | File extension of path |

Use `path` as the inputs for this operation.

Effects: pure.

File extension of path.

Return *[File extension of path](> host_path_extension path=`path`)*.

## normalize

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path relative to the granted filesystem root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Normalize path (dot segments) |

Use `path` as the inputs for this operation.

Effects: pure.

Normalize path (dot segments).

Return *[Normalize path (dot segments)](> host_path_normalize path=`path`)*.

## is_absolute

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path relative to the granted filesystem root |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | bool | Whether path is absolute |

Use `path` as the inputs for this operation.

Effects: pure.

Whether path is absolute.

Return *[Whether path is absolute](> host_path_is_absolute path=`path`)*.

## relative

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path within the granted filesystem root |
| base | text | required | Base directory for relative path calculation |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | text | Compute a lexical path relative to base; both arguments must share absolute/relative form |

Use `path`, `base` as the inputs for this operation.

Effects: pure.

Compute a lexical path relative to base; both arguments must share absolute/relative form. No disk access.

Return *[Compute a lexical path relative to base; both arguments must share absolute/relative form](> host_path_relative path=`path` base=`base`)*.

## with_extension

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | Path within the granted filesystem root |
| extension | text | required | New extension, with or without a leading dot |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | text | Replace the last extension; empty extension removes it |

Use `path`, `extension` as the inputs for this operation.

Effects: pure.

Call this function directly:

```sh
marqdo call lib/path.mq.md with_extension 'path="notes/draft.txt"' 'extension="md"' --effects none --json
```

Replace the last extension; empty extension removes it.

Return *[Replace the last extension; empty extension removes it](> host_path_with_extension path=`path` extension=`extension`)*.
