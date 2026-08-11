---
title: ext/web/web
description: Official web site classes (English). Tables + methods; no bag glue.
> lib/plugin.mq.md
> lib/sys.mq.md
> lib/json.mq.md
---

## ensure_plugin

Load the ABI v2 `web` plugin once.

*`p` = > plugin.native_path name=web *
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

## render
    + `db`=None

Offline HTML for tests / previews.

1. `db`
  *`url` = > json.get value=`db` key=url *
  **> web_render page=`self` url=`url`**
2. *
  **> web_render page=`self`**

# style
    + `name`=""

Style tables live as `##` exports in site style modules; compose resolves them by path.

**`self`**

## process
    + `style`=None
    + `name`=None
    + `path`=None

**`self`**

# db
    + `url`="sqlite:site.db"

> ensure_plugin
**> web_db_new url=`url`**

## init
    + `name`
    + `fields`

*`url` = > json.get value=`self` key=url *
**> web_db_init url=`url` name=`name` fields=`fields`**

## insert
    + `table`
    + `rows`

*`url` = > json.get value=`self` key=url *
**> web_db_insert url=`url` table=`table` rows=`rows`**

## select
    + `table`
    + `where`=None
    + `limit`=200

*`url` = > json.get value=`self` key=url *
*`r` = > web_db_select url=`url` table=`table` limit=`limit` *
**> json.get value=`r` key=rows**

## get
    + `table`
    + `id`

*`url` = > json.get value=`self` key=url *
**> web_db_get url=`url` table=`table` id=`id`**

## update
    + `table`
    + `id`
    + `row`

*`url` = > json.get value=`self` key=url *
**> web_db_update url=`url` table=`table` id=`id` row=`row`**

## delete
    + `table`
    + `id`

*`url` = > json.get value=`self` key=url *
**> web_db_delete url=`url` table=`table` id=`id`**

## exec
    + `sql`
    + `args`=None

*`url` = > json.get value=`self` key=url *
**> web_db_exec url=`url` sql=`sql` args=`args`**

# form
    + `table`=None
    + `action`=insert
    + `id`=None

Form API is planned (design §5.5); first trial ships the class shell only.

**`self`**

## fields
    + `fields`

**`self`**

## validate
    + `rules`=None
    + `data`

**`data`**

## render

**""**

## submit
    + `data`
    + `db`

**`data`**

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
    + `page`=None

**`self`**

## listen

Serve `/`, `/_part/{id}`, and optional `/admin` from the assembled page.

**> web_listen app=`self`**
