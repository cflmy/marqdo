---
title: web zh API smoke
description: Offline Chinese 网页.mq.md assemble (no listen).
> ext/web/网页.mq.md
> web-fixtures/components/nav.mq.md
> web-fixtures/components/side.mq.md
> web-fixtures/components/foot.mq.md
> web-fixtures/db/articles.mq.md as articles
> lib/json.mq.md
---

# main

*`store` = > 网页.数据库 地址="sqlite:web-fixtures/data/zh-smoke.db" *
*`schema` = > articles.schema *
> `store`.初始化 名=articles 字段=`schema`
> `store`.执行 sql="DELETE FROM articles"
> `store`.执行 sql="DELETE FROM sqlite_sequence WHERE name='articles'"

`layout` =

| 组件 | 样式 |
|------|------|
| nav.`nav` | |
| side.`side` | |
| foot.`foot` | |

`index` =

| 属性 | 值 | 样式 |
|------|-----|------|
| `title` | articles.`articles`.title | |
| `body` | articles.`articles`.body | |

`seed` =

| title | body |
|-------|------|
| 中文种子 | 正文 |

> `store`.插入 表=articles 行=`seed`

*`page` = > 网页.页面 标题="中文烟测" 引言="<h1>中文</h1>" *
*`page` = > `page`.组件装配 组件=`layout` *
*`page` = > `page`.主体装配 主体=`index` *

*`html` = > `page`.渲染 数据库=`store` *
1. `html`
  > print text=render-ok
2. *
  > print text=render-fail

*`nav` = > json.get value=`page` key=nav *
1. `nav`
  > print text=compose-ok
2. *
  > print text=compose-fail

`fields` =

| 字段 | 标签 | 类型 | 必填 | 默认 |
|------|------|------|------|------|
| title | 标题 | text | true | |

*`f` = > 网页.表单 表=articles 动作=插入 *
*`f` = > `f`.字段 字段=`fields` *
*`page` = > `page`.表单装配 id=article 表单=`f` *
*`fid` = > json.get value=`page` key=form_id *
1. `fid` == "article"
  > print text=form-ok
2. *
  > print text=form-fail

*`app` = > 网页.应用 页面=`page` 数据库=`store` 主机=127.0.0.1 端口=18082 *
*`about` = > 网页.页面 标题="关于" 引言="<p>关于</p>" *
*`app` = > `app`.路由 路径=/about 页面=`about` *
*`routes` = > json.get value=`app` key=routes *
*`routed` = > json.get value=`routes` key="/about" *
1. `routed`
  > print text=route-ok
2. *
  > print text=route-fail
