---
title: web middleware live server
description: Offline-assembled app that listens with CORS/security/compress/body_limit/json.
import web:ext/web/web.mq.md
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
---

# main

<!-- load web plugin so ABI primitives are registered -->

*p = > plugin.native_path name="web"*
1. `p`
  > plugin.load path=`p`
2. *
  > sys.exit code=1

`schema` =

| 字段 | 类型 |
|------|------|
| title | text |
| body | text |

*store = > web.db url="sqlite:web-fixtures/data/mw-live.db"*
> `store`.init name=articles fields=`schema`
> `store`.exec sql="DELETE FROM articles"

`rows` =

| title | body |
|-------|------|
| Hello | first post |

> `store`.insert table=articles rows=`rows`

*pg = > web.page title="live"*
*app = > web.app page=`pg` db=`store` port=18099*

`cors` =

| 允许来源 | 方法 | 头 | 暴露头 | 凭证 |
|----------|------|----|--------|------|
| https://a.example | GET,POST | Content-Type | X-Total | true |

`sec` =

| 头 | 值 |
|----|----|
| X-Frame-Options | DENY |
| X-Content-Type-Options | nosniff |

`api` =

| 路径 | 方法 | 表 | 条件 | 排序 | 上限 |
|------|------|----|----|----|------|
| /api/posts | GET | articles |  |  | 10 |
| /api/publish | POST | articles |  |  | 10 |

*app = > `app`.configure cors=`cors` security=`sec` compress=True body_limit=1000 json=`api`*
> `app`.listen
