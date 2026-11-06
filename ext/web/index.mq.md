---
markup: occlusion
title: ext/web
description: >-
  Web Artifact root skill (ADR 0007). Read this first. Layout is file-partitioned;
  tools run with marqdo call. No HTML/CSS injection as the default page method.
import:
  layout: ext/web/layout/index.mq.md
  page: ext/web/page.mq.md
  route: ext/web/route.mq.md
  component: ext/web/component.mq.md
  dom: ext/web/dom.mq.md
  client: ext/web/client.mq.md
  web: ext/web/web.mq.md
---

This file is the root skill for the Marqdo web package. If you are an AI agent
with no other Marqdo context, follow Choose a path, then Layout law, then How to
execute any tool. Do not invent MCP wrappers, raw HTML homepages, or a second
API. The Markdown ## headings are the tools. Author programs may still
`import web:ext/web/web.mq.md` for the facade; agents should start here.

## Choose a path

Pick one row. Do not mix English and Chinese tool names or trees.

| If you need | Do this |
|-------------|---------|
| Layout rules before writing files | Call `layout.rules` on `ext/web/layout/index.mq.md` (command below). |
| Preview HTML for a Document / View table | Call `web.render` via facade path or import `web` tools with `--effects plugin`. |
| Serve a site root | Author `serve.mq.md` calling `web.serve`, or call serve with `root=` · `port=`. |
| Chinese tool names | Open `ext/web/网页.mq.md` and stay in Chinese names. |

Agents use `##` tools with `marqdo call`. `# page` / `# route` / `# app` objects
are for author programs. Escape hatches `dom.raw` and `page.css` are not the
default path for building a homepage.

*None*

## Layout law

Web pages are built by file partition, not by injecting HTML. Split a homepage
into files for navigation, sidebar, main, and footer. If a region is still
complex (two-column footer, dense styles), split again until each file is
simple: one duty, one View table or short Markdown. Bind values, database
fields, and named styles through the unified View columns type, slot, value,
attrs, and style only. Forbidden as the default method: whole-page HTML
strings, CSS injection bags, restored compose fragments, and json.set DOM glue.
Escape hatch only: dom.raw / page.css, and only with a written reason.

Full text: doc/design/ext-web-layout-paradigm.md. In-package tools live at
ext/web/layout/index.mq.md.

Minimal commands to load the law (no plugin required):

Call example (plain text):
  marqdo call ext/web/layout/index.mq.md rules --json
  marqdo call ext/web/layout/index.mq.md view_schema --json

*None*

## How to execute any tool

Call example (plain text):
  marqdo call FILE FUNCTION name=value name=value --effects plugin --json

FILE is under `ext/web/`, usually `index.mq.md` or a module path. FUNCTION is the
exact ## heading. Nested ### use `parent.child`. Pass `--effects plugin` for
anything that loads the native web plugin (`render`, `serve`, `inspect`, …).
Layout helpers `rules` / `view_schema` / `accept_component` are pure and may omit
plugin effects. Use `--session` / `--toget` / `$name` to chain handles. Do not
call `ensure_plugin` or `host_*`; facades load the plugin.

Verify layout skill (no plugin):

Call example (plain text):
  marqdo call ext/web/layout/index.mq.md rules --json

*None*

## Package layout

| Path | Role |
|------|------|
| `ext/web/index.mq.md` | This root skill |
| `ext/web/layout/index.mq.md` | File-partition layout engineering tools |
| `ext/web/web.mq.md` | Author facade: page · route · serve · render · inspect · use |
| `ext/web/page.mq.md` | Document handle · render |
| `ext/web/route.mq.md` | Declared routes |
| `ext/web/component.mq.md` | View fragments · merge |
| `ext/web/dom.mq.md` | Markdown DOM; raw is escape hatch |
| `ext/web/client.mq.md` | WASM embed · patches |
| `ext/web/_capability.mq.md` | Plugin load only |
| `ext/web/网页.mq.md` | Chinese facade / skill mirror |

Related packages (not this facade): `ext/data`, `ext/security`, `ext/net`.

*None*

## layout module

File path: `ext/web/layout/index.mq.md`

Call example (plain text):
  marqdo call ext/web/layout/index.mq.md rules --json
  marqdo call ext/web/layout/index.mq.md accept_component name="nav" duty="top links only" --json

Main ## tools: rules, view_schema, merge_nodes, accept_component.

*None*

## facade module

File path: `ext/web/web.mq.md`

Author import: `web: ext/web/web.mq.md`. Agent call examples:

Call example (plain text):
  marqdo call ext/web/web.mq.md inspect --effects plugin --json

Main ## tools: page, route, render, use, inspect, serve, client_embed, …

*None*

## layout_rules

Root convenience: same payload as `layout.rules` so agents who only opened this
file can still fetch the law.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | layout rule map |

*> layout.rules*
