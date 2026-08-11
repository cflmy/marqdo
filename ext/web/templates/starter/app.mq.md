---
title: ext/web starter app
description: Variable name = table name; page methods bind regions.
> ext/web/web.mq.md
---

# main

> web.load_env path=.env
> web.ensure_plugin

`nav` =

| 前端变量 | 后端数据库 | 绑定css样式 |
|----------|------------|-------------|
| 首页 | / | |
| 管理 | /admin | |

`side` =

| 前端变量 | 后端数据库 | 绑定css样式 |
|----------|------------|-------------|
| 全部文章 | / | |
| 管理后台 | /admin | |

`articles` =

| 字段 | 类型 | 可空 |
|------|------|------|
| id | integer | false |
| title | text | false |
| body | text | true |

`主体` =

| 前端变量 | 后端数据库 | 绑定css样式 |
|----------|------------|-------------|
| title | title | card-title |
| body | body | card-body |

*`db` = > web.db *
*`articles` = > `db`.init name=articles fields=`articles` *

*`page` = > web.page title=Starter *
*`page` = > `page`.nav table=`nav` *
*`page` = > `page`.sidebar table=`side` *
*`page` = > `page`.main table=`articles` bind=`主体` layout=cards intro="<h1>Starter</h1>" *

*`app` = > web.app page=`page` db=`db` admin=True *
> `app`.static dir=./static
> `app`.listen
