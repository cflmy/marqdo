---
title: ext/web smoke (offline)
description: Variable-name tables + page region methods.
> ext/web/web.mq.md
> lib/json.mq.md
> lib/sys.mq.md
---

# main

> web.ensure_plugin

`nav` =

| 前端变量 | 后端数据库 | 绑定css样式 |
|----------|------------|-------------|
| 首页 | / | |
| 文档 | /docs | |

*`links` = > web.as_links table=`nav` *
*`ln` = > len value=`links` *
1. `ln` == 2
  > print text=nav-ok
2. *
  > print text=nav-bad

- [`项`](`links`)
  *`名` = > json.get value=`项` key=label *
  > print text=`名`

*`binds` = > web.as_bind table=`nav` *
*`bn` = > len value=`binds` *
1. `bn` == 2
  > print text=bind-ok
2. *
  > print text=bind-bad

*`page` = > web.page title=Smoke *
*`page` = > `page`.nav table=`nav` *
*`html` = > `page`.render *
*`parts` = > split value=`html` sep=首页 *
*`n` = > len value=`parts` *
1. `n` > 1
  > print text=render-ok
2. *
  > print text=render-bad

*`dir` = > sys.env_get name=TMPDIR *
1. `dir`
  *`_` = 1*
2. *
  *`dir` = /tmp *
*`slash` = > json.parse text={"a":"/mq-web-gold-"} *
*`a` = > json.get value=`slash` key=a *
*`pid` = > sys.env_get name=USER *
*`dbpath` = `dir` + `a` + `pid` + ".db" *
*`url` = "sqlite:" + `dbpath` *
*`db` = > web.db url=`url` *

`articles` =

| 字段 | 类型 | 可空 |
|------|------|------|
| id | integer | false |
| title | text | false |

*`articles` = > `db`.init name=articles fields=`articles` *

`种子` =

| @ | title |
|---|-------|
| 1 | hello |

> `db`.insert table=`articles` rows=`种子`

`主体` =

| 前端变量 | 后端数据库 | 绑定css样式 |
|----------|------------|-------------|
| title | title | card-title |

*`page` = > web.page title=Smoke db_url=`url` *
*`page` = > `page`.nav table=`nav` *
*`page` = > `page`.main table=`articles` bind=`主体` intro="<h1>Smoke</h1>" *
*`html2` = > `page`.render *
*`parts2` = > split value=`html2` sep=hello *
*`n2` = > len value=`parts2` *
1. `n2` > 1
  > print text=live-ok
2. *
  > print text=live-bad

*`文章` = > `db`.follow table=`articles` *
*`page` = > web.page title=Smoke *
*`page` = > `page`.nav table=`nav` *
*`page` = > `page`.follow name=main live=`文章` *
*`html3` = > `page`.render *
*`parts3` = > split value=`html3` sep=hello *
*`n3` = > len value=`parts3` *
1. `n3` > 1
  > print text=follow-ok
2. *
  > print text=follow-bad
