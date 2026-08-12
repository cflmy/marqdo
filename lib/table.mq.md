---
title: lib/table
description: >-
  List and map helpers. Prefer put (1-based path, same as footnotes); at/set_at/slice
  are 0-based. json is for parse/stringify.
---

## put
    + `in`
    + `at`
    + `value`

Directly update one element. `at=` is a text key, a **1-based** list index, or a list path of keys/indices (same nesting as footnotes).

**> host_collection_put in=`in` at=`at` value=`value`**

## len
    + `xs`

**> len `xs`**

## rows
    + `xs`

**> len `xs`**

## at
    + `xs`
    + `i`

Index is zero-based (same as builtin `at`). Out of range → `None`.

**> at value=`xs` index=`i`**

## row_at
    + `xs`
    + `i`

**> at value=`xs` index=`i`**

## append
    + `list`
    + `item`

**> host_list_append list=`list` item=`item`**

## prepend
    + `list`
    + `item`

**> host_list_prepend list=`list` item=`item`**

## concat
    + `a`
    + `b`

**> host_list_concat a=`a` b=`b`**

## insert
    + `list`
    + `index`
    + `item`

Index is zero-based (may insert at `len` to append).

**> host_list_insert list=`list` index=`index` item=`item`**

## set_at
    + `list`
    + `index`
    + `item`

Zero-based replace. Prefer `put` with 1-based index for author-facing edits.

**> host_list_set_at list=`list` index=`index` item=`item`**

## remove_at
    + `list`
    + `index`

**> host_list_remove_at list=`list` index=`index`**

## pop
    + `list`

Drop the last item (errors if empty). Use `last` first when you need the removed value.

*`n` = > len `list`*
1. `n` > 0
  **> host_list_remove_at list=`list` index=`n` - 1**
2. *
  **> host_list_remove_at list=`list` index=0**

## first
    + `list`

**> host_list_first list=`list`**

## last
    + `list`

**> host_list_last list=`list`**

## slice
    + `list`
    + `start`
    + `end`=None

Zero-based half-open range from `start` to `end`.

**> host_list_slice list=`list` start=`start` end=`end`**

## contains
    + `list`
    + `item`

**> host_list_contains list=`list` item=`item`**

## index_of
    + `list`
    + `item`

Zero-based index or `None`.

**> host_list_index_of list=`list` item=`item`**

## reverse
    + `list`

**> host_list_reverse list=`list`**

## clear
    + `value`

Empty list or empty map (by type).

**> host_collection_clear value=`value`**

## get
    + `map`
    + `key`

Missing key → `None`.

**> host_map_get map=`map` key=`key`**

## set
    + `map`
    + `key`
    + `value`

Shallow map update. Prefer `put` for nested paths.

**> host_map_set map=`map` key=`key` value=`value`**

## delete
    + `map`
    + `key`

**> host_map_delete map=`map` key=`key`**

## has
    + `map`
    + `key`

**> host_map_has map=`map` key=`key`**

## keys
    + `map`

**> host_map_keys map=`map`**

## values
    + `map`

**> host_map_values map=`map`**

## items
    + `map`

Rows shaped as key/value maps.

**> host_map_items map=`map`**

## merge
    + `a`
    + `b`

Right-hand keys win.

**> host_map_merge a=`a` b=`b`**

## size
    + `map`

**> host_map_size map=`map`**

## empty_list

**> host_list_concat a=None b=None**

## empty_map

**> host_map_merge a=None b=None**
