---
title: web collab bag smoke
description: route_ws room+presence for collab doc
import web:ext/web/web.mq.md
import json:lib/json.mq.md
import sys:lib/sys.mq.md
---

# main

**首页 = > web.page title="c"**
**应用 = > web.app page=`首页` admin=False**
**应用 = > `应用`.route_ws path="/live/{id}" mode="room" room_key="doc.{id}" presence=True**
**routes = > json.get value=`应用` key="ws_routes"**
**spec = > json.get value=`routes` key="/live/{id}"**
**mode = > json.get value=`spec` key="mode"**
**pr = > json.get value=`spec` key="presence"**
1. `mode` == "room"
  1. `pr` == True
    > print text=collab-ws-bag-ok
  2. *
    > print text=presence-fail
    > sys.exit code=1
2. *
  > print text=mode-fail
  > sys.exit code=1
