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

| 参数 | 类型 | 说明 |
|------|------|------|
| app | any | app handle |
| path | text | URL path |
| method | text | HTTP method |
| page | any | Document or None |
| fn | text | endpoint member |
| request | text | request kind |
| response | text | response kind |
| auth | text | auth tag |
| class | text | route class |

Build a declared route handle for `app` at `path` with `method`="GET" .
`page`=None holds a Document; `fn`="" names an Endpoint lib.member.
`request`="json", `response`="json", `auth`="", and `class`="" document the
contract. Optional class values: execute, short, read, control (empty →
heuristic / legacy_serial). This tool does not register yet — call `register`
or use the facade `web.route` which registers in one step.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | route handle |

> cap.load
`out` =

| _type | app | path | method | page | fn | request | response | auth | class |
|-------|-----|------|--------|------|----|---------|----------|------|-------|
| route | `app` | `path` | `method` | `page` | `fn` | `request` | `response` | `auth` | `class` |

*out*

## register

| 参数 | 类型 | 说明 |
|------|------|------|
| route | any | route handle |
| app | any | app or None |

Register declared route handle `route` on an app. Optional `app`=None falls back
to the app stored on the route object so stepwise agent sessions can omit it.
Page routes call web_app_route; Endpoint `fn` routes call web_app_invoke with
method and body/return contract fields pulled from the handle. User routes must
already be declared on this handle — this tool never invents paths. Prefer this
free tool under `--effects plugin` rather than a method chain when driving
registration from `marqdo call`. Soft-fails with ok=False when neither page nor
fn is present, so callers can detect incomplete declarations early.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | registration result |

> cap.load
**a = app**
1. `a` == None
  **a = [app](access(route, "app"))**
2. *
  **_ = 1**
**page = [page](access(route, "page"))**
**fn = [fn](access(route, "fn"))**
1. `page`
  **path = [path](access(route, "path"))**
  *> web_app_route app=`a` path=`path` page=`page`*
2. `fn`
  **path = [path](access(route, "path"))**
  **method = [method](access(route, "method"))**
  **request = [request](access(route, "request"))**
  **response = [response](access(route, "response"))**
  **class = [class](access(route, "class"))**
  *> web_app_invoke app=`a` path=`path` method=`method` fn=`fn` body=`request` return=`response` class=`class`*
3. *
  *> table.put in=None at="ok" value=False*

## use

| 参数 | 类型 | 说明 |
|------|------|------|
| route | any | route handle |
| middleware | any | middleware table |

Attach middleware table `middleware` to declared route handle `route`
(`|header|value|` or named capability rows). Prefer first-class middleware tables
over configure junk drawers. Call this free tool with `--effects plugin`.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | use result |

> cap.load
*> web_route_use route=`route` middleware=`middleware`*

# route

| 参数 | 类型 | 说明 |
|------|------|------|
| app | any | See prose default / 见叙述默认值 |
| path | any | See prose default / 见叙述默认值 |
| method | any | See prose default / 见叙述默认值 |
| page | any | See prose default / 见叙述默认值 |
| fn | any | See prose default / 见叙述默认值 |
| request | any | See prose default / 见叙述默认值 |
| response | any | See prose default / 见叙述默认值 |
| auth | any | See prose default / 见叙述默认值 |
| class | any | See prose default / 见叙述默认值 |

Inputs: `app`, `path`, `method`="GET", `page`=None, `fn`="", `request`="json", `response`="json", `auth`="", `class`=""

Declared route handle for author method chains. Prefer `web.route` / Artifact
`type: endpoint` discovery via `web.serve`. Optional `class`: `execute` |
`short` | `read` | `control` (empty → heuristic / `legacy_serial`).

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Constructed value / 构造结果 |

> cap.load
`out` =

| _type | app | path | method | page | fn | request | response | auth | class |
|-------|-----|------|--------|------|----|---------|----------|------|-------|
| route | `app` | `path` | `method` | `page` | `fn` | `request` | `response` | `auth` | `class` |

*out*

## use

| 参数 | 类型 | 说明 |
|------|------|------|
| middleware | any | middleware table |

Inputs: `middleware`

Attach middleware table to this route (`|header|value|` or named capability
rows). Prefer explicit tables authors can read; do not restore a configure bag
for headers and cors.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | use result |

> cap.load
*> web_route_use route=`self` middleware=`middleware`*

## register

| 参数 | 类型 | 说明 |
|------|------|------|
| app | any | app or None |

Inputs: `app`=None

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

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | registration result |

> cap.load
**a = app**
1. `a` == None
  **a = [app](access(self, "app"))**
2. *
  **_ = 1**
**page = [page](access(self, "page"))**
**fn = [fn](access(self, "fn"))**
1. `page`
  **path = [path](access(self, "path"))**
  *> web_app_route app=`a` path=`path` page=`page`*
2. `fn`
  **path = [path](access(self, "path"))**
  **method = [method](access(self, "method"))**
  **request = [request](access(self, "request"))**
  **response = [response](access(self, "response"))**
  **class = [class](access(self, "class"))**
  *> web_app_invoke app=`a` path=`path` method=`method` fn=`fn` body=`request` return=`response` class=`class`*
3. *
  *> table.put in=None at="ok" value=False*
