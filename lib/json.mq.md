---
title: lib/json
description: JSON parse/stringify (shared EN/ZH import). Not a dict builder — use lib/table.
---

Parse and stringify JSON. For maps/lists prefer GFM tables and lib/table put; do not chain json.set / json.append in new code.

## parse
    + `text`

Parse JSON text into a Marqdo value.

Caller: [json.parse] text="{}"

*> host_json_parse text=`text`*

## stringify
    + `value`
    + `indent`=None

Serialize value to JSON text; optional indent.

*> host_json_stringify value=`value` indent=`indent`*

## get
    + `value`
    + `key`

Shallow get from JSON object value at key. Prefer link-index get or table.get in new code.

*> host_json_get value=`value` key=`key`*

## keys
    + `value`

Keys of JSON object value.

*> host_json_keys value=`value`*

## quote
    + `text`

JSON-quote string text (for building request bodies).

*> host_json_quote text=`text`*

## set
    + `map`
    + `key`
    + `value`

Compatibility shallow set. Prefer table.put / table.set.

*> host_map_set map=`map` key=`key` value=`value`*

## append
    + `list`
    + `item`

Compatibility append. Prefer table.append.

*> host_list_append list=`list` item=`item`*
