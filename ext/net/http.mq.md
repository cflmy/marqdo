---
markup: occlusion
title: ext/net/http
description: HTTP primitives surface for Web Runtime (ADR 0007). Prefer Document/Endpoint Artifacts over raw calls.
import:
  plugin: lib/plugin.mq.md
  sys: lib/sys.mq.md
---

This file constructs the application Result record only; it does not implement an HTTP transport. For real HTTP requests, response limits and downloads, import lib/net.mq.md. The [network entry](index.mq.md) distinguishes pure URL transformations from native WebSocket exchanges. Keep these responsibilities explicit when extending the package so a readable tool name never implies a missing capability.

## _load

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | No business value on completion; a missing native plugin reports the cause and exits this run. |

Resolve and load the native web plugin before resource operations; missing binaries stop this run.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="web"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/net: native web plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.
Return *None*.

## result

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| ok | any | required | Whether this application-level operation succeeded |
| value | any | None | Value supplied to this operation; immutable transformations return a new value |
| error | any | None | Structured failure information supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Unified Result map `{ok, value, error}`. |

Use `ok`, `value`=None, `error`=None as the inputs for this operation.

Unified Result map `{ok, value, error}`.

`out` =

| ok | value | error |
|----|-------|-------|
| `ok` | `value` | `error` |

Return *out*.
