---
markup: occlusion
title: ext/web/route
description: Declared Endpoint / Document route registration (ADR 0007).
import:
  cap: ext/web/_capability/index.mq.md
  table: lib/table.mq.md
---

Routes declare the HTTP surface. Prefer Document/Endpoint metadata and serve
for whole sites; use route handles only for explicit app assembly. To obtain an
app use web.app on [the facade](../../web/web.mq.md); to discover a site's routes
without a live app use web.inspect with root. A fresh call without app/root
cannot inspect the private state of another running server.

```sh
marqdo call ext/web/route/index.mq.md new 'app={}' 'path="/hello"' 'page={"title":"Hello","body":"Hi"}' --effects plugin --session handbook --toget route --json
marqdo call ext/web/route/index.mq.md register 'route=$route' --effects plugin --session handbook --json
```

The returned registration is a value, not a listening server. Middleware and
Endpoint handlers remain explicit; schema/auth resources live in their own
packages. Update both route facades and HTTP artifact tests for registration
changes; never introduce undeclared user URLs as a hidden convenience.

## new

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| app | any | required | app handle |
| path | text | required | URL path |
| method | text | "GET" | HTTP method |
| page | any | None | Document or None |
| fn | text | "" | endpoint member |
| request | text | "json" | request kind |
| response | text | "json" | response kind |
| auth | text | "" | auth tag |
| class | text | "" | route class |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | route handle |

Use `app`, `path`, `method`="GET", `page`=None, `fn`="", `request`="json", `response`="json", `auth`="", `class`="" as the inputs for this operation.

Build a declared route handle for `app` at `path` with `method`="GET" .

`page`=None holds a Document; `fn`="" names an Endpoint lib.member.

`request`="json", `response`="json", `auth`="", and `class`="" document the
contract. Optional class values: execute, short, read, control (empty →
heuristic / legacy_serial). This tool does not register yet — call `register`
or use the facade `web.route` which registers in one step.

We **[Load the native plugin required by this operation](cap.load())**.

`out` =

| _type | app | path | method | page | fn | request | response | auth | class |
|-------|-----|------|--------|------|----|---------|----------|------|-------|
| route | `app` | `path` | `method` | `page` | `fn` | `request` | `response` | `auth` | `class` |

Return *out*.

## register

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| route | any | required | route handle |
| app | any | None | app or None |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | registration result |

Use `route`, `app`=None as the inputs for this operation.

Register declared route handle `route` on an app. Optional `app`=None falls back
to the app stored on the route object so stepwise agent sessions can omit it.
Page routes call web_app_route; Endpoint `fn` routes call web_app_invoke with
method and body/return contract fields pulled from the handle. User routes must
already be declared on this handle — this tool never invents paths. Prefer this
free tool under `--effects plugin` rather than a method chain when driving
registration from `marqdo call`. Soft-fails with ok=False when neither page nor
fn is present, so callers can detect incomplete declarations early.

We **[Load the native plugin required by this operation](cap.load())**.
We retain **a = app**.
1. `a` == None
  We retain **`a` = [Read app field](access(`route`, "app"))**.
2. *
  **_ = 1**
We retain **`page` = [Read page field](access(`route`, "page"))**.
We retain **`fn` = [Read fn field](access(`route`, "fn"))**.
1. `page`
  We retain **`path` = [Read path field](access(`route`, "path"))**.
  Return *[Register declared route handle route on an app](web_app_route(app=`a`, path=`path`, page=`page`))*.
2. `fn`
  We retain **`path` = [Read path field](access(`route`, "path"))**.
  We retain **`method` = [Read method field](access(`route`, "method"))**.
  We retain **`request` = [Read request field](access(`route`, "request"))**.
  We retain **`response` = [Read response field](access(`route`, "response"))**.
  We retain **`class` = [Read class field](access(`route`, "class"))**.
  Return *[Register declared route handle route on an app](web_app_invoke(app=`a`, path=`path`, method=`method`, fn=`fn`, body=`request`, return=`response`, class=`class`))*.
3. *
  Return *[Set the ok field in the new record](table.put(in=None, at="ok", value=False))*.

## use

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| middleware | any | required | middleware table |
| route | any | required | route handle |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | use result |

Use `middleware`, `route` as the inputs for this operation.

Attach middleware table `middleware` to declared route handle `route`
(`|header|value|` or named capability rows). Prefer first-class middleware tables
over configure junk drawers. Call this free tool with `--effects plugin`.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Attach middleware table middleware to declared route handle route (|header|value| or named capability rows)](web_route_use(route=`route`, middleware=`middleware`))*.

# route

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| app | any | required | Application handle created by the public web app constructor |
| path | any | required | URL path declared for this route |
| method | any | "GET" | HTTP request method for the declared route |
| page | any | None | Initial Document/page resource, or None for explicit later route mounts |
| fn | any | "" | Endpoint member path; empty when mounting a Document page |
| request | any | "json" | Request body format passed to the native Endpoint dispatcher |
| response | any | "json" | Response format passed to the native Endpoint dispatcher |
| auth | any | "" | Explicit route authentication policy label |
| class | any | "" | Execution class: execute, short, read or control; empty keeps native heuristics |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Declared route handle retaining app, path, method, page or endpoint and dispatch fields |

Use `app`, `path`, `method`="GET", `page`=None, `fn`="", `request`="json", `response`="json", `auth`="", `class`="" as the inputs for this operation.

Declared route handle for author method chains. Prefer `web.route` / Artifact
`type: endpoint` discovery via `web.serve`. Optional `class`: `execute` |
`short` | `read` | `control` (empty → heuristic / `legacy_serial`).

We **[Load the native plugin required by this operation](cap.load())**.

`out` =

| _type | app | path | method | page | fn | request | response | auth | class |
|-------|-----|------|--------|------|----|---------|----------|------|-------|
| route | `app` | `path` | `method` | `page` | `fn` | `request` | `response` | `auth` | `class` |

Return *out*.

## use

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| middleware | any | required | middleware table |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | use result |

Use `middleware` as the inputs for this operation.

Attach middleware table to this route (`|header|value|` or named capability
rows). Prefer explicit tables authors can read; do not restore a configure bag
for headers and cors.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Attach middleware table to this route (|header|value| or named capability rows)](web_route_use(route=`self`, middleware=`middleware`))*.

## register

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| app | any | None | app or None |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | registration result |

Use `app`=None as the inputs for this operation.

Register this declared route on an app handle. When optional `app` is None, use
the app stored on this route object so author chains can omit the argument after
construction. Document pages call web_app_route with the stored path and page
handle. Endpoint members call web_app_invoke with path, method, and the
request/response/class contract fields from this route. Prefer Artifact
discovery via web.serve for whole sites; use this method when assembling routes
in memory for tests or advanced mounts. Returns a soft failure map when neither
page nor fn is set, so callers can detect an incomplete declaration without a
hard crash. Never invents an undeclared path — registration only publishes what
this handle already carries.

We **[Load the native plugin required by this operation](cap.load())**.
We retain **a = app**.
1. `a` == None
  We retain **`a` = [Read app field](access(`self`, "app"))**.
2. *
  **_ = 1**
We retain **`page` = [Read page field](access(`self`, "page"))**.
We retain **`fn` = [Read fn field](access(`self`, "fn"))**.
1. `page`
  We retain **`path` = [Read path field](access(`self`, "path"))**.
  Return *[Register this declared route on an app handle](web_app_route(app=`a`, path=`path`, page=`page`))*.
2. `fn`
  We retain **`path` = [Read path field](access(`self`, "path"))**.
  We retain **`method` = [Read method field](access(`self`, "method"))**.
  We retain **`request` = [Read request field](access(`self`, "request"))**.
  We retain **`response` = [Read response field](access(`self`, "response"))**.
  We retain **`class` = [Read class field](access(`self`, "class"))**.
  Return *[Register this declared route on an app handle](web_app_invoke(app=`a`, path=`path`, method=`method`, fn=`fn`, body=`request`, return=`response`, class=`class`))*.
3. *
  Return *[Set the ok field in the new record](table.put(in=None, at="ok", value=False))*.
