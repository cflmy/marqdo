---
title: web demo
> ext/web/web.mq.md
---

# main

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

`foot` =

| 前端变量 | 后端数据库 | 绑定css样式 |
|----------|------------|-------------|
| Marqdo | https://github.com/cflmy/marqdo | |

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

`种子` =

| @ | title | body |
|---|-------|------|
| 1 | hello | first-row |
| 2 | second | card-body |

*`db` = > web.db url="sqlite:./data/web-demo.db" *
*`articles` = > `db`.init name=articles fields=`articles` *
> `db`.insert table=`articles` rows=`种子`

*`page` = > web.page title=Demo *
*`page` = > `page`.nav table=`nav` *
*`page` = > `page`.sidebar table=`side` *
*`page` = > `page`.footer table=`foot` *
*`page` = > `page`.main table=`articles` bind=`主体` layout=cards intro="<h1>Marqdo Web</h1><p>变量名即表名：articles；页面先创建再挂 nav / main。</p>" *

*`app` = > web.app page=`page` db=`db` admin=True host=127.0.0.1 port=18080 *
> `app`.static dir=ext/web/templates/starter/static
> print text=listening
> `app`.listen
