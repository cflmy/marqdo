---
markup: occlusion
title: ext/web/web
description: >-
  Web Artifact facade (ADR 0007) — page · route · serve · render · inspect · use.
  Document/Endpoint first-class. No compose_* / configure / author ensure_plugin.
  Agents: read ext/web/index.mq.md first (layout file-partition law).
import:
  cap: ext/web/_capability/index.mq.md
  page_mod: ext/web/page/index.mq.md
  route_mod: ext/web/route/index.mq.md
  dom_mod: ext/web/dom/index.mq.md
  client_mod: ext/web/client/index.mq.md
  table: lib/table.mq.md
---

Web facade — [start at the root handbook](index.mq.md), then use this module
for server lifecycle and rendering. Layout assembly remains in pure component
functions; database, security and network clients stay in their own packages.

```sh
marqdo call ext/web/web.mq.md render 'body="# A Document"' 'title="Preview"' --effects plugin --json
marqdo call ext/web/web.mq.md inspect 'root="examples/web-layout-handbook"' --effects plugin --json
```

inspect with root scans declared metadata without starting a server. To inspect
an in-memory app, pass its actual handle; an empty fresh process cannot inspect
another process. serve is a blocking lifecycle operation: write a launcher and
use marqdo run, not call. Explicit view: main Documents assemble once at startup
through normal functions and imports. Restart after changing their content.

Develop layout structure in layout/component and the Go artifact renderer;
keep this facade as orchestration. Preserve the existing app object and manual
route API. Contracts precede execution; ordinary prose titles are not functions.

## page

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| title | text | "" | document title |
| route | text | "/" | path |
| method | text | "GET" | HTTP method |
| body | text | "" | Markdown body |
| nodes | any | None | View table or None |
| data_source | text | "" | table name |
| data_order | text | "" | order key |
| data_where | text | "" | filters |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | page handle |

Use `title`="", `route`="/", `method`="GET", `body`="", `nodes`=None, `data_source`="", `data_order`="", `data_where`="" as the inputs for this operation.

Construct a Document handle with `title`="", `route`="/", `method`="GET",

`body`="", optional View `nodes`=None, and flat `data_source`="", `data_order`="",

`data_where`="" keys. Prefer discovering type: web Artifact files via serve
rather than hand-building every page in memory. nodes must be the unified View
schema, never a private dialect or an HTML string shell.

Return *[Page](page_mod.page(title=`title`, route=`route`, method=`method`, body=`body`, nodes=`nodes`, data_source=`data_source`, data_order=`data_order`, data_where=`data_where`))*.

## route

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| app | any | required | app handle |
| path | text | required | URL path |
| method | text | "GET" | HTTP method |
| page | any | None | Document or None |
| fn | text | "" | endpoint member |
| request | text | "json" | request kind |
| response | text | "json" | response kind |
| auth | text | "" | auth tag |
| class | text | "" | route class |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | registration result |

Use `app`, `path`, `method`="GET", `page`=None, `fn`="", `request`="json", `response`="json", `auth`="", `class`="" as the inputs for this operation.

Register a declared Document page or Endpoint on `app` at `path` with

`method`="GET" . `page`=None holds a Document; `fn`="" names an Endpoint
lib.member. `request`="json", `response`="json", `auth`="", and `class`=""
document the contract. Optional class values include execute, short, read,
control. User routes must be declared — never implicit. After building the
route object this tool registers it on the app.

We retain **`r` = [Route](route_mod.route(app=`app`, path=`path`, method=`method`, page=`page`, fn=`fn`, request=`request`, response=`response`, auth=`auth`, class=`class`))**.
Return *[Register](> `r`.register app=`app` )*.

## render

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| doc | any | None | Document or None |
| nodes | any | None | View table or None |
| body | text | "" | Markdown |
| title | text | "" | title supplied by the caller |
| db | any | None | optional db |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | HTML or payload |

Use `doc`=None, `nodes`=None, `body`="", `title`="", `db`=None as the inputs for this operation.

Render a Document or View nodes to HTML for offline preview. Prefer nodes from
file-partitioned components. Optional `doc`=None is a Document handle; when set,
delegates to page.render. Optional `nodes`=None is a View table; when set without
doc, renders that table. Otherwise renders Markdown `body`="" with optional

`title`="" and `db`=None . Requires the native plugin. Do not treat this as
permission to paste a full HTML homepage — keep using View tables and Markdown.

We **[Load the native plugin required by this operation](cap.load())**.
1. `doc`
  Return *[Render](> `doc`.render nodes=`nodes` db=`db` )*.
2. `nodes`
  Return *[Render a Document or View nodes to HTML for offline preview](web_render_nodes(page=`doc`, nodes=`nodes`, db=`db`, title=`title`))*.
3. *
  Return *[Render a Document or View nodes to HTML for offline preview](web_render_document(page=`doc`, body=`body`, db=`db`, title=`title`))*.

## use

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| middleware | any | required | middleware table |
| app | any | required | app handle |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | app handle |

Use `middleware`, `app` as the inputs for this operation.

Attach first-class `middleware` to `app`. Pass a GFM table (security headers,
cors row, and similar). This replaces the old configure(...) junk drawer with an
explicit table authors can read.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Attach first-class middleware to app](web_app_use(app=`app`, middleware=`middleware`))*.

## inspect

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| app | any | None | app or None |
| root | text | "" | Site directory to inspect without listening |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | {declared, system} |

Use `app`=None, `root`="" as the inputs for this operation.

Return declared versus system route tables for introspection. System routes such
as /_part and /_form must appear here. Use this to verify a site after serve or
manual registration. Optional `app`=None selects which application to inspect.

Optional `root`="" scans a site directory when supplied; it does not start a server.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Return declared versus system route tables for introspection](web_inspect(app=`app`, root=`root`))*.

## serve

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| root | text | "." | site root |
| host | text | "127.0.0.1" | bind host |
| port | number | 18081 | Explicit server port |
| db | any | None | optional db |
| static_dir | text | "" | static files |
| middleware | any | None | optional middleware |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | serve result |

Use `root`=".", `host`="127.0.0.1", `port`=18081, `db`=None, `static_dir`="", `middleware`=None as the inputs for this operation.

Scan `root`="." for type web or endpoint Artifacts, register declared routes,
attach system routes, and listen on `host`="127.0.0.1" and `port`=18081 . The
site directory should already follow the layout law (components split until
simple). Optional `db`=None, `static_dir`="", and `middleware`=None bind
resources without stuffing them into this facade as author APIs.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Scan root="](web_serve_root(root=`root`, host=`host`, port=`port`, db=`db`, static_dir=`static_dir`, middleware=`middleware`))*.

## client_embed

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| bridge | text | "/static/marqdo-bridge.js" | bridge script URL |
| wasm | text | "/static/marqdo_wasm.wasm" | wasm URL |
| source | text | "" | optional source |
| boot | bool | True | auto boot |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | embed descriptor |

Use `bridge`="/static/marqdo-bridge.js", `wasm`="/static/marqdo_wasm.wasm", `source`="", `boot`=True as the inputs for this operation.

Describe WASM bridge embed tags for a page that needs client Marqdo. Defaults:

`bridge`="/static/marqdo-bridge.js", `wasm`="/static/marqdo_wasm.wasm",

`source`="", `boot`=True . This is not a substitute for Document layout: still
partition nav and main in files, then embed the bridge where interaction needs
it.

Return *[Embed](client_mod.embed(bridge=`bridge`, wasm=`wasm`, source=`source`, boot=`boot`))*.

## text_patch

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| sel | text | required | CSS selector |
| text | text | required | new text |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | patch descriptor |

Use `sel`, `text` as the inputs for this operation.

Client patch: set text content for CSS selector `sel` to string `text`. Use for
small interactive updates after a Document is already laid out with View tables —
not for building the initial homepage shell.

Return *[Text patch](client_mod.text_patch(sel=`sel`, text=`text`))*.

## dom_patch

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| sel | text | required | CSS selector |
| html | text | "" | optional HTML fragment |
| attrs | any | None | optional attrs |
| remove | bool | False | remove node |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | patch descriptor |

Use `sel`, `html`="", `attrs`=None, `remove`=False as the inputs for this operation.

Client patch for selector `sel` with optional `html`="", `attrs`=None, or

`remove`=False . Treat html here as a surgical update, never as the primary way
to author nav, sidebar, or footer. Layout law still applies to the Document that
hosts the selector.

Return *[Dom patch](client_mod.dom_patch(sel=`sel`, html=`html`, attrs=`attrs`, remove=`remove`))*.

## make_style

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| table | any | required | property table |
| name | text | "" | style name |
| strict | bool | False | strict mode |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | CSS / style handle |

Use `table`, `name`="", `strict`=False as the inputs for this operation.

Compile a named style `table` into CSS. Optional `name`="" and `strict`=False .
Styles remain data next to content; this is not a hidden page DSL. Prefer
referencing style names from View style columns after file-partitioned
components define their look.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Compile a named style table into CSS](web_style(name=`name`, table=`table`, strict=`strict`))*.

## make_images

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| table | any | required | images table |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | images payload |

Use `table` as the inputs for this operation.

Build image descriptors from a readable `table` of sources and alts. Keep media
declarations in Markdown tables beside the Document that uses them.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Build image descriptors from a readable table of sources and alts. Keep media declarations in Markdown tables beside the Document that uses them.](web_images(table=`table`))*.

## make_head

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| table | any | required | head table |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | head payload |

Use `table` as the inputs for this operation.

Build head resource rows (link, script, icon) from a `table`. Prefer declaring
head needs as data rather than injecting opaque HTML head strings.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Build head resource rows (link, script, icon) from a table. Prefer declaring head needs as data rather than injecting opaque HTML head strings.](web_head(table=`table`))*.

# app

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| page | any | None | Initial Document/page resource, or None for explicit later route mounts |
| db | any | None | Existing database resource used for document data and native application services |
| host | any | "127.0.0.1" | Listen address; the default binds only the local host |
| port | any | 18081 | TCP listen port selected for the application |
| admin | any | False | Whether to enable the native administration routes |
| admin_prefix | any | "/admin" | URL prefix under which native administration routes are mounted |
| shell_css | any | None | Optional native shell stylesheet configuration; None keeps the runtime default |
| layout | any | None | Optional native shell layout configuration; component geometry remains View data |
| asset_version | any | None | Optional resource version used by the native shell for asset URLs |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Native app resource for static mounts, route registration, gates and listening |

Use `page`=None, `db`=None, `host`="127.0.0.1", `port`=18081, `admin`=False, `admin_prefix`="/admin", `shell_css`=None, `layout`=None, `asset_version`=None as the inputs for this operation.

Low-level app handle for advanced mounts. Prefer web.serve for Artifact sites
that already follow file-partition layout. Optional page, db, host, port, admin
flags, shell_css, layout, and asset_version configure the runtime without a
configure junk drawer of unrelated kwargs. Authors who need only Document
discovery should stay on serve; agents should not invent HTML shells on this
object. Methods static, listen, and gate hang off the returned handle for
mounts that remain table- and Metadata-driven.

We **[Load the native plugin required by this operation](cap.load())**.
Return *[Low-level app handle for advanced mounts](web_app_new(page=`page`, db=`db`, admin=`admin`, host=`host`, port=`port`, admin_prefix=`admin_prefix`, login_redirect=None, logout_redirect=None, shell_css=`shell_css`, layout=`layout`, asset_version=`asset_version`))*.

## static

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| dir | text | required | filesystem directory |
| mount | text | "/static" | URL mount |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | app handle |

Use `dir`, `mount`="/static" as the inputs for this operation.

Mount static files from `dir` at `mount`=/static on this app. Use for bridge
scripts and assets referenced by Documents — not for dumping an HTML-only site
tree that bypasses Artifact Documents.

Return *[Mount static files from dir at mount=/static on this app](web_app_static(app=`self`, dir=`dir`, mount=`mount`))*.

## listen

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | listen result |

Start listening with the already configured app host and port. Prefer serve for
root scanning; listen is for advanced app handles after manual route setup.

Return *[Start listening with the already configured app host and port. Prefer serve for root scanning; listen is for advanced app handles after manual route setup.](web_listen(app=`self`))*.

## gate

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | required | gated path |
| roles | any | "admin" | roles supplied by the caller |
| permissions | text | "" | Comma-separated permission names held by the current role or subject |
| match | text | "prefix" | match mode |
| on_deny | text | "forbid" | deny behavior |
| exclude | any | None | exclusions |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | app handle |

Use `path`, `roles`="admin", `permissions`="", `match`="prefix", `on_deny`="forbid", `exclude`=None as the inputs for this operation.

Attach an authorization gate on `path` for this app. Defaults: `roles`="admin",

`permissions`="", `match`="prefix", `on_deny`="forbid", `exclude`=None . Auth
resources live in ext/security; this method only wires the gate onto the web
app surface.

Return *[Attach an authorization gate on path for this app](web_app_gate(app=`self`, path=`path`, roles=`roles`, permissions=`permissions`, match=`match`, on_deny=`on_deny`, exclude=`exclude`))*.
