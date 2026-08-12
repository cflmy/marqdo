---
title: web assemble smoke
description: Offline assemble + render markers (no listen).
> ext/web/web.mq.md
> web-fixtures/styles/shell.mq.md as shell
> web-fixtures/components/nav.mq.md as nav
> web-fixtures/components/side.mq.md as side
> web-fixtures/components/foot.mq.md as foot
> web-fixtures/db/articles.mq.md as articles
> lib/fs.mq.md
> lib/json.mq.md
---

# main

*`store` = > web.db url="sqlite:web-fixtures/data/smoke.db" *
*`fields` = > articles.schema *
> `store`.init name=articles fields=`fields`
*`rows` = > `store`.select table=articles limit=1 *
1. `rows`
  *`_` = 1*
2. *
  *`seed` = > articles.seed *
  > `store`.insert table=articles rows=`seed`

`home` =

| 组件 | 样式 |
|------|------|
| nav.`nav` | shell.`topnav` |
| side.`side` | shell.`side_panel` |
| foot.`foot` | |

`index` =

| 属性 | 值 | 样式 |
|------|-----|------|
| title | articles.`articles`.title | shell.`card_title` |
| body | articles.`articles`.body | shell.`card_body` |

*`page` = > web.page title="smoke" intro="<h1>smoke</h1>" *
*`page` = > `page`.compose_components components=`home` *
*`page` = > `page`.compose_main main=`index` *
*`html` = > `page`.render db=`store` *

1. `html`
  > print text=render-ok
2. *
  > print text=render-fail

*`nav` = > json.get value=`page` key=nav *
1. `nav`
  > print text=compose-nav-ok
2. *
  > print text=compose-nav-fail

*`side` = > json.get value=`page` key=sidebar *
1. `side`
  > print text=compose-side-ok
2. *
  > print text=compose-side-fail

*`css` = > json.get value=`page` key=styles_css *
1. `css`
  > print text=styles-ok
2. *
  > print text=styles-fail

*`parts` = > json.get value=`page` key=parts *
*`index_part` = > json.get value=`parts` key=index *
1. `index_part`
  > print text=compose-ok
2. *
  > print text=compose-fail

*`got` = > `store`.get table=articles id=1 *
*`title` = > json.get value=`got` key=title *
1. `title` == "Hello Marqdo"
  > print text=db-ok
2. *
  > print text=db-fail
