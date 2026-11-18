---
markup: occlusion
title: web-net-site database lifecycle
import:
  data: ext/data/db.mq.md
  articles: articles.mq.md
  fs: lib/fs.mq.md
  text: lib/text.mq.md
---

The database entry connects schema and literal seed leaves to the actual data resource. It no longer uses the removed web database constructor. Grant fs.write and plugin with the site directory as --fs-root; the SQLite path is resolved through that native root rather than the process working directory.

## open

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| url | any | None | None selects data/site-net.db under the site root; tests provide a disposable explicit URI |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Initialized articles database resource with existing rows preserved |

Use `url`=None to open the site's local database. Create the default data directory through native file operations, then resolve the actual SQLite filename within the granted root. An explicit SQLite path goes through the same root and symlink checks before the native database resource receives it. Other supported database URIs retain their resource-specific backend contract; they are not sandboxed by filesystem permissions.

The schema reader preserves native field types and nullability. Initialization ensures the table exists, queries one row to distinguish an empty table, and inserts the original demonstration content only into an empty table. An existing or partly populated table is kept unchanged. This is not a column migrator, data repair tool or reason to delete a user's articles. A native connection, schema or insert diagnostic stops the caller before it starts serving.

For isolated verification choose a caller-owned temporary SQLite file and copy the article data leaf under that granted site root. Keep the returned resource in the same program for subsequent queries. Close the process before cleaning its owned directory; do not run tests against the example's existing data directory.

1. `url` == None
  We **[Create only the site's default data directory](fs.make_dirs(path="data"))** and **file = [Resolve its database filename](fs.resolve(path="data/site-net.db"))** and **url = "sqlite:" + file**.
1. [The explicit or default URI selects SQLite](text.starts_with(text=`url`, prefix="sqlite:"))
  We **file = [Read the selected SQLite path](text.replace(text=`url`, old="sqlite:", new="", count=1))** and **file = [Apply the native file-root policy](fs.resolve(path=`file`))** and **url = "sqlite:" + file**.
We **store = [Open the selected actual database resource](data.db(url=`url`))** and **fields = [Read the documented article schema](articles.schema())**.
We **[Ensure the table without clearing it](> `store`.init name="articles" fields=`fields`)** and **existing = [Check whether business records already exist](> `store`.select table="articles" limit=1)**.
1. not `existing`
  We **seed = [Read the original demonstration data](articles.seed())** and **[Seed only the empty article table](> `store`.insert table="articles" rows=`seed`)**.
Once the selected database is ready, return *store* for the site's resource bindings.
