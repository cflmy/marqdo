---
title: ext/web/web
description: Official web site classes (English). Tables + methods; no bag glue.
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
---

## ensure_plugin

Load the ABI v2 `web` plugin once.

*p = > plugin.native_path name="web"*
1. `p`
  > plugin.load path=`p`
2. *
  > print text=ext/web: native web plugin not found (build marqdo_plugin_web or marqdo ext add web)
  > sys.exit code=1
****

## make_style
    + `name`=""
    + `table`

Assemble a GFM style table into a CSS string. Two shapes are accepted:
`|选择器|属性|值|` rule rows (arbitrary selectors; rows sharing a selector are
merged, and a leading `|媒体|` column groups rows into `@media` blocks), or
`|属性|值|` property rows wrapped as `.name { … }`. Styles stay as data tables,
`make_style` turns them into CSS — 样式即数据、装配即函数.

> ensure_plugin
**> web_style name=`name` table=`table`**

# page
    + `title`="Marqdo Web"
    + `intro`=""

> ensure_plugin
**> web_page_new title=`title` intro=`intro`**

## compose_components
    + `components`

Assemble nav / sidebar / footer from a page table (`|组件|样式|` or `|src|style|`).

**> web_compose_components page=`self` components=`components`**

## compose_main
    + `main`

Assemble main from a bind table (`|属性|值|样式|` or `|front|back|css|`).

**> web_compose_main page=`self` main=`main`**

## query
    + `query`

Attach a DB where-condition map for the main bind. Values may contain `{param}`
placeholders that are resolved from dynamic-route params (e.g. `/post/{slug}`).

**> web_page_query page=`self` query=`query`**

## order
    + `order`

Set a default `ORDER BY` for the main bind (e.g. `-created_at` for newest first).

**> web_page_order page=`self` order=`order`**

## link_prefix
    + `prefix`

Prefix for card links in the main bind (default `/post/`). Cards whose bind has
an `href`/`链接` field become `<a href="{prefix}{href}">`.

**> web_page_link_prefix page=`self` prefix=`prefix`**

## css
    + `css`

Append a raw CSS string to the page's stylesheet (`styles_css`). Use it to ship
a hand-written theme alongside the assembled shell.

**> web_page_css page=`self` css=`css`**

## detail
    + `detail`=True

Render the main bind's first row as a full article (title/meta/tags/body) instead
of a list of cards. Set on the dynamic post page so `/post/{slug}` shows one post.

**> web_page_detail page=`self` detail=`detail`**

## meta
    + `meta`

SEO / OpenGraph metadata as a data table (`|key|value|`). Keys such as `title`, `description`, `canonical`, `og:type` become `<head>` tags at render time.

**> web_page_meta page=`self` meta=`meta`**

## paginate
    + `offset`=0
    + `limit`=10
    + `path`=/

List pagination for the main bind: sets DB `limit`/`offset` and renders previous/next navigation.

**> web_page_paginate page=`self` offset=`offset` limit=`limit` path=`path`**

## compose_form
    + `form`
    + `id`

Embed a form into the page main slot. `listen` auto-registers `GET|POST /_form/{id}` from page/route forms (optional `app.mount_form`).

**> web_compose_form page=`self` form=`form` id=`id`**

## render
    + `db`=None

Offline HTML for tests / previews.

1. `db`
  *url = db[^url]*
  **> web_render page=`self` url=`url`**
2. *
  **> web_render page=`self`**

# style
    + `name`=""

Style tables live as `##` exports in site style modules; compose resolves them by path.

**self**

## process
    + `style`=None
    + `name`=None
    + `path`=None

**self**

# db
    + `url`="sqlite:site.db"

Open a database handle. URL schemes: `sqlite:path` (default), `postgres://…` / `postgresql://…` (same CRUD methods).

> ensure_plugin
**> web_db_new url=`url`**

## init
    + `name`
    + `fields`

Create a table from a schema table. Optional columns: `唯一`/`unique`, `索引`/`index` (creates UNIQUE / INDEX).

*url = self[^url]*
**> web_db_init url=`url` name=`name` fields=`fields`**

## insert
    + `table`
    + `rows`
    + `txn`=None

*url = self[^url]*
**> web_db_insert url=`url` table=`table` rows=`rows` txn=`txn`**

## select
    + `table`
    + `where`=None
    + `limit`=200
    + `order`=None
    + `txn`=None

Simple filters: one-row map of column→value (AND `=`), or rows `|字段|操作|值|` (`=` `!=` `>` `>=` `<` `<=` `like` `in` `between` `is null`; add `|或|` = `是` to join a row with `OR`). `order` is a column name with optional `-` prefix for descending (`"created_at"`, `"-created_at"`), comma-separated for multiple keys. Pass `txn` to read inside an open transaction. For pages (with a total), use `paginate`.

*url = self[^url]*
*r = > web_db_select url=`url` table=`table` where=`where` limit=`limit` order=`order` offset=None txn=`txn`*
**r[^rows]**

## paginate
    + `table`
    + `where`=None
    + `limit`=200
    + `order`=None
    + `跳过`=0
    + `txn`=None

Like `select` but returns `{ rows, total }` — the total counts rows matching `where` regardless of `limit`/`跳过`, so you can render `上一页 / 下一页`. Set `跳过` to the number of rows to skip (e.g. page 2 with 10 per page ⇒ `跳过`=10).

*url = self[^url]*
**> web_db_select url=`url` table=`table` where=`where` limit=`limit` order=`order` offset=`跳过` txn=`txn`**

## get
    + `table`
    + `id`
    + `txn`=None

*url = self[^url]*
**> web_db_get url=`url` table=`table` id=`id` txn=`txn`**

## update
    + `table`
    + `id`
    + `row`
    + `txn`=None

*url = self[^url]*
**> web_db_update url=`url` table=`table` id=`id` row=`row` txn=`txn`**

## delete
    + `table`
    + `id`
    + `txn`=None

*url = self[^url]*
**> web_db_delete url=`url` table=`table` id=`id` txn=`txn`**

## exec
    + `sql`
    + `args`=None
    + `txn`=None

*url = self[^url]*
**> web_db_exec url=`url` sql=`sql` args=`args` txn=`txn`**

## query
    + `sql`
    + `args`=None
    + `txn`=None

Run bare SQL and return the result set — count / join / group / subqueries. Returns `{ rows, count }`.

*url = self[^url]*
**> web_db_query url=`url` sql=`sql` args=`args` txn=`txn`**

## count
    + `table`
    + `where`=None
    + `txn`=None

Count rows matching a `where` filter (same syntax as `select`). Returns a number.

*url = self[^url]*
*r = > web_db_count url=`url` table=`table` where=`where` txn=`txn`*
**r[^count]**

## migrate
    + `steps`

Apply versioned SQL migrations. `steps` is a `|version|sql|` / `|版本|SQL|` table. Applied versions are recorded in `_marqdo_migrations`. Re-running is a no-op for already-applied versions. SQLite only.

*url = self[^url]*
**> web_db_migrate url=`url` steps=`steps`**

## fts
    + `table`
    + `columns`
    + `name`=None

Create an FTS5 index on `table` for the listed content columns (CSV string or list). Default FTS name is `{table}_fts`. Keeps the index in sync via triggers. Requires integer `id` PK. SQLite only.

*url = self[^url]*
**> web_db_fts_create url=`url` table=`table` columns=`columns` name=`name`**

## search
    + `table`
    + `q`
    + `limit`=20
    + `name`=None

Full-text search (`MATCH`) against the FTS5 index from `fts`. Returns `{ rows, count }` with a `rank` column (bm25). SQLite only.

*url = self[^url]*
**> web_db_search url=`url` table=`table` q=`q` limit=`limit` name=`name`**

## 事务

Begin a transaction: borrows the pooled connection exclusively and returns a
`txn` handle. Write inside it, then `提交` (commit) or `回滚` (roll back).
Every statement runs on the same connection, so a batch is atomic.

*url = self[^url]*
**> web_db_begin url=`url`**

# txn
    + `txn`
    + `url`

A transaction handle from `db.事务`. All CRUD here runs on the transaction's
connection; finish with `提交` or `回滚`.

**self**

## insert
    + `table`
    + `rows`

*url = self[^url]*
*txn = self[^txn]*
**> web_db_insert url=`url` table=`table` rows=`rows` txn=`txn`**

## select
    + `table`
    + `where`=None
    + `limit`=200
    + `order`=None

Same filters as `db.select`; runs inside the transaction.

*url = self[^url]*
*txn = self[^txn]*
*r = > web_db_select url=`url` table=`table` where=`where` limit=`limit` order=`order` offset=None txn=`txn`*
**r[^rows]**

## get
    + `table`
    + `id`

*url = self[^url]*
*txn = self[^txn]*
**> web_db_get url=`url` table=`table` id=`id` txn=`txn`**

## update
    + `table`
    + `id`
    + `row`

*url = self[^url]*
*txn = self[^txn]*
**> web_db_update url=`url` table=`table` id=`id` row=`row` txn=`txn`**

## delete
    + `table`
    + `id`

*url = self[^url]*
*txn = self[^txn]*
**> web_db_delete url=`url` table=`table` id=`id` txn=`txn`**

## exec
    + `sql`
    + `args`=None

*url = self[^url]*
*txn = self[^txn]*
**> web_db_exec url=`url` sql=`sql` args=`args` txn=`txn`**

## 提交

Commit the transaction and return its connection to the pool.

*txn = self[^txn]*
**> web_db_commit txn=`txn`**

## 回滚

Roll the transaction back (undo every write) and return its connection.

*txn = self[^txn]*
**> web_db_rollback txn=`txn`**

# form
    + `table`=None
    + `action`=insert
    + `id`=None

Field table + rules table; submit writes through `# db`.

> ensure_plugin
**> web_form_new table=`table` action=`action` id=`id`**

## fields
    + `fields`

**> web_form_fields form=`self` fields=`fields`**

## rules
    + `rules`

**> web_form_rules form=`self` rules=`rules`**

## validate
    + `rules`=None
    + `data`

**> web_form_validate form=`self` rules=`rules` data=`data`**

## render
    + `id`=form
    + `data`=None
    + `errors`=None

**> web_form_render form=`self` id=`id` data=`data` errors=`errors`**

## submit
    + `data`
    + `db`

*url = db[^url]*
**> web_form_submit form=`self` data=`data` url=`url`**

# app
    + `page`
    + `db`=None
    + `admin`=False
    + `host`=127.0.0.1
    + `port`=18081

> ensure_plugin
**> web_app_new page=`page` db=`db` admin=`admin` host=`host` port=`port`**

## route
    + `path`
    + `page`

Mount an assembled page at `path` (e.g. `/about`). `/` is the home `page=`.

**> web_app_route app=`self` path=`path` page=`page`**

## mount_form
    + `id`
    + `form`

Register `GET|POST /_form/{id}` for listen when the form is not already embedded via `page.compose_form`.

**> web_app_mount_form app=`self` id=`id` form=`form`**

## static
    + `dir`
    + `mount`=/static

Serve files from `dir` under `mount` (default `/static`). Path is resolved from the process working directory at listen time.

**> web_app_static app=`self` dir=`dir` mount=`mount`**

## configure
    + `cors`=None
    + `security`=None
    + `compress`=None
    + `body_limit`=None
    + `json`=None
    + `access_log`=None
    + `cache_control`=None

Each capability is declared as a data table and assembled at listen time. The `cors` parameter takes a `|允许来源|方法|头|暴露头|凭证|` table (one row per origin; an empty `允许来源` column means any origin). The `security` parameter takes a `|头|值|` response-header table (e.g. `X-Frame-Options`, `Content-Security-Policy`, `X-Content-Type-Options`, `Referrer-Policy`, `Strict-Transport-Security`). Set `compress` to `True` to gzip response bodies. `body_limit` is the max request body bytes (e.g. `1048576`). The `json` parameter takes a `|路径|方法|表|条件|排序|上限|` table of JSON API endpoints backed by DB queries (each returns `application/json`). Set `access_log` to `True` to log `METHOD path status duration_ms` on stderr. `cache_control` sets a global `Cache-Control` header (e.g. `public, max-age=3600`).

Tables stay as data; `configure` assembles them. 配置即数据、装配即函数.

**> web_app_middleware app=`self` cors=`cors` security=`security` compress=`compress` body_limit=`body_limit` json_routes=`json` access_log=`access_log` cache_control=`cache_control`**

## listen

Serve `/`, routed pages, `/_part/{id}` (home) and `{path}/_part/{id}` (routes), `/_form/{id}` (from mounts + page embeds), optional `/static` (or custom mount), optional `/admin`, upload/download/WebSocket/RSS/sitemap/robots routes, redirects, and a custom 404 fallback. Production HTTPS should terminate at a reverse proxy; set `cookie_secure=True` when serving over TLS.

**> web_listen app=`self`**

## route_ws
    + `path`
    + `echo`=True
    + `mode`=None

Register a WebSocket endpoint at `path` (e.g. `/live`). `mode` is `echo` (default), `broadcast` (fan-out text to all sockets on this path), or `drain`. Legacy `echo=False` maps to `drain`. Connect from a client with `web.ws.connect`.

**> web_app_route_ws app=`self` path=`path` echo=`echo` mode=`mode`**

## auth
    + `users`
    + `session_ttl`=3600

Keep the app's `admin=True`, and gate `/admin*` behind a login page. `users` is an admin-users table (`|username|password|` / `|用户名|密码|`); unauthenticated requests redirect to `/admin/login`.

**> web_app_auth app=`self` users=`users` session_ttl=`session_ttl`**

## route_rss
    + `path`
    + `table`
    + `limit`=20
    + `order`=-created_at
    + `title`=Feed
    + `link`=/
    + `description`=""

Register an RSS 2.0 feed at `path` (e.g. `/feed.xml`) backed by a DB table.

**> web_app_route_rss app=`self` path=`path` table=`table` limit=`limit` order=`order` title=`title` link=`link` description=`description`**

## redirect
    + `from`
    + `to`
    + `permanent`=False

Register a redirect from `from` to `to`. `permanent=True` issues HTTP 301; otherwise 307.

**> web_app_redirect app=`self` from=`from` to=`to` permanent=`permanent`**

## error_page
    + `status`=404
    + `page`

Bind an assembled page for HTTP `404` or `500` responses.

**> web_app_error_page app=`self` status=`status` page=`page`**

## sitemap
    + `path`=/sitemap.xml
    + `base`=""
    + `table`=None
    + `loc`=path
    + `limit`=1000
    + `items`=None

Serve `sitemap.xml`. Prefer `table` + `loc` column from the DB, or pass an `items` table with `loc`/`路径` rows.

**> web_app_sitemap app=`self` path=`path` base=`base` table=`table` loc=`loc` limit=`limit` items=`items`**

## robots
    + `body`=None
    + `sitemap`=None

Serve `/robots.txt`. Omit `body` to emit a default Allow-all file, optionally with a `Sitemap:` line.

**> web_app_robots app=`self` body=`body` sitemap=`sitemap`**

## upload
    + `path`=/_upload
    + `field`=file
    + `storage`
    + `prefix`=uploads/
    + `max_bytes`=5242880
    + `types`=None

Mount `POST path` for multipart file upload. `storage` is a `# storage` handle or `file:…` / `s3://…` URL. Optional `types` is a `|type|ext|` / `|类型|扩展名|` allowlist (or a MIME CSV string). Success returns JSON `{ok,key,size,content_type}`.

**> web_app_upload app=`self` path=`path` field=`field` storage=`storage` prefix=`prefix` max_bytes=`max_bytes` types=`types`**

## download
    + `path`=/_media/{*key}
    + `storage`
    + `disposition`=attachment

Mount `GET path` to stream an object. Path must capture `key` (use `{*key}` for nested keys). `disposition` is `attachment` or `inline`.

**> web_app_download app=`self` path=`path` storage=`storage` disposition=`disposition`**

# auth
    + `users`
    + `session_ttl`=3600

Session/auth helper. Constructs a config object; `login` validates against the users table. To gate `/admin` on this app, use `app.auth users=…` instead.

> ensure_plugin
**> web_auth_new users=`users` session_ttl=`session_ttl`**

## login
    + `username`
    + `password`

Validate credentials against the users table and create a session. Returns `{ok, session_id, username}`.

*users = self[^users]*
*ttl = self[^session_ttl]*
**> web_auth_login username=`username` password=`password` users=`users` session_ttl=`ttl`**

## check
    + `session_id`

Returns `{ok, username}` when the session is valid.

**> web_auth_check session_id=`session_id`**

## logout
    + `session_id`

Destroy the session.

**> web_auth_logout session_id=`session_id`**

## hash_password
    + `password`

Hash a plaintext password for storage in admin user tables (argon2id). Store the returned `hash` in the `password` column; login verifies automatically.

**> web_password_hash password=`password`**

# cache
    + `url`="memory:"

Key–value cache. Use `memory:` for in-process (tests / single process) or `redis://host:6379/0` for Redis.

> ensure_plugin
**> web_cache_new url=`url`**

## get
    + `key`

*url = self[^url]*
**> web_cache_get url=`url` key=`key`**

## set
    + `key`
    + `value`
    + `ttl`=None

Optional `ttl` is seconds until expiry.

*url = self[^url]*
**> web_cache_set url=`url` key=`key` value=`value` ttl=`ttl`**

## del
    + `key`

*url = self[^url]*
**> web_cache_del url=`url` key=`key`**

## exists
    + `key`

*url = self[^url]*
**> web_cache_exists url=`url` key=`key`**

## ttl
    + `key`

*url = self[^url]*
**> web_cache_ttl url=`url` key=`key`**

# storage
    + `url`="file:data/blobs"

Object storage. `file:dir` stores blobs on disk (offline / gold). `s3://bucket?endpoint=http://127.0.0.1:9000&access_key=…&secret_key=…` talks to MinIO / S3.

> ensure_plugin
**> web_storage_new url=`url`**

## put
    + `key`
    + `body`=None
    + `path`=None
    + `content_type`="application/octet-stream"

Provide either `body` (text) or `path` (local file to upload).

*url = self[^url]*
**> web_storage_put url=`url` key=`key` body=`body` path=`path` content_type=`content_type`**

## get
    + `key`

*url = self[^url]*
**> web_storage_get url=`url` key=`key`**

## delete
    + `key`

*url = self[^url]*
**> web_storage_delete url=`url` key=`key`**

## list
    + `prefix`=""

*url = self[^url]*
**> web_storage_list url=`url` prefix=`prefix`**

# media
    + `storage`=None

Offline helpers for upload validation and saving into `# storage` (also used by HTTP `app.upload`).

> ensure_plugin
**> web_media_new storage=`storage`**

## validate
    + `filename`
    + `content_type`="application/octet-stream"
    + `size`
    + `max_bytes`=5242880
    + `types`=None

**> web_upload_validate filename=`filename` content_type=`content_type` size=`size` max_bytes=`max_bytes` types=`types`**

## save
    + `path`
    + `key`=None
    + `content_type`="application/octet-stream"
    + `prefix`=uploads/
    + `storage`=None

*st = storage*
1. `st` == None
  *st = self[^storage]*
2. *

**> web_upload_save storage=`st` path=`path` key=`key` content_type=`content_type` prefix=`prefix`**

# ws
    + `timeout_sec`=30

WebSocket client helper.

> ensure_plugin
`out` =

| timeout_sec | _type |
|-------------|-------|
| `timeout_sec` | ws |

**out**

## connect
    + `url`
    + `message`=""
    + `headers`=None

Single request–response: connect to `url`, send `message`, collect all server text replies, close. Returns `{ok, messages}`.

*timeout = self[^timeout_sec]*
**> web_ws_connect url=`url` message=`message` headers=`headers` timeout_sec=`timeout`**
