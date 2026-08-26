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

> ensure_plugin
**> web_db_new url=`url`**

## init
    + `name`
    + `fields`

*url = self[^url]*
**> web_db_init url=`url` name=`name` fields=`fields`**

## insert
    + `table`
    + `rows`

*url = self[^url]*
**> web_db_insert url=`url` table=`table` rows=`rows`**

## select
    + `table`
    + `where`=None
    + `limit`=200
    + `order`=None

Simple filters: one-row map of column→value (AND `=`), or rows `|字段|操作|值|` (`=` `!=` `>` `>=` `<` `<=` `like`). `order` is a column name with optional `-` prefix for descending (`"created_at"`, `"-created_at"`), comma-separated for multiple keys.

*url = self[^url]*
**> web_db_select url=`url` table=`table` where=`where` limit=`limit` order=`order`**
**r[^rows]**

## get
    + `table`
    + `id`

*url = self[^url]*
**> web_db_get url=`url` table=`table` id=`id`**

## update
    + `table`
    + `id`
    + `row`

*url = self[^url]*
**> web_db_update url=`url` table=`table` id=`id` row=`row`**

## delete
    + `table`
    + `id`

*url = self[^url]*
**> web_db_delete url=`url` table=`table` id=`id`**

## exec
    + `sql`
    + `args`=None

*url = self[^url]*
**> web_db_exec url=`url` sql=`sql` args=`args`**

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

## listen

Serve `/`, routed pages, `/_part/{id}` (home) and `{path}/_part/{id}` (routes), `/_form/{id}` (from mounts + page embeds), optional `/static` (or custom mount), and optional `/admin`.

**> web_listen app=`self`**

## route_ws
    + `path`
    + `echo`=True

Register a WebSocket endpoint at `path` (e.g. `/live`). With `echo=True` (default) the server replies to each received text frame; otherwise frames are drained. Connect from a client with `web.ws.connect`.

**> web_app_route_ws app=`self` path=`path` echo=`echo`**

## auth
    + `users`
    + `session_ttl`=3600

Keep the app's `admin=True`, and gate `/admin*` behind a login page. `users` is an admin-users table (`|username|password|` / `|用户名|密码|`); unauthenticated requests redirect to `/admin/login`.

**> web_app_auth app=`self` users=`users` session_ttl=`session_ttl`**

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
