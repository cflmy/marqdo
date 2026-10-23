---
markup: occlusion
title: web static artifact smoke
import:
  web: ext/web/web.mq.md
---

# main

**pg = > web.page title="static"**
**app = > web.app page=`pg`**
**app = > `app`.static dir="web-fixtures/public" mount="/static"**
**d = [static_dir](access(app, "static_dir"))**
**m = [static_mount](access(app, "static_mount"))**
1. `d`
  > print text=dir-ok
2. *
  > print text=dir-fail
1. `m` == "/static"
  > print text=mount-ok
2. *
  > print text=mount-fail
