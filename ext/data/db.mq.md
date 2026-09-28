---
title: ext/data/db
description: Database Resource — CRUD, txn, migrate, FTS (ADR 0007).
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
import table:lib/table.mq.md
---

## _load

**p = > plugin.native_path name="web"**
1. `p`
  > plugin.load path=`p`
2. *
  > print text=ext/data: native web plugin not found
  > sys.exit code=1
****

# db
    + `url`="sqlite:site.db"

Open a database handle. URL schemes: `sqlite:path` (default), `postgres://…` / `postgresql://…` (same CRUD methods).

> _load
*> web_db_new url=`url`*

## init
    + `name`
    + `fields`

Create a table from a schema table. Optional columns: `唯一`/`unique`, `索引`/`index` (creates UNIQUE / INDEX), `外键`/`fk`/`references` (e.g. `posts.id` or `posts(id)`). Columns named `created_at` / `updated_at` are filled automatically on insert/update when present.

**url = [url](self)**
*> web_db_init url=`url` name=`name` fields=`fields`*

## insert
    + `table`
    + `rows`
    + `txn`=None

**url = [url](self)**
*> web_db_insert url=`url` table=`table` rows=`rows` txn=`txn`*

## select
    + `table`
    + `where`=None
    + `limit`=200
    + `order`=None
    + `txn`=None

Simple filters: one-row map of column→value (AND `=`), or rows `|字段|操作|值|` (`=` `!=` `>` `>=` `<` `<=` `like` `in` `between` `is null`; add `|或|` = `是` to join a row with `OR`). `order` is a column name with optional `-` prefix for descending (`"created_at"`, `"-created_at"`), comma-separated for multiple keys. Pass `txn` to read inside an open transaction. For pages (with a total), use `paginate`.

**url = [url](self)**
**r = > web_db_select url=`url` table=`table` where=`where` limit=`limit` order=`order` offset=None txn=`txn`**
*[rows](r)*

## paginate
    + `table`
    + `where`=None
    + `limit`=200
    + `order`=None
    + `跳过`=0
    + `txn`=None

Like `select` but returns `{ rows, total }` — the total counts rows matching `where` regardless of `limit`/`跳过`, so you can render `上一页 / 下一页`. Set `跳过` to the number of rows to skip (e.g. page 2 with 10 per page ⇒ `跳过`=10).

**url = [url](self)**
*> web_db_select url=`url` table=`table` where=`where` limit=`limit` order=`order` offset=`跳过` txn=`txn`*

## get
    + `table`
    + `id`
    + `txn`=None

**url = [url](self)**
*> web_db_get url=`url` table=`table` id=`id` txn=`txn`*

## update
    + `table`
    + `id`
    + `row`
    + `txn`=None

**url = [url](self)**
*> web_db_update url=`url` table=`table` id=`id` row=`row` txn=`txn`*

## delete
    + `table`
    + `id`
    + `txn`=None

**url = [url](self)**
*> web_db_delete url=`url` table=`table` id=`id` txn=`txn`*

## exec
    + `sql`
    + `args`=None
    + `txn`=None

**url = [url](self)**
*> web_db_exec url=`url` sql=`sql` args=`args` txn=`txn`*

## query
    + `sql`
    + `args`=None
    + `txn`=None

Run bare SQL and return the result set — count / join / group / subqueries. Returns `{ rows, count }`.

**url = [url](self)**
*> web_db_query url=`url` sql=`sql` args=`args` txn=`txn`*

## count
    + `table`
    + `where`=None
    + `txn`=None

Count rows matching a `where` filter (same syntax as `select`). Returns a number.

**url = [url](self)**
**r = > web_db_count url=`url` table=`table` where=`where` txn=`txn`**
*[count](r)*

## migrate
    + `steps`

Apply versioned SQL migrations. `steps` is a `|version|sql|` / `|版本|SQL|` table. Applied versions are recorded in `_marqdo_migrations`. Re-running is a no-op for already-applied versions. SQLite only.

**url = [url](self)**
*> web_db_migrate url=`url` steps=`steps`*

## fts
    + `table`
    + `columns`
    + `name`=None

Create an FTS5 index on `table` for the listed content columns (CSV string or list). Default FTS name is `{table}_fts`. Keeps the index in sync via triggers. Requires integer `id` PK. SQLite only.

**url = [url](self)**
*> web_db_fts_create url=`url` table=`table` columns=`columns` name=`name`*

## search
    + `table`
    + `q`
    + `limit`=20
    + `name`=None

Full-text search (`MATCH`) against the FTS5 index from `fts`. Returns `{ rows, count }` with a `rank` column (bm25). SQLite only.

**url = [url](self)**
*> web_db_search url=`url` table=`table` q=`q` limit=`limit` name=`name`*

## 事务

Begin a transaction: borrows the pooled connection exclusively and returns a
`txn` handle. Write inside it, then `提交` (commit) or `回滚` (roll back).
Every statement runs on the same connection, so a batch is atomic.

**url = [url](self)**
*> web_db_begin url=`url`*

# txn
    + `txn`
    + `url`

A transaction handle from `db.事务`. All CRUD here runs on the transaction's
connection; finish with `提交` or `回滚`.

*self*

## insert
    + `table`
    + `rows`

**url = [url](self)**
**txn = [txn](self)**
*> web_db_insert url=`url` table=`table` rows=`rows` txn=`txn`*

## select
    + `table`
    + `where`=None
    + `limit`=200
    + `order`=None

Same filters as `db.select`; runs inside the transaction.

**url = [url](self)**
**txn = [txn](self)**
**r = > web_db_select url=`url` table=`table` where=`where` limit=`limit` order=`order` offset=None txn=`txn`**
*[rows](r)*

## get
    + `table`
    + `id`

**url = [url](self)**
**txn = [txn](self)**
*> web_db_get url=`url` table=`table` id=`id` txn=`txn`*

## update
    + `table`
    + `id`
    + `row`

**url = [url](self)**
**txn = [txn](self)**
*> web_db_update url=`url` table=`table` id=`id` row=`row` txn=`txn`*

## delete
    + `table`
    + `id`

**url = [url](self)**
**txn = [txn](self)**
*> web_db_delete url=`url` table=`table` id=`id` txn=`txn`*

## exec
    + `sql`
    + `args`=None

**url = [url](self)**
**txn = [txn](self)**
*> web_db_exec url=`url` sql=`sql` args=`args` txn=`txn`*

## 提交

Commit the transaction and return its connection to the pool.

**txn = [txn](self)**
*> web_db_commit txn=`txn`*

## 回滚

Roll the transaction back (undo every write) and return its connection.

**txn = [txn](self)**
*> web_db_rollback txn=`txn`*

