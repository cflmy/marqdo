---
markup: occlusion
title: lib/table
description: >-
  List and map helpers. Prefer put (1-based path, same nesting as access reads);
  at/set_at/slice are 0-based. Use lib/json only for parse/stringify.
---

List and map helpers. Prefer put for nested writes; authors should read with access reads, not new footnote indexes.

Transform lists, maps and row records using explicit data shapes.

Behavior and limits: put/access integer paths are one-based; at/slice/insert/remove_at are zero-based. Updates return new values: keep the result. rows is a length alias; from_columns/to_columns perform shape conversion.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Implementation: [src/host/collection.rs](../src/host/collection.rs)、[src/host/collection_ops.rs](../src/host/collection_ops.rs)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## put

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| in | any | required | Input collection; None seeds a new record |
| at | any | required | Text key, one-based index, or nested path of keys and indices |
| value | any | required | Value to transform or store, without mutating the input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Update one element of collection in at path at to value |

Use `in`, `at`, `value` as the inputs for this operation.

Effects: pure.

Update one element of collection in at path at to value.

at is a text key, a 1-based list index, or a list of keys/indices (same nesting as access reads).

Caller: [table.put] in=None at="Authorization" value="Bearer-x"

Return *[Update one element of collection in at path at to value](> host_collection_put in=`in` at=`at` value=`value`)*.

## len

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| xs | any | required | Input list or collection, as required by this operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Length of list or map xs |

Use `xs` as the inputs for this operation.

Effects: pure.

Length of list or map xs.

Return *[Len](len(`xs`))*.

## rows

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| xs | any | required | Input list or collection, as required by this operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Row count for list xs (alias of len) |

Use `xs` as the inputs for this operation.

Effects: pure.

Row count for list xs (alias of len).

Return *[Len](len(`xs`))*.

## at

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| xs | any | required | Input list or collection, as required by this operation |
| i | number | required | Zero-based element or row index |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Zero-based index into xs at i (same as builtin at) |

Use `xs`, `i` as the inputs for this operation.

Effects: pure.

Zero-based index into xs at i (same as builtin at). Out of range → None.

Return *[At](at(value=`xs`, index=`i`))*.

## row_at

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| xs | any | required | Input list or collection, as required by this operation |
| i | number | required | Zero-based element or row index |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Zero-based row from xs at i |

Use `xs`, `i` as the inputs for this operation.

Effects: pure.

Zero-based row from xs at i.

Return *[At](at(value=`xs`, index=`i`))*.

## append

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | any | required | Input list; updates return a new list |
| item | any | required | Element to insert, replace or compare |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Append item to list |

Use `list`, `item` as the inputs for this operation.

Effects: pure.

Append item to list.

Return *[Append item to list](> host_list_append list=`list` item=`item`)*.

## prepend

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | any | required | Input list; updates return a new list |
| item | any | required | Element to insert, replace or compare |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Prepend item to list |

Use `list`, `item` as the inputs for this operation.

Effects: pure.

Prepend item to list.

Return *[Prepend item to list](> host_list_prepend list=`list` item=`item`)*.

## concat

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| a | any | required | First list; None is an empty-list seed |
| b | any | required | List appended after a |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Concatenate lists a and b |

Use `a`, `b` as the inputs for this operation.

Effects: pure.

Concatenate lists a and b.

Return *[Concatenate lists a and b](> host_list_concat a=`a` b=`b`)*.

## insert

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | any | required | Input list; updates return a new list |
| index | number | required | Zero-based position to insert, replace or remove |
| item | any | required | Element to insert, replace or compare |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Insert item into list at zero-based index (may insert at len to append) |

Use `list`, `index`, `item` as the inputs for this operation.

Effects: pure.

Insert item into list at zero-based index (may insert at len to append).

Return *[Insert item into list at zero-based index (may insert at len to append)](> host_list_insert list=`list` index=`index` item=`item`)*.

## set_at

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | any | required | Input list; updates return a new list |
| index | number | required | Zero-based position to insert, replace or remove |
| item | any | required | Element to insert, replace or compare |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Zero-based replace of list at index with item |

Use `list`, `index`, `item` as the inputs for this operation.

Effects: pure.

Zero-based replace of list at index with item. Prefer put with a 1-based index for author-facing edits.

Return *[Zero-based replace of list at index with item](> host_list_set_at list=`list` index=`index` item=`item`)*.

## remove_at

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | any | required | Input list; updates return a new list |
| index | number | required | Zero-based position to insert, replace or remove |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Remove zero-based index from list |

Use `list`, `index` as the inputs for this operation.

Effects: pure.

Remove zero-based index from list.

Return *[Remove zero-based index from list](> host_list_remove_at list=`list` index=`index`)*.

## pop

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | any | required | Input list; updates return a new list |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Drop the last item of list (errors if empty) |

Use `list` as the inputs for this operation.

Effects: pure.

Drop the last item of list (errors if empty). Use last first when you need the removed value.

We retain **`n` = [Len](len(`list`))**.
1. `n` > 0
  Return *[Drop the last item of list (errors if empty)](> host_list_remove_at list=`list` index=`n` - 1)*.
2. *
  Return *[Drop the last item of list (errors if empty)](> host_list_remove_at list=`list` index=0)*.

## first

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | any | required | Input list; updates return a new list |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | First item of list |

Use `list` as the inputs for this operation.

Effects: pure.

First item of list.

Return *[First item of list](> host_list_first list=`list`)*.

## last

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | any | required | Input list; updates return a new list |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Last item of list |

Use `list` as the inputs for this operation.

Effects: pure.

Last item of list.

Return *[Last item of list](> host_list_last list=`list`)*.

## slice

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | any | required | Input list; updates return a new list |
| start | number | required | Inclusive zero-based slice start |
| end | any | None | Exclusive zero-based slice end; None selects through the end |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Zero-based half-open slice of list from start to end |

Use `list`, `start`, `end`=None as the inputs for this operation.

Effects: pure.

Zero-based half-open slice of list from start to end.

Return *[Zero-based half-open slice of list from start to end](> host_list_slice list=`list` start=`start` end=`end`)*.

## contains

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | any | required | Input list; updates return a new list |
| item | any | required | Element to insert, replace or compare |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | bool | Whether list contains item |

Use `list`, `item` as the inputs for this operation.

Effects: pure.

Whether list contains item.

Return *[Whether list contains item](> host_list_contains list=`list` item=`item`)*.

## index_of

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | any | required | Input list; updates return a new list |
| item | any | required | Element to insert, replace or compare |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Zero-based index of item in list, or None |

Use `list`, `item` as the inputs for this operation.

Effects: pure.

Zero-based index of item in list, or None.

Return *[Zero-based index of item in list, or None](> host_list_index_of list=`list` item=`item`)*.

## reverse

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | any | required | Input list; updates return a new list |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Reverse list |

Use `list` as the inputs for this operation.

Effects: pure.

Reverse list.

Return *[Reverse list](> host_list_reverse list=`list`)*.

## sort

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | any | required | Input list; updates return a new list |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Stable ascending sort: numbers, text, bool, None, then structured values by type/display |

Use `list` as the inputs for this operation.

Effects: pure.

Stable ascending sort: numbers, text, bool, None, then structured values by type/display. Numeric cross-type comparison preserves large integer order.

Return *[Stable ascending sort: numbers, text, bool, None, then structured values by type/display](> host_list_sort list=`list`)*.

## sort_by

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | any | required | Input list; updates return a new list |
| key | text | required | Field name or key required by this operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Sort a list of maps by field key |

Use `list`, `key` as the inputs for this operation.

Effects: pure.

Sort a list of maps by field key.

Return *[Sort a list of maps by field key](> host_list_sort_by list=`list` key=`key`)*.

## unique

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | any | required | Input list; updates return a new list |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Stable unique of list (first wins) |

Use `list` as the inputs for this operation.

Effects: pure.

Stable unique of list (first wins).

Return *[Stable unique of list (first wins)](> host_list_unique list=`list`)*.

## chunk

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | any | required | Input list; updates return a new list |
| size | number | required | Positive number of elements per chunk |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Split list into chunks of size |

Use `list`, `size` as the inputs for this operation.

Effects: pure.

Split list into chunks of size.

Return *[Split list into chunks of size](> host_list_chunk list=`list` size=`size`)*.

## zip

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| a | any | required | First list of each pair |
| b | any | required | Second list of each pair |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Pair lists a and b until the shorter ends |

Use `a`, `b` as the inputs for this operation.

Effects: pure.

Pair lists a and b until the shorter ends.

Return *[Pair lists a and b until the shorter ends](> host_list_zip a=`a` b=`b`)*.

## flatten

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | any | required | Input list; updates return a new list |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | One-level flatten of list |

Use `list` as the inputs for this operation.

Effects: pure.

One-level flatten of list.

Return *[One-level flatten of list](> host_list_flatten list=`list`)*.

## clear

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | any | required | Value to transform or store, without mutating the input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Empty list or empty map for value (by type) |

Use `value` as the inputs for this operation.

Effects: pure.

Empty list or empty map for value (by type).

Return *[Empty list or empty map for value (by type)](> host_collection_clear value=`value`)*.

## get

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| map | any | required | Input string-keyed map |
| key | text | required | Field name or key required by this operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Shallow get from map at key |

Use `map`, `key` as the inputs for this operation.

Effects: pure.

Shallow get from map at key. Missing → None. Prefer access reads in author code.

Return *[Shallow get from map at key](> host_map_get map=`map` key=`key`)*.

## set

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| map | any | required | Input string-keyed map |
| key | text | required | Field name or key required by this operation |
| value | any | required | Value to transform or store, without mutating the input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Shallow map update |

Use `map`, `key`, `value` as the inputs for this operation.

Effects: pure.

Shallow map update. Prefer put for nested paths.

Return *[Shallow map update](> host_map_set map=`map` key=`key` value=`value`)*.

## delete

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| map | any | required | Input string-keyed map |
| key | text | required | Field name or key required by this operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Delete key from map |

Use `map`, `key` as the inputs for this operation.

Effects: pure.

Delete key from map.

Return *[Delete key from map](> host_map_delete map=`map` key=`key`)*.

## has

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| map | any | required | Input string-keyed map |
| key | text | required | Field name or key required by this operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | bool | Whether map has key |

Use `map`, `key` as the inputs for this operation.

Effects: pure.

Whether map has key.

Return *[Whether map has key](> host_map_has map=`map` key=`key`)*.

## keys

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| map | any | required | Input string-keyed map |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Keys of map |

Use `map` as the inputs for this operation.

Effects: pure.

Keys of map.

Return *[Keys of map](> host_map_keys map=`map`)*.

## values

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| map | any | required | Input string-keyed map |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Values of map |

Use `map` as the inputs for this operation.

Effects: pure.

Values of map.

Return *[Values of map](> host_map_values map=`map`)*.

## items

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| map | any | required | Input string-keyed map |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Key/value rows from map |

Use `map` as the inputs for this operation.

Effects: pure.

Key/value rows from map.

Return *[Key/value rows from map](> host_map_items map=`map`)*.

## merge

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| a | any | required | Base record; None seeds an empty record |
| b | any | required | Record merged over a; later values win |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Merge maps a and b (right-hand keys win) |

Use `a`, `b` as the inputs for this operation.

Effects: pure.

Merge maps a and b (right-hand keys win).

Return *[Merge maps a and b (right-hand keys win)](> host_map_merge a=`a` b=`b`)*.

## size

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| map | any | required | Input string-keyed map |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | number | Key count of map |

Use `map` as the inputs for this operation.

Effects: pure.

Key count of map.

Return *[Key count of map](> host_map_size map=`map`)*.

## empty_list

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Empty list |

Effects: pure.

Empty list.

Return *[Empty list](> host_list_concat a=None b=None)*.

## empty_map

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Return a new empty map as a seed for immutable updates; this is not an empty list |

Effects: pure.

Empty map.

Return a new empty map as a seed for immutable updates; this is not an empty list.

Return *[Return a new empty map as a seed for immutable updates; this is not an empty list](> host_map_merge a=None b=None)*.

## range

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| start | number | required | First numeric value of the sequence |
| end | number | required | Exclusive bound of the sequence |
| step | number | 1 | Nonzero range increment; sign must point toward the end |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Generate a half-open integer sequence; negative steps are allowed, zero is rejected, at most 1000000 items |

Use `start`, `end`, `step`=1 as the inputs for this operation.

Effects: pure.

Generate a half-open integer sequence; negative steps are allowed, zero is rejected, at most 1000000 items.

Return *[Generate a half-open integer sequence; negative steps are allowed, zero is rejected, at most 1000000 items](> host_table_range start=`start` end=`end` step=`step`)*.

## enumerate

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | list | required | Input list; immutable updates return a new list |
| start | number | 0 | Index value assigned to the first record; default is one |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Return index/value records without modifying the input; index starts at start |

Use `list`, `start`=0 as the inputs for this operation.

Effects: pure.

Return index/value records without modifying the input; index starts at start.

Return *[Return index/value records without modifying the input; index starts at start](> host_table_enumerate list=`list` start=`start`)*.

## pluck

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | list | required | Input list; immutable updates return a new list |
| key | text | required | Field name or storage key to read, compare or update |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Extract one field from every record; a missing field is an error instead of silently becoming None |

Use `list`, `key` as the inputs for this operation.

Effects: pure.

Extract one field from every record; a missing field is an error instead of silently becoming None.

Return *[Extract one field from every record; a missing field is an error instead of silently becoming None](> host_table_pluck list=`list` key=`key`)*.

## project

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | list | required | Input list; immutable updates return a new list |
| keys | list | required | Ordered field-name list |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Keep the requested fields, in keys order; every requested field must exist |

Use `list`, `keys` as the inputs for this operation.

Effects: pure.

Keep the requested fields, in keys order; every requested field must exist.

Return *[Keep the requested fields, in keys order; every requested field must exist](> host_table_project list=`list` keys=`keys`)*.

## where

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | list | required | Input list; immutable updates return a new list |
| key | text | required | Field name or storage key to read, compare or update |
| value | any | required | Value to transform or store, without mutating the input |
| op | text | "eq" | Operation name understood by this helper |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Keep matching records |

Use `list`, `key`, `value`, `op`="eq" as the inputs for this operation.

Effects: pure.

Keep matching records. Operators: eq/ne/gt/ge/lt/le/contains/in; these are fixed operations, never code to evaluate.

Return *[Keep matching records](> host_table_where list=`list` key=`key` value=`value` op=`op`)*.

## group_by

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | list | required | Input list; immutable updates return a new list |
| key | text | required | Field name or storage key to read, compare or update |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Return first-seen ordered key/count/rows records; group keys keep their original value types |

Use `list`, `key` as the inputs for this operation.

Effects: pure.

Call this function directly:

```sh
marqdo call lib/table.mq.md group_by 'list=[{"team":"a","n":3},{"team":"b","n":1},{"team":"a","n":2}]' 'key="team"' --effects none --json
```

Return first-seen ordered key/count/rows records; group keys keep their original value types.

Return *[Return first-seen ordered key/count/rows records; group keys keep their original value types](> host_table_group_by list=`list` key=`key`)*.

## unique_by

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | list | required | Input list; immutable updates return a new list |
| key | text | required | Field name or storage key to read, compare or update |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Keep the first record per field value; input order is preserved |

Use `list`, `key` as the inputs for this operation.

Effects: pure.

Keep the first record per field value; input order is preserved.

Return *[Keep the first record per field value; input order is preserved](> host_table_unique_by list=`list` key=`key`)*.

## aggregate

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | list | required | Input list; immutable updates return a new list |
| key | any | None | Field name or storage key to read, compare or update |
| op | text | "sum" | Operation name understood by this helper |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | number | Aggregate scalar values or one record field: sum/count/min/max/mean |

Use `list`, `key`=None, `op`="sum" as the inputs for this operation.

Effects: pure.

Aggregate scalar values or one record field: sum/count/min/max/mean. Empty sum/count is zero; empty min/max/mean fails; integer sum checks overflow.

Return *[Aggregate scalar values or one record field: sum/count/min/max/mean](> host_table_aggregate list=`list` key=`key` op=`op`)*.

## sort_keys

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | list | required | Input list; immutable updates return a new list |
| keys | list | required | Ordered field-name list |
| descending | bool | False | Whether the multi-key sort is descending |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Stable lexicographic sort by fields in keys order; descending reverses the comparison, keeping equal-key order |

Use `list`, `keys`, `descending`=False as the inputs for this operation.

Effects: pure.

Stable lexicographic sort by fields in keys order; descending reverses the comparison, keeping equal-key order.

Return *[Stable lexicographic sort by fields in keys order; descending reverses the comparison, keeping equal-key order](> host_table_sort_keys list=`list` keys=`keys` descending=`descending`)*.

## rename

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | list | required | Input list; immutable updates return a new list |
| fields | map | required | Structured field/value record attached to the log event |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Rename fields using an old-name/new-name map; duplicate output names fail |

Use `list`, `fields` as the inputs for this operation.

Effects: pure.

Rename fields using an old-name/new-name map; duplicate output names fail.

Return *[Rename fields using an old-name/new-name map; duplicate output names fail](> host_table_rename list=`list` fields=`fields`)*.

## from_columns

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| columns | map | required | Column-name/list record; all columns must have equal length |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Convert a column map to row records |

Use `columns` as the inputs for this operation.

Effects: pure.

Convert a column map to row records. Every column must be a list of equal length; scalar object fields are not inferred as columns.

Return *[Convert a column map to row records](> host_table_from_columns columns=`columns`)*.

## to_columns

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| rows | list | required | List of row records, rather than an implicit column map |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | map | Convert records with identical field sets into a column map; empty input yields an empty map |

Use `rows` as the inputs for this operation.

Effects: pure.

Convert records with identical field sets into a column map; empty input yields an empty map.

Return *[Convert records with identical field sets into a column map; empty input yields an empty map](> host_table_to_columns rows=`rows`)*.

## join

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| a | list | required | First operand; its domain is described by this operation |
| b | list | required | Second operand; its domain is described by this operation |
| key | text | required | Field name or storage key to read, compare or update |
| right_key | text | required | Matching field in right-hand records |
| mode | text | "inner" | Join mode: inner or left |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Join records by field equality, preserving left and matching-right order |

Use `a`, `b`, `key`, `right_key`, `mode`="inner" as the inputs for this operation.

Effects: pure.

Join records by field equality, preserving left and matching-right order. mode is inner/left; colliding right fields use right_ prefix; unresolved name collisions fail.

Return *[Join records by field equality, preserving left and matching-right order](> host_table_join a=`a` b=`b` key=`key` right_key=`right_key` mode=`mode`)*.

## cast

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| rows | list | required | List of row records, rather than an implicit column map |
| schema | map | required | Field/type record using the standard contract type vocabulary |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Convert schema fields to text/int/number/bool/any; missing or invalid cells report row and field |

Use `rows`, `schema` as the inputs for this operation.

Effects: pure.

Convert schema fields to text/int/number/bool/any; missing or invalid cells report row and field. Other fields survive.

Return *[Convert schema fields to text/int/number/bool/any; missing or invalid cells report row and field](> host_table_cast rows=`rows` schema=`schema`)*.

## union

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| a | list | required | First operand; its domain is described by this operation |
| b | list | required | Second operand; its domain is described by this operation |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Stable set union; first occurrences win, using the same type-sensitive equality as unique |

Use `a`, `b` as the inputs for this operation.

Effects: pure.

Stable set union; first occurrences win, using the same type-sensitive equality as unique.

Return *[Stable set union; first occurrences win, using the same type-sensitive equality as unique](> host_table_union a=`a` b=`b`)*.

## intersection

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| a | list | required | First operand; its domain is described by this operation |
| b | list | required | Second operand; its domain is described by this operation |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Stable set intersection; duplicates are removed |

Use `a`, `b` as the inputs for this operation.

Effects: pure.

Stable set intersection; duplicates are removed.

Return *[Stable set intersection; duplicates are removed](> host_table_intersection a=`a` b=`b`)*.

## difference

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| a | list | required | First operand; its domain is described by this operation |
| b | list | required | Second operand; its domain is described by this operation |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Stable values in a but not b; duplicates are removed |

Use `a`, `b` as the inputs for this operation.

Effects: pure.

Stable values in a but not b; duplicates are removed.

Return *[Stable values in a but not b; duplicates are removed](> host_table_difference a=`a` b=`b`)*.

## try_cast

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| rows | list | required | List of row records, rather than an implicit column map |
| schema | map | required | Field/type record using the standard contract type vocabulary |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | map | Recoverable cast; invalid fields return Result instead of aborting the caller |

Use `rows`, `schema` as the inputs for this operation.

Effects: pure.

Recoverable cast; invalid fields return Result instead of aborting the caller.

Return *[Recoverable cast; invalid fields return Result instead of aborting the caller](> host_table_try_cast rows=`rows` schema=`schema`)*.

## validate

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| rows | list | required | List of row records, rather than an implicit column map |
| schema | map | required | Field/type record using the standard contract type vocabulary |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | map | Validate required fields using core gradual types, returning Result with error |

Use `rows`, `schema` as the inputs for this operation.

Effects: pure.

Validate required fields using core gradual types, returning Result with error.details row/field diagnostics. int is the core number alias; use cast for strict integer conversion.

Return *[Validate required fields using core gradual types, returning Result with error.details row/field diagnostics](> host_table_validate rows=`rows` schema=`schema`)*.

## find

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | list | required | Ordered row records to test without changing their order |
| key | text | required | Field name whose value is compared in each row |
| value | any | required | Comparison operand; in tests membership in this supplied collection |
| op | text | "eq" | Comparison: eq, ne, gt, ge, lt, le, contains or in; invalid operators fail |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | First matching row, or None when no row matches |

Use `list`, `key`, `value`, `op`="eq" as the inputs for this operation.

Effects: pure.

Use the same comparison as where. Missing fields and invalid operators still produce a diagnostic; the input rows remain unchanged.

We retain **`selected` = [Where](where(list=`list`, key=`key`, value=`value`, op=`op`))**.
Return *[First](first(list=`selected`))*.

## any

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | list | required | Ordered row records to test without changing their order |
| key | text | required | Field name whose value is compared in each row |
| value | any | required | Comparison operand; in tests membership in this supplied collection |
| op | text | "eq" | Comparison: eq, ne, gt, ge, lt, le, contains or in; invalid operators fail |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | bool | Whether at least one row matches; false on an empty list |

Use `list`, `key`, `value`, `op`="eq" as the inputs for this operation.

Effects: pure.

Use the same comparison as where. Missing fields and invalid operators still produce a diagnostic; the input rows remain unchanged.

We retain **`selected` = [Where](where(list=`list`, key=`key`, value=`value`, op=`op`))**.
We retain **`matched` = [Len](len(`selected`))**.
Return *matched > 0*.

## all

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | list | required | Ordered row records to test without changing their order |
| key | text | required | Field name whose value is compared in each row |
| value | any | required | Comparison operand; in tests membership in this supplied collection |
| op | text | "eq" | Comparison: eq, ne, gt, ge, lt, le, contains or in; invalid operators fail |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | bool | Whether every row matches; true on an empty list |

Use `list`, `key`, `value`, `op`="eq" as the inputs for this operation.

Effects: pure.

Use the same comparison as where. Missing fields and invalid operators still produce a diagnostic; the input rows remain unchanged.

We retain **`selected` = [Where](where(list=`list`, key=`key`, value=`value`, op=`op`))**.
We retain **`matched` = [Len](len(`selected`))**.
We retain **`total` = [Len](len(`list`))**.
Return *matched == total*.

## summarize

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| list | list | required | Input list; immutable updates return a new list |
| key | text | required | Field name or storage key to read, compare or update |
| field | text | required | Numeric field to aggregate within each group |
| op | text | "sum" | Operation name understood by this helper |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Group by key and aggregate one field in a batch, returning first-seen key/count/value records |

Use `list`, `key`, `field`, `op`="sum" as the inputs for this operation.

Effects: pure.

Group by key and aggregate one field in a batch, returning first-seen key/count/value records. Uses aggregate semantics and avoids repeatedly copying a growing output list in author loops.

Return *[Group by key and aggregate one field in a batch, returning first-seen key/count/value records](> host_table_summarize list=`list` key=`key` field=`field` op=`op`)*.

## from_pairs

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| pairs | list | required | Ordered row records with text key and arbitrary value fields; extra fields are ignored |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | New record in first-key insertion order; later entries replace an existing key's value |

Effects: pure.

The input `pairs` makes a long named record readable as an @ table with key/value columns.
Use this when a horizontal record would become too wide to inspect, such as prompt sections or
configuration fragments. It uses the same ordinary row records and immutable map update as the
rest of this library; no JSON text or alternate schema language is parsed. An empty list returns
an empty record. Invalid keys are rejected by put, rather than silently creating a dummy column.

We begin with **record = [Create a new record](empty_map())**. Each row contributes one explicit
field; repeated keys update that field while keeping the position of its first occurrence.

- [Copy each named field into the new record](遍历 pairs 为 pair)
  Read **key = [Read this row's field name](access(pair, "key"))**
  and **value = [Read this row's field value](access(pair, "value"))**.
  We retain **record = [Set this field without changing the input rows](put(in=`record`, at=`key`, value=`value`))**.

After processing the ordered rows, return *record*. The original pairs remain available to the
caller; this function performs no filesystem, network or plugin operation.
