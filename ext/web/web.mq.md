---
title: ext/web/web
description: >-
  Web Artifact facade (ADR 0007) — page · route · serve · render · inspect · use.
  Document/Endpoint first-class. No compose_* / configure / author ensure_plugin.
import cap:ext/web/_capability.mq.md
import page_mod:ext/web/page.mq.md
import route_mod:ext/web/route.mq.md
import dom_mod:ext/web/dom.mq.md
import client_mod:ext/web/client.mq.md
import table:lib/table.mq.md
---

Importing this module is the web **capability** (native plugin loads on first facade call).

## page
    + `title`=""
    + `route`="/"
    + `method`="GET"
    + `body`=""
    + `nodes`=None
    + `data_source`=""
    + `data_order`=""
    + `data_where`=""

Construct a Document handle. Prefer `type: web` Artifact files discovered by `serve`.

*> page_mod.page title=`title` route=`route` method=`method` body=`body` nodes=`nodes` data_source=`data_source` data_order=`data_order` data_where=`data_where`*

## route
    + `app`
    + `path`
    + `method`="GET"
    + `page`=None
    + `fn`=""
    + `request`="json"
    + `response`="json"
    + `auth`=""

Register a declared Document page or Endpoint (`fn` = `lib.member`).

**r = > route_mod.route app=`app` path=`path` method=`method` page=`page` fn=`fn` request=`request` response=`response` auth=`auth`**
*> `r`.register app=`app`*

## render
    + `doc`=None
    + `nodes`=None
    + `body`=""
    + `title`=""
    + `db`=None

Render a Document or View nodes to HTML (offline / preview).

> cap.load
1. `doc`
  *> `doc`.render nodes=`nodes` db=`db`*
2. `nodes`
  *> web_render_nodes page=`doc` nodes=`nodes` db=`db` title=`title`*
3. *
  *> web_render_document page=`doc` body=`body` db=`db` title=`title`*

## use
    + `app`
    + `middleware`

First-class middleware — pass a GFM table (security headers, cors row, etc.). Replaces `configure(...)` junk drawer.

> cap.load
*> web_app_use app=`app` middleware=`middleware`*

## inspect
    + `app`=None

Return `{declared, system}` route tables for introspection (code-as-documentation).

> cap.load
*> web_inspect app=`app`*

## serve
    + `root`="."
    + `host`="127.0.0.1"
    + `port`=18081
    + `db`=None
    + `static_dir`=""
    + `middleware`=None

Scan `root/**/*.mq.md` for `type: web|endpoint` (ZH `类型: 网页|端点`), register declared routes, attach system routes, listen.

> cap.load
*> web_serve_root root=`root` host=`host` port=`port` db=`db` static_dir=`static_dir` middleware=`middleware`*

## client_embed
    + `bridge`="/static/marqdo-bridge.js"
    + `wasm`="/static/marqdo_wasm.wasm"
    + `source`=""
    + `boot`=True

*> client_mod.embed bridge=`bridge` wasm=`wasm` source=`source` boot=`boot`*

## text_patch
    + `sel`
    + `text`

*> client_mod.text_patch sel=`sel` text=`text`*

## dom_patch
    + `sel`
    + `html`=""
    + `attrs`=None
    + `remove`=False

*> client_mod.dom_patch sel=`sel` html=`html` attrs=`attrs` remove=`remove`*

## make_style
    + `name`=""
    + `table`
    + `strict`=False

Style table → CSS (data, not hidden page DSL).

> cap.load
*> web_style name=`name` table=`table` strict=`strict`*

## make_images
    + `table`

> cap.load
*> web_images table=`table`*

## make_head
    + `table`

> cap.load
*> web_head table=`table`*

# app
    + `page`=None
    + `db`=None
    + `host`=127.0.0.1
    + `port`=18081
    + `admin`=False
    + `admin_prefix`=/admin
    + `shell_css`=None
    + `layout`=None
    + `asset_version`=None

Low-level app handle for advanced mounts. Prefer `web.serve` for Artifact sites.

> cap.load
*> web_app_new page=`page` db=`db` admin=`admin` host=`host` port=`port` admin_prefix=`admin_prefix` login_redirect=None logout_redirect=None shell_css=`shell_css` layout=`layout` asset_version=`asset_version`*

## static
    + `dir`
    + `mount`=/static

*> web_app_static app=`self` dir=`dir` mount=`mount`*

## listen

*> web_listen app=`self`*

## gate
    + `path`
    + `roles`=admin
    + `permissions`=""
    + `match`=prefix
    + `on_deny`=forbid
    + `exclude`=None

*> web_app_gate app=`self` path=`path` roles=`roles` permissions=`permissions` match=`match` on_deny=`on_deny` exclude=`exclude`*
