---
title: web-site
description: Home = page table; main inline; class methods assemble.
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

*`store` = > db.open *
*`page` = > web.page title="Marqdo Web Site" intro="<h1>Web Site</h1><p>Tables + class methods.</p>" *
*`page` = > `page`.compose_components components=`home` *
*`page` = > `page`.compose_main main=`index` *
*`app` = > web.app page=`page` db=`store` admin=True host=127.0.0.1 port=18081 *
> `app`.listen
