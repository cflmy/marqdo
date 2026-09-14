---
title: web middleware smoke
description: Offline assemble of CORS/security/compress/json middleware (no listen).
import web:ext/web/web.mq.md
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
---

# main

<!-- load web plugin so ABI primitives are registered -->

**p = > plugin.native_path name="web"**
1. `p`
  > plugin.load path=`p`
2. *
  > sys.exit code=1

<!-- CORS table: |允许来源|方法|头|暴露头|凭证| -->

`cors` =

| 允许来源 | 方法 | 头 | 暴露头 | 凭证 |
|----------|------|----|--------|------|
| https://a.example | GET,POST | Content-Type,Authorization | X-Total | true |
| https://b.example | GET | Content-Type |  |  |

<!-- security table: |头|值| -->

`sec` =

| 头 | 值 |
|----|----|
| X-Frame-Options | DENY |
| X-Content-Type-Options | nosniff |
| Content-Security-Policy | default-src 'self' |

<!-- json routes table: |路径|方法|表|条件|排序|上限| -->

`api` =

| 路径 | 方法 | 表 | 条件 | 排序 | 上限 |
|------|------|----|----|----|------|
| /api/posts | GET | articles |  | "-created_at" | 20 |
| /api/publish | POST | articles |  |  | 10 |

<!-- assemble via app.configure -->

**pg = > web.page title="mw"**
**app = > web.app page=`pg`**
**app = > `app`.configure cors=`cors` security=`sec` compress=True body_limit=1048576 json=`api`**

**mw = [middleware](app)**

<!-- cors -->

**cobj = [cors](mw)**
**origins = [allow_origins](cobj)**
**o0 = [1](origins)**
1. `o0` == "https://a.example"
  > print text=cors-origin-ok
2. *
  > print text=cors-origin-fail

**methods = [methods](cobj)**
**m1 = [2](methods)**
1. `m1` == "POST"
  > print text=cors-method-ok
2. *
  > print text=cors-method-fail

**expose = [expose_headers](cobj)**
**e0 = [1](expose)**
1. `e0` == "X-Total"
  > print text=cors-expose-ok
2. *
  > print text=cors-expose-fail

**cred = [credentials](cobj)**
1. `cred`
  > print text=cors-cred-ok
2. *
  > print text=cors-cred-fail

<!-- security -->

**secmap = [security](mw)**
**frame = [X-Frame-Options](secmap)**
1. `frame` == "DENY"
  > print text=security-frame-ok
2. *
  > print text=security-frame-fail

**csp = [Content-Security-Policy](secmap)**
1. `csp` == "default-src 'self'"
  > print text=security-csp-ok
2. *
  > print text=security-csp-fail

<!-- compress / body_limit -->

**comp = [compress](mw)**
1. `comp`
  > print text=compress-ok
2. *
  > print text=compress-fail

**bl = [body_limit](mw)**
1. `bl` == 1048576
  > print text=body-limit-ok
2. *
  > print text=body-limit-fail

<!-- json routes -->

**jmap = [json_routes](mw)**
**posts = [api/posts](jmap)**
**pok = [method](posts)**
1. `pok` == "GET"
  > print text=json-get-ok
2. *
  > print text=json-get-fail

**ptable = [table](posts)**
1. `ptable` == "articles"
  > print text=json-table-ok
2. *
  > print text=json-table-fail

**porder = [order](posts)**
1. `porder` == "-created_at"
  > print text=json-order-ok
2. *
  > print text=json-order-fail

**plimit = [limit](posts)**
1. `plimit` == 20
  > print text=json-limit-ok
2. *
  > print text=json-limit-fail

**pub = [api/publish](jmap)**
**pmethod = [method](pub)**
1. `pmethod` == "POST"
  > print text=json-post-ok
2. *
  > print text=json-post-fail
