---
title: web route smoke
description: Offline app.route registration (no listen).
import web:ext/web/web.mq.md
import json:lib/json.mq.md
---

# main

*home = > web.page title="Home" intro="<h1>Home</h1>"*
*about = > web.page title="About" intro="<h1>About</h1>"*
*app = > web.app page=`home` host=127.0.0.1 port=18081*
*app = > `app`.route path=/about page=`about`*
*app = > `app`.route path=docs page=`about`*

*routes = > json.get value=`app` key=routes*
*about_page = > json.get value=`routes` key="/about"*
*docs_page = > json.get value=`routes` key="/docs"*

1. `about_page`
  > print text=about-ok
2. *
  > print text=about-fail

1. `docs_page`
  > print text=docs-ok
2. *
  > print text=docs-fail

*html = > `about`.render*
1. `html`
  > print text=render-ok
2. *
  > print text=render-fail
