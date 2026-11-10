---
markup: occlusion
title: Session-aware nav component
import:
  component: ext/web/component/index.mq.md
---

This module owns only the selected navigation responsibility. Its labels and destinations retain the original site order. The session-dependent rows are a typed navigation resource because their visibility must follow each request rather than a startup snapshot.

## nav

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | One typed navigation resource with stable fragment identity |

Construction is pure and does not log in, log out or register a route. Empty when values keep links visible; guest and auth use the actual session supplied by the server. The resource renderer retains native navigation classes for the existing theme. Hiding Admin never protects its URL: the serving entry separately binds a real /admin role gate.

The parent places this region and owns its grid relationship. Edit labels here and update declared routes when changing a destination; do not concatenate an HTML navigation string or return an obsolete front/back binding table.

`items` =

| @ | label | href | when |
|---|---|---|---|
| 1 | Marqdo | / |  |
| 2 | About | /about |  |
| 3 | New | /new |  |
| 4 | Login | /login | guest |
| 5 | Admin | /admin | auth |
| 6 | Logout | /admin/logout | auth |

We **resource = [Describe current-session navigation](component.navigation(items=`items`, region="nav"))**.
Return *[Keep this region as a typed View resource](component.resource(value=`resource`, slot="nav"))*.
