---
markup: occlusion
title: lib/toml
description: TOML parse/stringify with inline tables, multiline strings and arrays-of-tables. Dates become ISO text.
---

Read and write standard TOML configurations. Datetime scalars become ISO text; serialization does not preserve comments.

Read and write standard TOML configuration data.

Behavior and limits: Supports inline tables, multiline strings and arrays-of-tables. TOML dates become ISO text. Serialization rejects None and does not preserve original comments.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Implementation: [src/host/toml_ops.rs](../src/host/toml_ops.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## parse

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Parse TOML text into maps/lists/scalars |

Use `text` as the inputs for this operation.

Effects: pure.

Parse TOML text into maps/lists/scalars.

Caller: [toml.parse] text=`cfg`

Return *[Parse TOML text into maps/lists/scalars](> host_toml_parse text=`text`)*.

## stringify

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | map | required | Value to transform or store, without mutating the input |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | text | Serialize a map as TOML, rejecting None and unsupported value kinds; does not preserve source comments |

Use `value` as the inputs for this operation.

Effects: pure.

Serialize a map as TOML, rejecting None and unsupported value kinds; does not preserve source comments.

Return *[Serialize a map as TOML, rejecting None and unsupported value kinds; does not preserve source comments](> host_toml_stringify value=`value`)*.

## try_parse

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text to read, transform or emit as described by this operation |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | map | Recoverable TOML parsing supports inline tables, multiline strings and arrays-of-tables; datetime scalars become ISO text |

Use `text` as the inputs for this operation.

Effects: pure.

Call this function directly:

```sh
marqdo call lib/toml.mq.md try_parse 'text="name="Marqdo""' --effects none --json
```

Recoverable TOML parsing supports inline tables, multiline strings and arrays-of-tables; datetime scalars become ISO text.

Return *[Recoverable TOML parsing supports inline tables, multiline strings and arrays-of-tables; datetime scalars become ISO text](> host_toml_try_parse text=`text`)*.
