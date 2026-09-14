---
title: lib/table
description: >-
  List and map helpers. Prefer put (1-based path, same nesting as link-index get);
  at/set_at/slice are 0-based. Use lib/json only for parse/stringify.
---

List and map helpers. Prefer put for nested writes; authors should read with link-index get, not new footnote indexes.

## put
    + `in`
    + `at`
    + `value`

Update one element of collection in at path at to value.

at is a text key, a 1-based list index, or a list of keys/indices (same nesting as link-index get).

Caller: [table.put] in=None at="Authorization" value="Bearer-x"

*> host_collection_put in=`in` at=`at` value=`value`*

## len
    + `xs`

Length of list or map xs.

*> len `xs`*

## rows
    + `xs`

Row count for list xs (alias of len).

*> len `xs`*

## at
    + `xs`
    + `i`

Zero-based index into xs at i (same as builtin at). Out of range → None.

*> at value=`xs` index=`i`*

## row_at
    + `xs`
    + `i`

Zero-based row from xs at i.

*> at value=`xs` index=`i`*

## append
    + `list`
    + `item`

Append item to list.

*> host_list_append list=`list` item=`item`*

## prepend
    + `list`
    + `item`

Prepend item to list.

*> host_list_prepend list=`list` item=`item`*

## concat
    + `a`
    + `b`

Concatenate lists a and b.

*> host_list_concat a=`a` b=`b`*

## insert
    + `list`
    + `index`
    + `item`

Insert item into list at zero-based index (may insert at len to append).

*> host_list_insert list=`list` index=`index` item=`item`*

## set_at
    + `list`
    + `index`
    + `item`

Zero-based replace of list at index with item. Prefer put with a 1-based index for author-facing edits.

*> host_list_set_at list=`list` index=`index` item=`item`*

## remove_at
    + `list`
    + `index`

Remove zero-based index from list.

*> host_list_remove_at list=`list` index=`index`*

## pop
    + `list`

Drop the last item of list (errors if empty). Use last first when you need the removed value.

**`n` = > len `list`**
1. `n` > 0
  *> host_list_remove_at list=`list` index=`n` - 1*
2. *
  *> host_list_remove_at list=`list` index=0*

## first
    + `list`

First item of list.

*> host_list_first list=`list`*

## last
    + `list`

Last item of list.

*> host_list_last list=`list`*

## slice
    + `list`
    + `start`
    + `end`=None

Zero-based half-open slice of list from start to end.

*> host_list_slice list=`list` start=`start` end=`end`*

## contains
    + `list`
    + `item`

Whether list contains item.

*> host_list_contains list=`list` item=`item`*

## index_of
    + `list`
    + `item`

Zero-based index of item in list, or None.

*> host_list_index_of list=`list` item=`item`*

## reverse
    + `list`

Reverse list.

*> host_list_reverse list=`list`*

## sort
    + `list`

Ascending sort of list (int/num/text preferred).

*> host_list_sort list=`list`*

## sort_by
    + `list`
    + `key`

Sort a list of maps by field key.

*> host_list_sort_by list=`list` key=`key`*

## unique
    + `list`

Stable unique of list (first wins).

*> host_list_unique list=`list`*

## chunk
    + `list`
    + `size`

Split list into chunks of size.

*> host_list_chunk list=`list` size=`size`*

## zip
    + `a`
    + `b`

Pair lists a and b until the shorter ends.

*> host_list_zip a=`a` b=`b`*

## flatten
    + `list`

One-level flatten of list.

*> host_list_flatten list=`list`*

## clear
    + `value`

Empty list or empty map for value (by type).

*> host_collection_clear value=`value`*

## get
    + `map`
    + `key`

Shallow get from map at key. Missing → None. Prefer link-index get in author code.

*> host_map_get map=`map` key=`key`*

## set
    + `map`
    + `key`
    + `value`

Shallow map update. Prefer put for nested paths.

*> host_map_set map=`map` key=`key` value=`value`*

## delete
    + `map`
    + `key`

Delete key from map.

*> host_map_delete map=`map` key=`key`*

## has
    + `map`
    + `key`

Whether map has key.

*> host_map_has map=`map` key=`key`*

## keys
    + `map`

Keys of map.

*> host_map_keys map=`map`*

## values
    + `map`

Values of map.

*> host_map_values map=`map`*

## items
    + `map`

Key/value rows from map.

*> host_map_items map=`map`*

## merge
    + `a`
    + `b`

Merge maps a and b (right-hand keys win).

*> host_map_merge a=`a` b=`b`*

## size
    + `map`

Key count of map.

*> host_map_size map=`map`*

## empty_list

Empty list.

*> host_list_concat a=None b=None*

## empty_map

Empty map.

*> host_map_merge a=None b=None*
