---
title: ext/net/http
description: HTTP primitives surface for Web Runtime (ADR 0007). Prefer Document/Endpoint Artifacts over raw calls.
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
---

## _load

**p = > plugin.native_path name="web"**
1. `p`
  > plugin.load path=`p`
2. *
  > print text=ext/net: native web plugin not found
  > sys.exit code=1
****

## result
    + `ok`
    + `value`=None
    + `error`=None

Unified Result map `{ok, value, error}`.

`out` =

| ok | value | error |
|----|-------|-------|
| `ok` | `value` | `error` |

*out*
