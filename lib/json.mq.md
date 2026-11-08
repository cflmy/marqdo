---
markup: occlusion
title: lib/json
description: JSON parse/stringify (shared EN/ZH import). Not a dict builder — use lib/table.
---

Parse and stringify JSON. For maps/lists prefer GFM tables and lib/table put; do not chain json.set / json.append in new code.

Parse and serialize JSON at the data boundary.

Behavior and limits: Use table for collection operations. JSON boundary rejects unsafe integers, duplicate keys and nonfinite numbers; do not turn opaque handles into guessed records.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Implementation: [src/host/json.rs](../src/host/json.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## parse

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Parse JSON text into a Marqdo value |

Use `text` as the inputs for this operation.

Effects: pure.

Parse JSON text into a Marqdo value.

Caller: [json.parse] text="{}"

Return *[Parse JSON text into a Marqdo value](> host_json_parse text=`text`)*.

## stringify

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | any | required | Value to transform or store, without mutating the input |
| indent | any | None | JSON indentation setting; None uses compact output |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Serialize value to JSON text; optional indent |

Use `value`, `indent`=None as the inputs for this operation.

Effects: pure.

Serialize value to JSON text; optional indent.

Return *[Serialize value to JSON text; optional indent](> host_json_stringify value=`value` indent=`indent`)*.

## get

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | any | required | Value to transform or store, without mutating the input |
| key | text | required | Field name or key required by this operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Shallow get from JSON object value at key |

Use `value`, `key` as the inputs for this operation.

Effects: pure.

Shallow get from JSON object value at key. Prefer link-index get or table.get in new code.

Return *[Shallow get from JSON object value at key](> host_json_get value=`value` key=`key`)*.

## keys

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | any | required | Value to transform or store, without mutating the input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Keys of JSON object value |

Use `value` as the inputs for this operation.

Effects: pure.

Keys of JSON object value.

Return *[Keys of JSON object value](> host_json_keys value=`value`)*.

## quote

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | JSON-quote string text (for building request bodies) |

Use `text` as the inputs for this operation.

Effects: pure.

JSON-quote string text (for building request bodies).

Return *[JSON-quote string text (for building request bodies)](> host_json_quote text=`text`)*.

## set

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| map | any | required | Input string-keyed map |
| key | text | required | Field name or key required by this operation |
| value | any | required | Value to transform or store, without mutating the input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Compatibility shallow set |

Use `map`, `key`, `value` as the inputs for this operation.

Effects: pure.

Compatibility shallow set. Prefer table.put / table.set.

Return *[Compatibility shallow set](> host_map_set map=`map` key=`key` value=`value`)*.

## append

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | any | required | Input list; updates return a new list |
| item | any | required | Element to insert, replace or compare |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Compatibility append |

Use `list`, `item` as the inputs for this operation.

Effects: pure.

Compatibility append. Prefer table.append.

Return *[Compatibility append](> host_list_append list=`list` item=`item`)*.

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
marqdo call lib/json.mq.md try_parse 'text="{"name":"Marqdo","n":2}"' --effects none --json
```

Return {ok,value,error}; parse failures are recoverable data.

Return *[Return {ok,value,error}; parse failures are recoverable data](> host_try_json_parse text=`text`)*.
