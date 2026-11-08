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

| 参数 | 类型 | 说明 |
|------|------|------|
| title | text | document title |
| route | text | path |
| method | text | HTTP method |
| body | text | Markdown body |
| nodes | any | View table or None |
| data_source | text | table name |
| data_order | text | order key |
| data_where | text | filters |

Construct a Document handle with `title`="", `route`="/", `method`="GET",
`body`="", optional View `nodes`=None, and flat `data_source`="", `data_order`="",
`data_where`="" keys. Prefer discovering type: web Artifact files via serve
rather than hand-building every page in memory. nodes must be the unified View
schema, never a private dialect or an HTML string shell.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | page handle |

*> page_mod.page title=`title` route=`route` method=`method` body=`body` nodes=`nodes` data_source=`data_source` data_order=`data_order` data_where=`data_where`*

## route

| 参数 | 类型 | 说明 |
|------|------|------|
| app | any | app handle |
| path | text | URL path |
| method | text | HTTP method |
| page | any | Document or None |
| fn | text | endpoint member |
| request | text | request kind |
| response | text | response kind |
| auth | text | auth tag |
| class | text | route class |

Register a declared Document page or Endpoint on `app` at `path` with
`method`="GET" . `page`=None holds a Document; `fn`="" names an Endpoint
lib.member. `request`="json", `response`="json", `auth`="", and `class`=""
document the contract. Optional class values include execute, short, read,
control. User routes must be declared — never implicit. After building the
route object this tool registers it on the app.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | registration result |

**r = > route_mod.route app=`app` path=`path` method=`method` page=`page` fn=`fn` request=`request` response=`response` auth=`auth` class=`class`**
*> `r`.register app=`app`*

## render

| 参数 | 类型 | 说明 |
|------|------|------|
| doc | any | Document or None |
| nodes | any | View table or None |
| body | text | Markdown |
| title | text | title |
| db | any | optional db |

Render a Document or View nodes to HTML for offline preview. Prefer nodes from
file-partitioned components. Optional `doc`=None is a Document handle; when set,
delegates to page.render. Optional `nodes`=None is a View table; when set without
doc, renders that table. Otherwise renders Markdown `body`="" with optional
`title`="" and `db`=None . Requires the native plugin. Do not treat this as
permission to paste a full HTML homepage — keep using View tables and Markdown.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | HTML or payload |

> cap.load
1. `doc`
  *> `doc`.render nodes=`nodes` db=`db`*
2. `nodes`
  *> web_render_nodes page=`doc` nodes=`nodes` db=`db` title=`title`*
3. *
  *> web_render_document page=`doc` body=`body` db=`db` title=`title`*

## use

| 参数 | 类型 | 说明 |
|------|------|------|
| app | any | app handle |
| middleware | any | middleware table |

Attach first-class `middleware` to `app`. Pass a GFM table (security headers,
cors row, and similar). This replaces the old configure(...) junk drawer with an
explicit table authors can read.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | app handle |

> cap.load
*> web_app_use app=`app` middleware=`middleware`*

## inspect

| 参数 | 类型 | 说明 |
|------|------|------|
| app | any | app or None |
| root | text | Site directory to inspect without listening |

Return declared versus system route tables for introspection. System routes such
as /_part and /_form must appear here. Use this to verify a site after serve or
manual registration. Optional `app`=None selects which application to inspect.

Optional `root`="" scans a site directory when supplied; it does not start a server.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | {declared, system} |

> cap.load
*> web_inspect app=`app` root=`root`*

## serve

| 参数 | 类型 | 说明 |
|------|------|------|
| root | text | site root |
| host | text | bind host |
| port | number | port |
| db | any | optional db |
| static_dir | text | static files |
| middleware | any | optional middleware |

Scan `root`="." for type web or endpoint Artifacts, register declared routes,
attach system routes, and listen on `host`="127.0.0.1" and `port`=18081 . The
site directory should already follow the layout law (components split until
simple). Optional `db`=None, `static_dir`="", and `middleware`=None bind
resources without stuffing them into this facade as author APIs.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | serve result |

> cap.load
*> web_serve_root root=`root` host=`host` port=`port` db=`db` static_dir=`static_dir` middleware=`middleware`*

## client_embed

| 参数 | 类型 | 说明 |
|------|------|------|
| bridge | text | bridge script URL |
| wasm | text | wasm URL |
| source | text | optional source |
| boot | bool | auto boot |

Describe WASM bridge embed tags for a page that needs client Marqdo. Defaults:
`bridge`="/static/marqdo-bridge.js", `wasm`="/static/marqdo_wasm.wasm",
`source`="", `boot`=True . This is not a substitute for Document layout: still
partition nav and main in files, then embed the bridge where interaction needs
it.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | embed descriptor |

*> client_mod.embed bridge=`bridge` wasm=`wasm` source=`source` boot=`boot`*

## text_patch

| 参数 | 类型 | 说明 |
|------|------|------|
| sel | text | CSS selector |
| text | text | new text |

Client patch: set text content for CSS selector `sel` to string `text`. Use for
small interactive updates after a Document is already laid out with View tables —
not for building the initial homepage shell.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | patch descriptor |

*> client_mod.text_patch sel=`sel` text=`text`*

## dom_patch

| 参数 | 类型 | 说明 |
|------|------|------|
| sel | text | CSS selector |
| html | text | optional HTML fragment |
| attrs | any | optional attrs |
| remove | bool | remove node |

Client patch for selector `sel` with optional `html`="", `attrs`=None, or
`remove`=False . Treat html here as a surgical update, never as the primary way
to author nav, sidebar, or footer. Layout law still applies to the Document that
hosts the selector.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | patch descriptor |

*> client_mod.dom_patch sel=`sel` html=`html` attrs=`attrs` remove=`remove`*

## make_style

| 参数 | 类型 | 说明 |
|------|------|------|
| name | text | style name |
| table | any | property table |
| strict | bool | strict mode |

Compile a named style `table` into CSS. Optional `name`="" and `strict`=False .
Styles remain data next to content; this is not a hidden page DSL. Prefer
referencing style names from View style columns after file-partitioned
components define their look.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | CSS / style handle |

> cap.load
*> web_style name=`name` table=`table` strict=`strict`*

## make_images

| 参数 | 类型 | 说明 |
|------|------|------|
| table | any | images table |

Build image descriptors from a readable `table` of sources and alts. Keep media
declarations in Markdown tables beside the Document that uses them.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | images payload |

> cap.load
*> web_images table=`table`*

## make_head

| 参数 | 类型 | 说明 |
|------|------|------|
| table | any | head table |

Build head resource rows (link, script, icon) from a `table`. Prefer declaring
head needs as data rather than injecting opaque HTML head strings.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | head payload |

> cap.load
*> web_head table=`table`*

# app

| 参数 | 类型 | 说明 |
|------|------|------|
| page | any | See prose default / 见叙述默认值 |
| db | any | See prose default / 见叙述默认值 |
| host | any | See prose default / 见叙述默认值 |
| port | any | See prose default / 见叙述默认值 |
| admin | any | See prose default / 见叙述默认值 |
| admin_prefix | any | See prose default / 见叙述默认值 |
| shell_css | any | See prose default / 见叙述默认值 |
| layout | any | See prose default / 见叙述默认值 |
| asset_version | any | See prose default / 见叙述默认值 |

Inputs: `page`=None, `db`=None, `host`=127.0.0.1, `port`=18081, `admin`=False, `admin_prefix`="/admin", `shell_css`=None, `layout`=None, `asset_version`=None

Low-level app handle for advanced mounts. Prefer web.serve for Artifact sites
that already follow file-partition layout. Optional page, db, host, port, admin
flags, shell_css, layout, and asset_version configure the runtime without a
configure junk drawer of unrelated kwargs. Authors who need only Document
discovery should stay on serve; agents should not invent HTML shells on this
object. Methods static, listen, and gate hang off the returned handle for
mounts that remain table- and Metadata-driven.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Constructed value / 构造结果 |

> cap.load
*> web_app_new page=`page` db=`db` admin=`admin` host=`host` port=`port` admin_prefix=`admin_prefix` login_redirect=None logout_redirect=None shell_css=`shell_css` layout=`layout` asset_version=`asset_version`*

## static

| 参数 | 类型 | 说明 |
|------|------|------|
| dir | text | filesystem directory |
| mount | text | URL mount |

Mount static files from `dir` at `mount`=/static on this app. Use for bridge
scripts and assets referenced by Documents — not for dumping an HTML-only site
tree that bypasses Artifact Documents.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | app handle |

*> web_app_static app=`self` dir=`dir` mount=`mount`*

## listen

Start listening with the already configured app host and port. Prefer serve for
root scanning; listen is for advanced app handles after manual route setup.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | listen result |

*> web_listen app=`self`*

## gate

| 参数 | 类型 | 说明 |
|------|------|------|
| path | text | gated path |
| roles | any | roles |
| permissions | text | permissions |
| match | text | match mode |
| on_deny | text | deny behavior |
| exclude | any | exclusions |

Attach an authorization gate on `path` for this app. Defaults: `roles`="admin",
`permissions`="", `match`="prefix", `on_deny`="forbid", `exclude`=None . Auth
resources live in ext/security; this method only wires the gate onto the web
app surface.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | app handle |

*> web_app_gate app=`self` path=`path` roles=`roles` permissions=`permissions` match=`match` on_deny=`on_deny` exclude=`exclude`*
