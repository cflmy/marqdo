---
markup: occlusion
title: web.use middleware smoke
description: First-class middleware via web.use (replaces configure junk drawer).
import:
  web: ext/web/web.mq.md
---

# main

`headers` =

| header | value |
|--------|-------|
| X-Content-Type-Options | nosniff |
| X-Frame-Options | DENY |

**pg = > web.page title="use-smoke"**
**app = > web.app page=`pg`**
**app = > web.use app=`app` middleware=`headers`**
**mw = [middleware](access(app, "middleware"))**
**sec = [security](access(mw, "security"))**
1. `sec`
  > print text=use-ok
2. *
  > print text=use-fail
