---
title: web P3 smoke — audit timestamps, FK, RBAC, gallery (offline)
import web:ext/web/web.mq.md
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
import json:lib/json.mq.md
---

# main

*p = > plugin.native_path name="web"*
1. `p`
  > plugin.load path=`p`
2. *
  > sys.exit code=1

*store = > web.db url="sqlite:web-fixtures/data/p3-smoke.db"*
> `store`.exec sql="DROP TABLE IF EXISTS comments"
> `store`.exec sql="DROP TABLE IF EXISTS posts"

`post_fields` =

| 字段 | 类型 | 可空 |
|------|------|------|
| id | int | 否 |
| title | text | 否 |
| created_at | timestamp | 是 |
| updated_at | timestamp | 是 |

> `store`.init name="posts" fields=`post_fields`

`comment_fields` =

| 字段 | 类型 | 可空 | 外键 |
|------|------|------|------|
| id | int | 否 | |
| post_id | int | 否 | posts |
| body | text | 否 | |

> `store`.init name="comments" fields=`comment_fields`

`rows` =

| @ | title |
|---|-------|
| 1 | Hello P3 |

> `store`.insert table="posts" rows=`rows`

*got = > `store`.get table="posts" id=1*
*ca = got[^created_at]*
1. `ca`
  > print text=audit-insert-ok
2. *
  > print text=audit-insert-fail

*upd = > json.parse text={"title":"Hello P3b"}*
> `store`.update table="posts" id=1 row=`upd`
*got2 = > `store`.get table="posts" id=1*
*ua2 = got2[^updated_at]*
1. `ua2`
  > print text=audit-update-ok
2. *
  > print text=audit-update-fail

*fk = > `store`.query sql="SELECT sql FROM sqlite_master WHERE type='table' AND name='comments'"*
*fkddl = fk[^rows][^1][^sql]*
1. `fkddl`
  > print text=fk-ok
2. *
  > print text=fk-fail

`users` =

| username | password | role |
|----------|----------|------|
| admin | secret | admin |
| alice | secret | author |

*login = > web_auth_login username="alice" password="secret" users=`users` session_ttl=120*
*role = login[^role]*
1. `role` == "author"
  > print text=rbac-login-ok
2. *
  > print text=rbac-login-fail

*pg = > web.page title="p3"*
*app = > web.app page=`pg` port=18130*
*app = > `app`.auth users=`users`*
*app = > `app`.gate path="/write*" roles="admin,author"*
*gates = app[^gates]*
*ng = > len value=`gates`*
1. `ng` >= 2
  > print text=rbac-gate-ok
2. *
  > print text=rbac-gate-fail

*blob = > web.storage url="file:web-fixtures/data/p3-gallery"*
> `blob`.put key="uploads/a.txt" body="hi" content_type="text/plain"
*app = > `app`.gallery path="/gallery" storage=`blob` prefix="uploads/" title="Media" download_base="/_media"*
*gal = app[^gallery_routes]*
*g = gal[^/gallery]*
1. `g`
  > print text=gallery-ok
2. *
  > print text=gallery-fail
