---
markup: occlusion
title: lib/csv
description: RFC4180-ish CSV parse/stringify (Mid M4). First row = headers → list of maps.
---

CSV parse and stringify. First row is headers; rows become a list of maps.

Parse and serialize header-based CSV records.

Behavior and limits: Parsed cells are text; use table.cast for explicit types. stringify follows the first row field order. Use row records, not a column map.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Implementation: [src/host/csv.rs](../src/host/csv.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## parse

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Parse CSV text into a list of maps |

Use `text` as the inputs for this operation.

Effects: pure.

Parse CSV text into a list of maps.

Caller: [csv.parse] text=`raw`

Return *[Parse CSV text into a list of maps](> host_csv_parse text=`text`)*.

## stringify

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| rows | list | required | List of row records |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Serialize a list of maps to CSV text |

Use `rows` as the inputs for this operation.

Effects: pure.

Serialize a list of maps to CSV text.

Return *[Serialize a list of maps to CSV text](> host_csv_stringify rows=`rows`)*.

## try_parse

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text to read, transform or emit as described by this operation |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | map | Return {ok,value,error}; parse failures are recoverable data |

Use `text` as the inputs for this operation.

Effects: pure.

Call this function directly:

```sh
marqdo call lib/csv.mq.md try_parse 'text="name,n"' --effects none --json
```

Return {ok,value,error}; parse failures are recoverable data.

Return *[Return {ok,value,error}; parse failures are recoverable data](> host_try_csv_parse text=`text`)*.
