---
title: lib/browser
description: >-
  Readable browser effect helpers for WASM clients. Prefer GFM tables + these
  helpers; do not build effects with json.set chains.
import table:lib/table.mq.md
---

## set_text
    + `sel`
    + `text`

Build set_text effect map for the official bridge.

*`m` = > table.put in=None at=sel value=text*
**> table.put in=None at="set_text" value=m**

## set_html
    + `sel`
    + `html`

Build set_html effect map.

*`m` = > table.put in=None at=sel value=html*
**> table.put in=None at="set_html" value=m**

## set_value
    + `sel`
    + `value`

Build set_value effect map.

*`m` = > table.put in=None at=sel value=value*
**> table.put in=None at="set_value" value=m**

## set_class
    + `sel`
    + `class`

Build set_class effect map.

*`m` = > table.put in=None at=sel value=class*
**> table.put in=None at="set_class" value=m**

## set_attr
    + `sel`
    + `attrs`

Build set_attr effect map; attrs is usually a small map of attribute names to values.

*`m` = > table.put in=None at=sel value=attrs*
**> table.put in=None at="set_attr" value=m**

## wrap
    + `key`
    + `value`

Single-key effect map wrapper.

**> table.put in=None at=key value=value**

## merge
    + `a`
    + `b`

Merge two effect maps; right-hand keys win.

**> table.merge a=`a` b=`b`**

## canvas
    + `sel`
    + `commands`
    + `clear`=True

Build canvas effect; commands is usually an @ GFM table.

`spec` =

| sel | clear | commands |
|-----|-------|----------|
| `sel` | `clear` | `commands` |

**> table.put in=None at="canvas" value=spec**

## audio
    + `op`="play"
    + `id`="default"
    + `src`=""
    + `freq`=None
    + `ms`=None
    + `sel`=""

Build audio effect for the bridge.

*`spec` = > table.put in=None at="op" value=op*
*`spec` = > table.put in=`spec` at="id" value=id*
1. `src` != ""
    *`spec` = > table.put in=`spec` at="src" value=src*
1. `freq` != None
    *`spec` = > table.put in=`spec` at="freq" value=freq*
1. `ms` != None
    *`spec` = > table.put in=`spec` at="ms" value=ms*
1. `sel` != ""
    *`spec` = > table.put in=`spec` at="sel" value=sel*
**> table.put in=None at="audio" value=spec**

## read_file
    + `sel`
    + `then`
    + `as`="text"

Build read_file effect.

`spec` =

| sel | as | then |
|-----|----|------|
| `sel` | `as` | `then` |

**> table.put in=None at="read_file" value=spec**

## observe
    + `specs`

Build observe effect; specs is one map or an @ list of maps.

**> table.put in=None at="observe" value=specs**

## unobserve
    + `specs`

Build unobserve effect; specs is a map or @ list with id.

**> table.put in=None at="unobserve" value=specs**

## focus
    + `sel`

**> table.put in=None at="focus" value=sel**

## navigate
    + `url`
    + `replace`=False

`spec` =

| url | replace |
|-----|---------|
| `url` | `replace` |

**> table.put in=None at="navigate" value=spec**

## storage
    + `spec`

**> table.put in=None at="storage" value=spec**

## interval
    + `ms`
    + `then`
    + `id`="default"

`spec` =

| ms | then | id |
|----|------|----|
| `ms` | `then` | `id` |

**> table.put in=None at="interval" value=spec**

## clear_interval
    + `id`="default"

*`spec` = > table.put in=None at="id" value=id*
**> table.put in=None at="clear_interval" value=spec**

## fetch
    + `url`
    + `then`
    + `method`="GET"

`spec` =

| url | then | method |
|-----|------|--------|
| `url` | `then` | `method` |

**> table.put in=None at="fetch" value=spec**

## fetch_all
    + `requests`
    + `then`

`spec` =

| requests | then |
|----------|------|
| `requests` | `then` |

**> table.put in=None at="fetch_all" value=spec**

## after
    + `ms`
    + `then`

`spec` =

| ms | then |
|----|------|
| `ms` | `then` |

**> table.put in=None at="after" value=spec**

## ws
    + `spec`

**> table.put in=None at="ws" value=spec**

## clipboard
    + `text`

*`spec` = > table.put in=None at="text" value=text*
**> table.put in=None at="clipboard" value=spec**

## download
    + `body`
    + `filename`="download.txt"
    + `mime`="text/plain;charset=utf-8"

`spec` =

| body | filename | mime |
|------|----------|------|
| `body` | `filename` | `mime` |

**> table.put in=None at="download" value=spec**
