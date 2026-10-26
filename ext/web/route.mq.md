---
markup: occlusion
title: ext/web/route
description: Declared Endpoint / Document route registration (ADR 0007).
import:
  cap: ext/web/_capability.mq.md
  table: lib/table.mq.md
---

# route
    + `app`
    + `path`
    + `method`="GET"
    + `page`=None
    + `fn`=""
    + `request`="json"
    + `response`="json"
    + `auth`=""
    + `class`=""

Declared route handle. Prefer `web.route` / Artifact `type: endpoint` discovery via `web.serve`.
Optional `class`: `execute` | `short` | `read` | `control` (empty → heuristic / `legacy_serial`).

> cap.load
`out` =

| _type | app | path | method | page | fn | request | response | auth | class |
|-------|-----|------|--------|------|----|---------|----------|------|-------|
| route | `app` | `path` | `method` | `page` | `fn` | `request` | `response` | `auth` | `class` |

*out*

## use
    + `middleware`

Attach middleware table to this route (`|header|value|` or named capability rows).

> cap.load
*> web_route_use route=`self` middleware=`middleware`*

## register
    + `app`=None

Register this declared route on an app handle.

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
