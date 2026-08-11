---
title: ext/web
description: >-
  Dynamic website toolkit. Variable name = table name; page regions bound by methods.
> lib/sys.mq.md
> lib/json.mq.md
> lib/plugin.mq.md
> lib/fs.mq.md
---

## load_env
    + `path`=None

**> sys.load_dotenv path=`path`**

## ensure_plugin

*`p` = > plugin.native_path name=web *
1. `p`
  > plugin.load path=`p`
2. *
  > print text=web-plugin-missing: cargo build -p marqdo_plugin_web or marqdo ext add web
  > sys.exit code=1

---

## as_links
    + `table`

**> web_as_links table=`table`**

## as_fields
    + `table`

**> web_as_fields table=`table`**

## as_bind
    + `table`

**> web_as_bind table=`table`**

## as_rows
    + `table`

**> web_as_rows table=`table`**

## table_name
    + `table`

*`t` = > type `table` *
1. `t` == map
  *`n` = > json.get value=`table` key=name *
  1. `n`
    **`n`**
  2. *
    **`table`**
2. *
  **`table`**

---

# page
    + `theme`=starter
    + `title`=Marqdo Web
    + `intro`="<p>Welcome</p>"
    + `layout`=cards
    + `db_url`=None

*`self` = > json.parse text={"_type":"page","theme":"starter","title":"Marqdo Web","intro":"<p>Welcome</p>","layout":"cards","slots":{},"nav":[],"sidebar":[],"footer":[]} *
*`self` = > json.set map=`self` key=theme value=`theme` *
*`self` = > json.set map=`self` key=title value=`title` *
*`self` = > json.set map=`self` key=intro value=`intro` *
*`self` = > json.set map=`self` key=layout value=`layout` *
1. `db_url`
  *`self` = > json.set map=`self` key=db_url value=`db_url` *
2. *
  *`_` = 1*
**`self`**

## nav
    + `table`

*`b` = > as_bind table=`table` *
*`self` = > json.set map=`self` key=nav value=`b` *
**`self`**

## sidebar
    + `table`

*`b` = > as_bind table=`table` *
*`self` = > json.set map=`self` key=sidebar value=`b` *
**`self`**

## footer
    + `table`

*`b` = > as_bind table=`table` *
*`self` = > json.set map=`self` key=footer value=`b` *
**`self`**

## main
    + `table`
    + `bind`
    + `layout`=cards
    + `intro`=""

*`tn` = > table_name table=`table` *
*`b` = > as_bind table=`bind` *
*`self` = > json.set map=`self` key=table value=`tn` *
*`self` = > json.set map=`self` key=main value=`b` *
*`self` = > json.set map=`self` key=layout value=`layout` *
1. `intro`
  *`self` = > json.set map=`self` key=intro value=`intro` *
2. *
  *`_` = 1*
**`self`**

## follow
    + `name`
    + `live`

*`slots` = > json.get value=`self` key=slots *
1. `slots`
  *`_` = 1*
2. *
  *`slots` = > json.parse text={} *
*`slots` = > json.set map=`slots` key=`name` value=`live` *
*`self` = > json.set map=`self` key=slots value=`slots` *
1. `name` == main
  *`self` = > json.set map=`self` key=main value=`live` *
2. `name` == 主体
  *`self` = > json.set map=`self` key=main value=`live` *
3. *
  *`_` = 1*
**`self`**

## resolve_live
    + `value`

*`t` = > type `value` *
1. `t` == map
  *`kind` = > json.get value=`value` key=_type *
  1. `kind` == live
    *`url` = > json.get value=`value` key=url *
    *`table` = > json.get value=`value` key=table *
    *`limit` = > json.get value=`value` key=limit *
    1. `limit`
      *`_` = 1*
    2. *
      *`limit` = 200*
    *`r` = > web_db_all url=`url` table=`table` limit=`limit` *
    **> json.get value=`r` key=rows**
  2. *
    **`value`**
2. *
  **`value`**

## render

*`title` = > json.get value=`self` key=title *
*`nav` = > json.get value=`self` key=nav *
*`sidebar` = > json.get value=`self` key=sidebar *
*`footer` = > json.get value=`self` key=footer *
*`intro` = > json.get value=`self` key=intro *
*`layout` = > json.get value=`self` key=layout *
*`db_url` = > json.get value=`self` key=db_url *
*`table` = > json.get value=`self` key=table *
*`main` = > json.get value=`self` key=main *
*`main` = > `self`.resolve_live value=`main` *
**> web_render title=`title` nav=`nav` sidebar=`sidebar` footer=`footer` main=`main` intro=`intro` layout=`layout` db_url=`db_url` table=`table`**

---

# db
    + `url`=None

*`u` = `url` *
1. `u`
  *`_` = 1*
2. *
  *`u` = > sys.env_get name=DATABASE_URL *
1. `u`
  *`_` = 1*
2. *
  *`u` = sqlite:./data/app.db *
*`self` = > json.parse text={"_type":"db","tables":[]} *
*`self` = > json.set map=`self` key=url value=`u` *
**`self`**

## init
    + `name`
    + `fields`
    + `primary`=id

*`url` = > json.get value=`self` key=url *
*`fs` = > as_fields table=`fields` *
*`_` = > web_db_define url=`url` table=`name` fields=`fs` primary=`primary` *
*`tables` = > json.get value=`self` key=tables *
*`tables` = > json.append list=`tables` item=`name` *
*`self` = > json.set map=`self` key=tables value=`tables` *
*`h` = > json.parse text={"_type":"db_table"} *
*`h` = > json.set map=`h` key=name value=`name` *
*`h` = > json.set map=`h` key=url value=`url` *
**`h`**

## define
    + `table`
    + `fields`
    + `primary`=id

*`tn` = > table_name table=`table` *
**> `self`.init name=`tn` fields=`fields` primary=`primary`**

## migrate
    + `dir`=migrations

*`url` = > json.get value=`self` key=url *
**> web_db_migrate url=`url` dir=`dir`**

## all
    + `table`
    + `limit`=200

*`url` = > json.get value=`self` key=url *
*`tn` = > table_name table=`table` *
*`r` = > web_db_all url=`url` table=`tn` limit=`limit` *
**> json.get value=`r` key=rows**

## follow
    + `table`
    + `limit`=200

*`url` = > json.get value=`self` key=url *
*`tn` = > table_name table=`table` *
*`live` = > json.parse text={"_type":"live","kind":"table"} *
*`live` = > json.set map=`live` key=url value=`url` *
*`live` = > json.set map=`live` key=table value=`tn` *
*`live` = > json.set map=`live` key=limit value=`limit` *
**`live`**

## insert
    + `table`
    + `rows`

*`url` = > json.get value=`self` key=url *
*`tn` = > table_name table=`table` *
*`rs` = > as_rows table=`rows` *
**> web_db_insert url=`url` table=`tn` rows=`rs`**

## query
    + `sql`
    + `args`=None

*`url` = > json.get value=`self` key=url *
**> web_db_query url=`url` sql=`sql` args=`args`**

---

# store

*`ep` = > sys.env_get name=MARQDO_S3_ENDPOINT *
*`self` = > json.parse text={"_type":"store","ok":false} *
1. `ep`
  *`self` = > json.set map=`self` key=ok value=True *
  *`self` = > json.set map=`self` key=endpoint value=`ep` *
2. *
  *`_` = 1*
**`self`**

---

# cache

*`u` = > sys.env_get name=REDIS_URL *
1. `u`
  *`_` = 1*
2. *
  *`u` = > sys.env_get name=MARQDO_REDIS_URL *
*`self` = > json.parse text={"_type":"cache","ok":false} *
1. `u`
  *`self` = > json.set map=`self` key=ok value=True *
  *`self` = > json.set map=`self` key=url value=`u` *
2. *
  *`_` = 1*
**`self`**

---

# app
    + `page`=None
    + `db`=None
    + `admin`=True
    + `host`=None
    + `port`=None

*`self` = > json.parse text={"_type":"app","static_dir":null,"admin":true} *
*`self` = > json.set map=`self` key=page value=`page` *
*`self` = > json.set map=`self` key=db value=`db` *
*`self` = > json.set map=`self` key=admin value=`admin` *
*`h` = `host` *
1. `h`
  *`_` = 1*
2. *
  *`h` = > sys.env_get name=MARQDO_WEB_HOST *
1. `h`
  *`_` = 1*
2. *
  *`h` = 127.0.0.1 *
*`p` = `port` *
1. `p`
  *`_` = 1*
2. *
  *`p` = > sys.env_get name=MARQDO_WEB_PORT *
1. `p`
  *`_` = 1*
2. *
  *`p` = 8080 *
*`self` = > json.set map=`self` key=host value=`h` *
*`self` = > json.set map=`self` key=port value=`p` *
**`self`**

## static
    + `dir`

*`self` = > json.set map=`self` key=static_dir value=`dir` *
**`self`**

## listen
    + `duration_ms`=None

*`page` = > json.get value=`self` key=page *
*`db` = > json.get value=`self` key=db *
*`db_url` = None*
*`tables` = > json.parse text=[] *
1. `db`
  *`db_url` = > json.get value=`db` key=url *
  *`tables` = > json.get value=`db` key=tables *
  *`page` = > json.set map=`page` key=db_url value=`db_url` *
2. *
  *`_` = 1*
*`host` = > json.get value=`self` key=host *
*`port` = > json.get value=`self` key=port *
*`sd` = > json.get value=`self` key=static_dir *
*`admin` = > json.get value=`self` key=admin *
**> web_listen host=`host` port=`port` page=`page` static_dir=`sd` db_url=`db_url` admin=`admin` admin_tables=`tables` duration_ms=`duration_ms`**

---

## scaffold
    + `dest`
    + `theme`=starter

*`sep` = > json.parse text={"css":"/static/theme.css","env":"/.env.example","mig":"/migrations/001_init.sql","idx":"/index.mq.md"} *
*`css_s` = > json.get value=`sep` key=css *
*`env_s` = > json.get value=`sep` key=env *
*`mig_s` = > json.get value=`sep` key=mig *
*`idx_s` = > json.get value=`sep` key=idx *
*`css_out` = `dest` + `css_s` *
*`env_out` = `dest` + `env_s` *
*`mig_out` = `dest` + `mig_s` *
*`idx_out` = `dest` + `idx_s` *

*`theme_src` = "ext/web/templates/starter/static/theme.css" *
*`css` = > fs.read_text path=`theme_src` *
> fs.write_text path=`css_out` text=`css`

*`env` = "MARQDO_WEB_HOST=127.0.0.1\nMARQDO_WEB_PORT=8080\nDATABASE_URL=sqlite:./data/app.db\n" *
> fs.write_text path=`env_out` text=`env`

*`mig` = "CREATE TABLE IF NOT EXISTS articles (\n  id INTEGER PRIMARY KEY AUTOINCREMENT,\n  title TEXT NOT NULL,\n  body TEXT,\n  created_at TEXT DEFAULT (datetime('now'))\n);\n" *
> fs.write_text path=`mig_out` text=`mig`

*`app` = > json.parse text="---\n> ext/web/web.mq.md\n---\n\n# main\n\n> web.load_env path=.env\n> web.ensure_plugin\n\n`nav` =\n\n| 前端变量 | 后端数据库 | 绑定css样式 |\n|----------|------------|-------------|\n| 首页 | / | |\n| 管理 | /admin | |\n\n`side` =\n\n| 前端变量 | 后端数据库 | 绑定css样式 |\n|----------|------------|-------------|\n| 全部文章 | / | |\n| 管理后台 | /admin | |\n\n`articles` =\n\n| 字段 | 类型 | 可空 |\n|------|------|------|\n| id | integer | false |\n| title | text | false |\n| body | text | true |\n\n`主体` =\n\n| 前端变量 | 后端数据库 | 绑定css样式 |\n|----------|------------|-------------|\n| title | title | card-title |\n| body | body | card-body |\n\n*`db` = > web.db *\n*`articles` = > `db`.init name=articles fields=`articles` *\n\n*`page` = > web.page title=Starter *\n*`page` = > `page`.nav table=`nav` *\n*`page` = > `page`.sidebar table=`side` *\n*`page` = > `page`.main table=`articles` bind=`主体` layout=cards intro=\"<h1>Starter</h1>\" *\n\n*`app` = > web.app page=`page` db=`db` admin=True *\n> `app`.static dir=./static\n> `app`.listen\n" *
> fs.write_text path=`idx_out` text=`app`
**`dest`**
