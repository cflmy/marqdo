---
title: lib/json
description: JSON parse/stringify (shared EN/ZH import)
---

## parse
    + `text`

**> host_json_parse text=`text`**

## stringify
    + `value`
    + `indent`=None

**> host_json_stringify value=`value` indent=`indent`**

## get
    + `value`
    + `key`

**> host_json_get value=`value` key=`key`**

## keys
    + `value`

**> host_json_keys value=`value`**

## quote
    + `text`

**> host_json_quote text=`text`**

## set
    + `map`
    + `key`
    + `value`

**> host_map_set map=`map` key=`key` value=`value`**

## append
    + `list`
    + `item`

**> host_list_append list=`list` item=`item`**
