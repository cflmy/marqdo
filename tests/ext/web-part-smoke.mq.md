---
title: web part smoke
description: Routed pages stamp _route and emit {path}/_part/{id} slot sources.
import web:ext/web/web.mq.md
import nav:web-fixtures/components/nav.mq.md
import side:web-fixtures/components/side.mq.md
import foot:web-fixtures/components/foot.mq.md
import json:lib/json.mq.md
---

# main

`layout` =

| 组件 | 样式 |
|------|------|
| nav.`nav` | |
| side.`side` | |
| foot.`foot` | |

*home = > web.page title="Home" intro="<h1>Home</h1>"*
*home = > `home`.compose_components components=`layout`*

*about = > web.page title="About" intro="<h1>About</h1>"*
*about = > `about`.compose_components components=`layout`*

*app = > web.app page=`home` host=127.0.0.1 port=18081*
*app = > `app`.route path="/about" page=`about`*

*routes = > json.get value=`app` key="routes"*
*routed = > json.get value=`routes` key="/about"*
*route = > json.get value=`routed` key="_route"*
1. `route` == "/about"
  > print text=route-stamp-ok
2. *
  > print text=route-stamp-fail

*home_html = > `home`.render*
*home_chunks = > split value=`home_html` sep="/_part/"*
*hn = > len value=`home_chunks`*
1. `hn` >= 2
  > print text=home-part-ok
2. *
  > print text=home-part-fail

*about_html = > `routed`.render*
*about_chunks = > split value=`about_html` sep="/about/_part/"*
*an = > len value=`about_chunks`*
1. `an` >= 2
  > print text=route-part-ok
2. *
  > print text=route-part-fail
