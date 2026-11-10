---
markup: occlusion
title: Tenant demonstration database
import:
  db: ext/data/db.mq.md
  fs: lib/fs.mq.md
---

This initializer owns one demonstration table. It keeps existing rows and seeds only an empty table. Pass a disposable absolute SQLite URI for tests; no caller should need to delete a user's data to repeat initialization.

## open

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| url | any | None | None chooses data/tenant-saas.db under the granted site root; otherwise an explicit SQLite URI |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Initialized database resource containing retained user rows or initial demo notes |

Use `url`=None with fs.write and plugin on the site root. The default path is resolved through the native filesystem so its location does not change with the process working directory. An explicit SQLite URI belongs to the caller, who must grant and manage that native database boundary. This initializer never deletes rows or resets ids during a restart.

The schema preserves the title and tenant_id fields used by the scoped query. Check for any existing row before seeding; a nonempty or partially populated database is retained as supplied. Only a completely empty table receives the two notes. This is an idempotent demonstration initializer, not a migration or repair tool.

1. `url` == None
  We **[Create the site's selected data directory](fs.make_dirs(path="data"))** and **path = [Resolve the native database filename](fs.resolve(path="data/tenant-saas.db"))** and **url = "sqlite:" + path**.

`fields` =

| name | type | nullable |
|---|---|---|
| id | integer | False |
| title | text | False |
| tenant_id | text | False |

We **store = [Open the selected database resource](db.db(url=`url`))** and **[Ensure the demonstration table exists](> `store`.init name="items" fields=`fields`)** and **existing = [Observe whether any business rows already exist](> `store`.select table="items" limit=1)**.
1. not `existing`

  `rows` =

  | @ | title | tenant_id |
  |---|---|---|
  | 1 | Acme note | acme |
  | 2 | Beta note | beta |

  We **[Insert both initial notes only into the empty table](> `store`.insert table="items" rows=`rows`)**.
Once the selected database is ready, return *store* for route discovery and binding.
