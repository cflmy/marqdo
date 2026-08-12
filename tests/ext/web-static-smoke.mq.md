---
title: web static smoke
description: Offline app.static registration (no listen).
> ext/web/web.mq.md
> lib/json.mq.md
---

# main

*`home` = > web.page title="Home" *
*`app` = > web.app page=`home` host=127.0.0.1 port=18081 *
*`app` = > `app`.static dir=web-fixtures/public mount=/static *

*`dir` = > json.get value=`app` key=static_dir *
1. `dir` == "web-fixtures/public"
  > print text=dir-ok
2. *
  > print text=dir-fail

*`mount` = > json.get value=`app` key=static_mount *
1. `mount` == "/static"
  > print text=mount-ok
2. *
  > print text=mount-fail

*`app2` = > `app`.static dir=assets mount=/assets *
*`m2` = > json.get value=`app2` key=static_mount *
1. `m2` == "/assets"
  > print text=custom-ok
2. *
  > print text=custom-fail
