---
markup: occlusion
title: ext/data/db
description: Database Resource — CRUD, txn, migrate, FTS (ADR 0007).
import:
  plugin: lib/plugin.mq.md
  sys: lib/sys.mq.md
  table: lib/table.mq.md
---

Database receivers own a connection URL. Begin a transaction with the database receiver, use its returned identifier for transaction operations, then commit or roll back explicitly. Read [the data workflow](index.mq.md) for a real isolated SQLite example and temporary-directory ownership. Native URLs resolve differently from filesystem helpers: validate an absolute SQLite path inside the granted root before passing it to the plugin. New query behavior belongs in the native web database implementation; schema and filters remain readable tables here.

## _load

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | No business value on completion; a missing native plugin reports the cause and exits this run. |

Resolve and load the native web plugin before resource operations; missing binaries stop this run.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="web"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/data: native web plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.
Return *None*.

# db

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| url | any | "sqlite:site.db" | Resource connection URL; scheme selects the documented backend |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Open a database handle. URL schemes: `sqlite:path` (default), `postgres://…` / `postgresql://…` (same CRUD methods). |

Use `url`="sqlite:site.db" as the inputs for this operation.

Open a database handle. URL schemes: `sqlite:path` (default), `postgres://…` / `postgresql://…` (same CRUD methods).

We **[Load the native plugin required by this operation](_load())**.
Return *[Open a database handle. URL schemes: sqlite:path (default), postgres://… / postgresql://… (same CRUD methods).](web_db_new(url=`url`))*.

## init

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| name | any | required | Name registered or looked up by this operation |
| fields | any | required | Readable field definitions with the columns described by this resource |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Create a table from a schema table. Optional columns: `唯一`/`unique`, `索引`/`index` (creates UNIQUE / INDEX), `外键`/`fk`/`references` (e.g. `posts.id` or `posts(id)`). Columns named `created_at` / `updated_at` are filled automatically on insert/update when present. |

Use `name`, `fields` as the inputs for this operation.

Create a table from a schema table. Optional columns: `唯一`/`unique`, `索引`/`index` (creates UNIQUE / INDEX), `外键`/`fk`/`references` (e.g. `posts.id` or `posts(id)`). Columns named `created_at` / `updated_at` are filled automatically on insert/update when present.

We retain **`url` = [Read url field](access(`self`, "url"))**.
Return *[Create a table from a schema table](web_db_init(url=`url`, name=`name`, fields=`fields`))*.

## insert

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| table | any | required | Database table name or form submission table |
| rows | any | required | Input row records to insert or process |
| txn | any | None | Open transaction returned by the same database resource; None runs outside a transaction |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Insert the supplied row records, optionally within the selected open transaction. |

Use `table`, `rows`, `txn`=None as the inputs for this operation.

Insert the supplied row records, optionally within the selected open transaction.

We retain **`url` = [Read url field](access(`self`, "url"))**.
Return *[Insert the supplied row records, optionally within the selected open transaction.](web_db_insert(url=`url`, table=`table`, rows=`rows`, txn=`txn`))*.

## select

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| table | any | required | Database table name or form submission table |
| where | any | None | Explicit filter map or comparison-row table; omitted means no extra filter |
| limit | any | 200 | Maximum returned entries or candidates; this is not a success threshold |
| order | any | None | Field order specification, with a leading minus for descending database order |
| txn | any | None | Open transaction returned by the same database resource; None runs outside a transaction |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Simple filters: one-row map of column→value (AND `=`), or rows `&#124;字段&#124;操作&#124;值&#124;` (`=` `!=` `>` `>=` `<` `<=` `like` `in` `between` `is null`; add `&#124;或&#124;` = `是` to join a row with `OR`). `order` is a column name with optional `-` prefix for descending (`"created_at"`, `"-created_at"`), comma-separated for multiple keys. Pass `txn` to read inside an open transaction. For pages (with a total), use `paginate`. |

Use `table`, `where`=None, `limit`=200, `order`=None, `txn`=None as the inputs for this operation.

Simple filters: one-row map of column→value (AND `=`), or rows `|字段|操作|值|` (`=` `!=` `>` `>=` `<` `<=` `like` `in` `between` `is null`; add `|或|` = `是` to join a row with `OR`). `order` is a column name with optional `-` prefix for descending (`"created_at"`, `"-created_at"`), comma-separated for multiple keys. Pass `txn` to read inside an open transaction. For pages (with a total), use `paginate`.

We retain **`url` = [Read url field](access(`self`, "url"))**.
We retain **`r` = [Simple filters: one-row map of column→value (AND =), or rows |字段|操作|值| (= != > >= < <= like in between is null; add |或| = 是 to join a row with OR)](web_db_select(url=`url`, table=`table`, where=`where`, limit=`limit`, order=`order`, offset=None, txn=`txn`))**.
Return *[rows](access(r, "rows"))*.

## paginate

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| table | any | required | Database table name or form submission table |
| where | any | None | Explicit filter map or comparison-row table; omitted means no extra filter |
| limit | any | 200 | Maximum returned entries or candidates; this is not a success threshold |
| order | any | None | Field order specification, with a leading minus for descending database order |
| 跳过 | any | 0 | Number of matching rows skipped before the database page |
| txn | any | None | Open transaction returned by the same database resource; None runs outside a transaction |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Like `select` but returns `{ rows, total }` — the total counts rows matching `where` regardless of `limit`/`跳过`, so you can render `上一页 / 下一页`. Set `跳过` to the number of rows to skip (e.g. page 2 with 10 per page ⇒ `跳过`=10). |

Use `table`, `where`=None, `limit`=200, `order`=None, `跳过`=0, `txn`=None as the inputs for this operation.

Like `select` but returns `{ rows, total }` — the total counts rows matching `where` regardless of `limit`/`跳过`, so you can render `上一页 / 下一页`. Set `跳过` to the number of rows to skip (e.g. page 2 with 10 per page ⇒ `跳过`=10).

We retain **`url` = [Read url field](access(`self`, "url"))**.
Return *[Like select but returns { rows, total } — the total counts rows matching where regardless of limit/跳过, so you can render 上一页 / 下一页](web_db_select(url=`url`, table=`table`, where=`where`, limit=`limit`, order=`order`, offset=`跳过`, txn=`txn`))*.

## get

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| table | any | required | Database table name or form submission table |
| id | any | required | Identifier produced or selected by the corresponding resource operation |
| txn | any | None | Open transaction returned by the same database resource; None runs outside a transaction |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Read a single row by its identifier from the selected database table. |

Use `table`, `id`, `txn`=None as the inputs for this operation.

Read a single row by its identifier from the selected database table.

We retain **`url` = [Read url field](access(`self`, "url"))**.
Return *[Read a single row by its identifier from the selected database table.](web_db_get(url=`url`, table=`table`, id=`id`, txn=`txn`))*.

## update

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| table | any | required | Database table name or form submission table |
| id | any | required | Identifier produced or selected by the corresponding resource operation |
| row | any | required | Field/value record to apply to the selected row |
| txn | any | None | Open transaction returned by the same database resource; None runs outside a transaction |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Update the supplied fields of the selected row; use update_if when a revision comparison is required. |

Use `table`, `id`, `row`, `txn`=None as the inputs for this operation.

Update the supplied fields of the selected row; use update_if when a revision comparison is required.

We retain **`url` = [Read url field](access(`self`, "url"))**.
Return *[Update the supplied fields of the selected row; use update_if when a revision comparison is required.](web_db_update(url=`url`, table=`table`, id=`id`, row=`row`, txn=`txn`))*.

## update_if

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| table | any | required | Database table name or form submission table |
| id | any | required | Identifier produced or selected by the corresponding resource operation |
| row | any | required | Field/value record to apply to the selected row |
| expect_col | any | required | Column checked atomically in the conditional database update |
| expect_val | any | required | Expected current value of the comparison column |
| txn | any | None | Open transaction returned by the same database resource; None runs outside a transaction |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Conditional update (CAS) for task/revision control: `UPDATE … WHERE id=? AND expect_col=expect_val`. Returns `{ ok, updated, matched }`; `updated=0` means conflict — do not simulate atomicity with read-then-write in `.mq.md`. |

Use `table`, `id`, `row`, `expect_col`, `expect_val`, `txn`=None as the inputs for this operation.

Conditional update (CAS) for task/revision control: `UPDATE … WHERE id=? AND expect_col=expect_val`. Returns `{ ok, updated, matched }`; `updated=0` means conflict — do not simulate atomicity with read-then-write in `.mq.md`.

We retain **`url` = [Read url field](access(`self`, "url"))**.
Return *[Conditional update (CAS) for task/revision control: UPDATE … WHERE id=? AND expect_col=expect_val](web_db_update_if(url=`url`, table=`table`, id=`id`, row=`row`, expect_col=`expect_col`, expect_val=`expect_val`, txn=`txn`))*.

## delete

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| table | any | required | Database table name or form submission table |
| id | any | required | Identifier produced or selected by the corresponding resource operation |
| txn | any | None | Open transaction returned by the same database resource; None runs outside a transaction |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Delete the selected row, optionally inside the caller-owned transaction. |

Use `table`, `id`, `txn`=None as the inputs for this operation.

Delete the selected row, optionally inside the caller-owned transaction.

We retain **`url` = [Read url field](access(`self`, "url"))**.
Return *[Delete the selected row, optionally inside the caller-owned transaction.](web_db_delete(url=`url`, table=`table`, id=`id`, txn=`txn`))*.

## exec

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| sql | any | required | SQL statement; dynamic values belong in bound args rather than concatenated SQL |
| args | any | None | Arguments passed to the selected command or operation |
| txn | any | None | Open transaction returned by the same database resource; None runs outside a transaction |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Execute SQL with bound arguments and return the native execution outcome. |

Use `sql`, `args`=None, `txn`=None as the inputs for this operation.

Execute SQL with bound arguments and return the native execution outcome.

We retain **`url` = [Read url field](access(`self`, "url"))**.
Return *[Execute SQL with bound arguments and return the native execution outcome.](web_db_exec(url=`url`, sql=`sql`, args=`args`, txn=`txn`))*.

## query

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| sql | any | required | SQL statement; dynamic values belong in bound args rather than concatenated SQL |
| args | any | None | Arguments passed to the selected command or operation |
| txn | any | None | Open transaction returned by the same database resource; None runs outside a transaction |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Run bare SQL and return the result set — count / join / group / subqueries. Returns `{ rows, count }`. |

Use `sql`, `args`=None, `txn`=None as the inputs for this operation.

Run bare SQL and return the result set — count / join / group / subqueries. Returns `{ rows, count }`.

We retain **`url` = [Read url field](access(`self`, "url"))**.
Return *[Run bare SQL and return the result set — count / join / group / subqueries. Returns { rows, count }.](web_db_query(url=`url`, sql=`sql`, args=`args`, txn=`txn`))*.

## count

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| table | any | required | Database table name or form submission table |
| where | any | None | Explicit filter map or comparison-row table; omitted means no extra filter |
| txn | any | None | Open transaction returned by the same database resource; None runs outside a transaction |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Count rows matching a `where` filter (same syntax as `select`). Returns a number. |

Use `table`, `where`=None, `txn`=None as the inputs for this operation.

Count rows matching a `where` filter (same syntax as `select`). Returns a number.

We retain **`url` = [Read url field](access(`self`, "url"))**.
We retain **`r` = [Count rows matching a where filter (same syntax as select). Returns a number.](web_db_count(url=`url`, table=`table`, where=`where`, txn=`txn`))**.
Return *[count](access(r, "count"))*.

## migrate

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| steps | any | required | Ordered migration or operation records in the documented table shape |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Apply versioned SQL migrations. `steps` is a `&#124;version&#124;sql&#124;` / `&#124;版本&#124;SQL&#124;` table. Applied versions are recorded in `_marqdo_migrations`. Re-running is a no-op for already-applied versions. SQLite only. |

Use `steps` as the inputs for this operation.

Apply versioned SQL migrations. `steps` is a `|version|sql|` / `|版本|SQL|` table. Applied versions are recorded in `_marqdo_migrations`. Re-running is a no-op for already-applied versions. SQLite only.

We retain **`url` = [Read url field](access(`self`, "url"))**.
Return *[Apply versioned SQL migrations](web_db_migrate(url=`url`, steps=`steps`))*.

## fts

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| table | any | required | Database table name or form submission table |
| columns | any | required | Column-name list or column definitions required by this operation |
| name | any | None | Name registered or looked up by this operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Create an FTS5 index on `table` for the listed content columns (CSV string or list). Default FTS name is `{table}_fts`. Keeps the index in sync via triggers. Requires integer `id` PK. SQLite only. |

Use `table`, `columns`, `name`=None as the inputs for this operation.

Create an FTS5 index on `table` for the listed content columns (CSV string or list). Default FTS name is `{table}_fts`. Keeps the index in sync via triggers. Requires integer `id` PK. SQLite only.

We retain **`url` = [Read url field](access(`self`, "url"))**.
Return *[Create an FTS5 index on table for the listed content columns (CSV string or list)](web_db_fts_create(url=`url`, table=`table`, columns=`columns`, name=`name`))*.

## search

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| table | any | required | Database table name or form submission table |
| q | any | required | Full-text search query |
| limit | any | 20 | Maximum returned entries or candidates; this is not a success threshold |
| name | any | None | Name registered or looked up by this operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Full-text search (`MATCH`) against the FTS5 index from `fts`. Returns `{ rows, count }` with a `rank` column (bm25). SQLite only. |

Use `table`, `q`, `limit`=20, `name`=None as the inputs for this operation.

Full-text search (`MATCH`) against the FTS5 index from `fts`. Returns `{ rows, count }` with a `rank` column (bm25). SQLite only.

We retain **`url` = [Read url field](access(`self`, "url"))**.
Return *[Full-text search (MATCH) against the FTS5 index from fts. Returns { rows, count } with a rank column (bm25). SQLite only.](web_db_search(url=`url`, table=`table`, q=`q`, limit=`limit`, name=`name`))*.

## 事务

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Begin a transaction: borrows the pooled connection exclusively and returns a Every statement runs on the same connection, so a batch is atomic. |

Begin a transaction: borrows the pooled connection exclusively and returns a
`txn` handle. Write inside it, then `提交` (commit) or `回滚` (roll back).
Every statement runs on the same connection, so a batch is atomic.

We retain **`url` = [Read url field](access(`self`, "url"))**.
Return *[Begin a transaction: borrows the pooled connection exclusively and returns a Every statement runs on the same connection, so a batch is atomic.](web_db_begin(url=`url`))*.

# txn

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| txn | any | required | Open transaction returned by the same database resource; None runs outside a transaction |
| url | any | required | Resource connection URL; scheme selects the documented backend |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | A transaction handle from `db.事务`. All CRUD here runs on the transaction's connection; finish with `提交` or `回滚`. The transaction identifier and connection URL remain explicit fields for every method. |

Use `txn`, `url` as the inputs for this operation.

A transaction handle from `db.事务`. All CRUD here runs on the transaction's
connection; finish with `提交` or `回滚`.

The transaction identifier and connection URL remain explicit fields for every method.

`handle` =

| txn | url |
|---|---|
| `txn` | `url` |

Return *handle*.

## insert

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| table | any | required | Database table name or form submission table |
| rows | any | required | Input row records to insert or process |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Delegate to web_db_insert with the supplied inputs, preserving its result and failure behavior. |

Use `table`, `rows` as the inputs for this operation.

Delegate to web_db_insert with the supplied inputs, preserving its result and failure behavior.

We retain **`url` = [Read url field](access(`self`, "url"))**.
We retain **`txn` = [Read txn field](access(`self`, "txn"))**.
Return *[Delegate to web_db_insert with the supplied inputs, preserving its result and failure behavior.](web_db_insert(url=`url`, table=`table`, rows=`rows`, txn=`txn`))*.

## select

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| table | any | required | Database table name or form submission table |
| where | any | None | Explicit filter map or comparison-row table; omitted means no extra filter |
| limit | any | 200 | Maximum returned entries or candidates; this is not a success threshold |
| order | any | None | Field order specification, with a leading minus for descending database order |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Same filters as `db.select`; runs inside the transaction. |

Use `table`, `where`=None, `limit`=200, `order`=None as the inputs for this operation.

Same filters as `db.select`; runs inside the transaction.

We retain **`url` = [Read url field](access(`self`, "url"))**.
We retain **`txn` = [Read txn field](access(`self`, "txn"))**.
We retain **`r` = [Same filters as db.select; runs inside the transaction.](web_db_select(url=`url`, table=`table`, where=`where`, limit=`limit`, order=`order`, offset=None, txn=`txn`))**.
Return *[rows](access(r, "rows"))*.

## get

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| table | any | required | Database table name or form submission table |
| id | any | required | Identifier produced or selected by the corresponding resource operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Delegate to web_db_get with the supplied inputs, preserving its result and failure behavior. |

Use `table`, `id` as the inputs for this operation.

Delegate to web_db_get with the supplied inputs, preserving its result and failure behavior.

We retain **`url` = [Read url field](access(`self`, "url"))**.
We retain **`txn` = [Read txn field](access(`self`, "txn"))**.
Return *[Delegate to web_db_get with the supplied inputs, preserving its result and failure behavior.](web_db_get(url=`url`, table=`table`, id=`id`, txn=`txn`))*.

## update

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| table | any | required | Database table name or form submission table |
| id | any | required | Identifier produced or selected by the corresponding resource operation |
| row | any | required | Field/value record to apply to the selected row |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Delegate to web_db_update with the supplied inputs, preserving its result and failure behavior. |

Use `table`, `id`, `row` as the inputs for this operation.

Delegate to web_db_update with the supplied inputs, preserving its result and failure behavior.

We retain **`url` = [Read url field](access(`self`, "url"))**.
We retain **`txn` = [Read txn field](access(`self`, "txn"))**.
Return *[Delegate to web_db_update with the supplied inputs, preserving its result and failure behavior.](web_db_update(url=`url`, table=`table`, id=`id`, row=`row`, txn=`txn`))*.

## delete

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| table | any | required | Database table name or form submission table |
| id | any | required | Identifier produced or selected by the corresponding resource operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Delegate to web_db_delete with the supplied inputs, preserving its result and failure behavior. |

Use `table`, `id` as the inputs for this operation.

Delegate to web_db_delete with the supplied inputs, preserving its result and failure behavior.

We retain **`url` = [Read url field](access(`self`, "url"))**.
We retain **`txn` = [Read txn field](access(`self`, "txn"))**.
Return *[Delegate to web_db_delete with the supplied inputs, preserving its result and failure behavior.](web_db_delete(url=`url`, table=`table`, id=`id`, txn=`txn`))*.

## exec

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| sql | any | required | SQL statement; dynamic values belong in bound args rather than concatenated SQL |
| args | any | None | Arguments passed to the selected command or operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Delegate to web_db_exec with the supplied inputs, preserving its result and failure behavior. |

Use `sql`, `args`=None as the inputs for this operation.

Delegate to web_db_exec with the supplied inputs, preserving its result and failure behavior.

We retain **`url` = [Read url field](access(`self`, "url"))**.
We retain **`txn` = [Read txn field](access(`self`, "txn"))**.
Return *[Delegate to web_db_exec with the supplied inputs, preserving its result and failure behavior.](web_db_exec(url=`url`, sql=`sql`, args=`args`, txn=`txn`))*.

## 提交

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Commit the transaction and return its connection to the pool. |

Commit the transaction and return its connection to the pool.

We retain **`txn` = [Read txn field](access(`self`, "txn"))**.
Return *[Commit the transaction and return its connection to the pool.](web_db_commit(txn=`txn`))*.

## 回滚

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Roll the transaction back (undo every write) and return its connection. |

Roll the transaction back (undo every write) and return its connection.

We retain **`txn` = [Read txn field](access(`self`, "txn"))**.
Return *[Roll the transaction back (undo every write) and return its connection.](web_db_rollback(txn=`txn`))*.
