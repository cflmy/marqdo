---
title: ext/web/web
description: Official web site classes (English). Tables + methods; no bag glue.
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
import json:lib/json.mq.md
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

## compose_form
    + `form`
    + `id`

Embed a form into the page main slot. `listen` auto-registers `GET|POST /_form/{id}` from page/route forms (optional `app.mount_form`).

**> web_compose_form page=`self` form=`form` id=`id`**

## render
    + `db`=None

Offline HTML for tests / previews.

1. `db`
  *url = > json.get value=`db` key="url"*
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

*url = > json.get value=`self` key="url"*
**> web_db_init url=`url` name=`name` fields=`fields`**

## insert
    + `table`
    + `rows`

*url = > json.get value=`self` key="url"*
**> web_db_insert url=`url` table=`table` rows=`rows`**

## select
    + `table`
    + `where`=None
    + `limit`=200

Simple filters: one-row map of column→value (AND `=`), or rows `|字段|操作|值|` (`=` `!=` `>` `>=` `<` `<=` `like`).

*url = > json.get value=`self` key="url"*
*r = > web_db_select url=`url` table=`table` where=`where` limit=`limit`*
**> json.get value=`r` key="rows"**

## get
    + `table`
    + `id`

*url = > json.get value=`self` key="url"*
**> web_db_get url=`url` table=`table` id=`id`**

## update
    + `table`
    + `id`
    + `row`

*url = > json.get value=`self` key="url"*
**> web_db_update url=`url` table=`table` id=`id` row=`row`**

## delete
    + `table`
    + `id`

*url = > json.get value=`self` key="url"*
**> web_db_delete url=`url` table=`table` id=`id`**

## exec
    + `sql`
    + `args`=None

*url = > json.get value=`self` key="url"*
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

*url = > json.get value=`db` key="url"*
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
