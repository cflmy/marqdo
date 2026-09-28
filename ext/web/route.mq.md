---
title: ext/web/route
description: Declared Endpoint / Document route registration (ADR 0007).
import cap:ext/web/_capability.mq.md
import table:lib/table.mq.md
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

Declared route handle. Prefer `web.route` / Artifact `type: endpoint` discovery via `web.serve`.

> cap.load
`out` =

| _type | app | path | method | page | fn | request | response | auth |
|-------|-----|------|--------|------|----|---------|----------|------|
| route | `app` | `path` | `method` | `page` | `fn` | `request` | `response` | `auth` |

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
  **a = [app](self)**
2. *
  **_ = 1**
**page = [page](self)**
**fn = [fn](self)**
1. `page`
  **path = [path](self)**
  *> web_app_route app=`a` path=`path` page=`page`*
2. `fn`
  **path = [path](self)**
  **method = [method](self)**
  **request = [request](self)**
  **response = [response](self)**
  *> web_app_invoke app=`a` path=`path` method=`method` fn=`fn` body=`request` return=`response`*
3. *
  *> table.put in=None at="ok" value=False*
