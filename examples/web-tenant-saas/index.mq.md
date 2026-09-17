---
title: web-tenant-saas demo
description: Dual-tenant isolation via ?t= query + tenant_scope JSON API.
import web:ext/web/web.mq.md
---

# main

`items_schema` =

| name | type | nullable |
|------|------|----------|
| id | integer | false |
| title | text | false |
| tenant_id | text | false |

`api` =

| path | method | table | order | limit | tenant_scope |
|------|--------|-------|-------|-------|--------------|
| api/items | GET | items | id | 50 | true |

**db = > web.db url="sqlite:data/tenant-saas.db"**
> `db`.init name=items fields=`items_schema`
> `db`.exec sql="DELETE FROM items"
> `db`.exec sql="INSERT INTO items(title, tenant_id) VALUES('Acme note','acme')"
> `db`.exec sql="INSERT INTO items(title, tenant_id) VALUES('Beta note','beta')"

**page = > web.page title="Tenant SaaS" intro="<p>Try <a href='/api/items?t=acme'>/api/items?t=acme</a> vs <a href='/api/items?t=beta'>/api/items?t=beta</a>.</p>"**
**app = > web.app page=`page` db=`db` host="127.0.0.1" port=18092 admin=False**
**app = > `app`.tenant mode="path" param="t" default_scope=False**
**app = > `app`.configure json=`api`**
> `app`.listen
