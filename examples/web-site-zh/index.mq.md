---
title: 中文站点示例
description: Home + /about + /new（表单嵌入）+ 后台；只用 ext/web/网页.mq.md。
> ext/web/网页.mq.md
> styles/shell.mq.md
> components/nav.mq.md
> components/side.mq.md
> components/foot.mq.md
> db/articles.mq.md
> db/index.mq.md as db
---

# main

`home` =

| 组件 | 样式 |
|------|------|
| nav.`nav` | shell.`topnav` |
| side.`side` | shell.`side_panel` |
| foot.`foot` | |

`index` =

| 属性 | 值 | 样式 |
|------|-----|------|
| `title` | articles.`articles`.title | shell.`card_title` |
| `body` | articles.`articles`.body | shell.`card_body` |

`article_fields` =

| 字段 | 标签 | 类型 | 必填 | 默认 |
|------|------|------|------|------|
| title | 标题 | text | true | |
| body | 正文 | textarea | false | |

`article_rules` =

| 字段 | 规则 | 消息 |
|------|------|------|
| title | required | 标题不能为空 |
| title | max:120 | 标题太长 |
| body | max:8000 | 正文太长 |

*`store` = > db.open *
*`page` = > 网页.页面 标题="Marqdo 中文站" 引言="<h1>中文站</h1><p>表格 + 类方法。</p>" *
*`page` = > `page`.组件装配 组件=`home` *
*`page` = > `page`.主体装配 主体=`index` *

*`about` = > 网页.页面 标题="关于" 引言="<h1>关于</h1><p>导入网页.mq.md 即用中文 API。</p>" *
*`about` = > `about`.组件装配 组件=`home` *

*`article_form` = > 网页.表单 表=articles 动作=插入 *
*`article_form` = > `article_form`.字段 字段=`article_fields` *
*`article_form` = > `article_form`.规则 规则=`article_rules` *

*`new` = > 网页.页面 标题="写文章" 引言="<h1>写文章</h1><p>表单嵌在主区。</p>" *
*`new` = > `new`.组件装配 组件=`home` *
*`new` = > `new`.表单装配 id=article 表单=`article_form` *

*`app` = > 网页.应用 页面=`page` 数据库=`store` 后台=True 主机=127.0.0.1 端口=18082 *
*`app` = > `app`.路由 路径=/about 页面=`about` *
*`app` = > `app`.路由 路径=/new 页面=`new` *
*`app` = > `app`.静态 目录=public 挂载=/static *
> `app`.监听
