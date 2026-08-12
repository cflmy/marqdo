---
title: web-site
description: Home + /about + /new (form in page slot) + admin.
> ext/web/web.mq.md
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

Main content is not a reusable component, so it is authored here.

`index` =

| 属性 | 值 | 样式 |
|------|-----|------|
| `title` | articles.`articles`.title | shell.`card_title` |
| `body` | articles.`articles`.body | shell.`card_body` |

Article create form (design §5.5): field table + rules table.

`article_fields` =

| 字段 | 标签 | 类型 | 必填 | 默认 |
|------|------|------|------|------|
| title | Title | text | true | |
| body | Body | textarea | false | |

`article_rules` =

| 字段 | 规则 | 消息 |
|------|------|------|
| title | required | Title is required |
| title | max:120 | Title is too long |
| body | max:8000 | Body is too long |

*`store` = > db.open *
*`page` = > web.page title="Marqdo Web Site" intro="<h1>Web Site</h1><p>Tables + class methods.</p>" *
*`page` = > `page`.compose_components components=`home` *
*`page` = > `page`.compose_main main=`index` *

About page: same shell, different intro (no main bind).

*`about` = > web.page title="About" intro="<h1>About</h1><p>Marqdo web = tables + class methods.</p>" *
*`about` = > `about`.compose_components components=`home` *

*`article_form` = > web.form table=articles action=insert *
*`article_form` = > `article_form`.fields fields=`article_fields` *
*`article_form` = > `article_form`.rules rules=`article_rules` *

New article: form embedded in the page main slot (design §5.5.4).

*`new` = > web.page title="New article" intro="<h1>New article</h1><p>Form lives in the main slot.</p>" *
*`new` = > `new`.compose_components components=`home` *
*`new` = > `new`.compose_form id=article form=`article_form` *

*`app` = > web.app page=`page` db=`store` admin=True host=127.0.0.1 port=18081 *
*`app` = > `app`.route path=/about page=`about` *
*`app` = > `app`.route path=/new page=`new` *
> `app`.listen
