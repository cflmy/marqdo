---
markup: occlusion
title: ext/net/http
description: HTTP primitives surface for Web Runtime (ADR 0007). Prefer Document/Endpoint Artifacts over raw calls.
import:
  plugin: lib/plugin.mq.md
  sys: lib/sys.mq.md
---

This file constructs application Result records and binds explicit server redirects; it does not implement an HTTP client. For real HTTP requests, response limits and downloads, import lib/net.mq.md. The [network entry](index.mq.md) distinguishes pure URL transformations from native WebSocket exchanges. Keep these responsibilities explicit when extending the package so a readable tool name never implies a missing capability.

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

## redirect

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| app | any | required | Prepared web application |
| from | text | required | Explicit source route |
| to | text | required | Redirect destination |
| permanent | bool | False | False uses a temporary redirect; True uses the native permanent status |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Updated app with the declared server redirect |

Use `app`, `from`, `to` and `permanent`=False before web.listen. This is a server route policy owned by the network resource, not a client-side history operation. Keep the returned app because binding produces a new value. Redirecting a legacy entry path can coexist with Document and Endpoint discovery without rebuilding the application through removed configure methods. The native runtime validates declared source paths. No listener starts here, and constructing a redirect does not verify that its destination page exists.

We **[Load the existing native HTTP runtime](_load())**.
Return *[Bind this declared HTTP redirect](web_app_redirect(app=`app`, from=`from`, to=`to`, permanent=`permanent`))*.
