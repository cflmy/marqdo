---
title: web ws broadcast + access_log smoke (offline)
import web:ext/web/web.mq.md
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
---

# main

**p = > plugin.native_path name="web"**
1. `p`
  > plugin.load path=`p`
2. *
  > sys.exit code=1

**pg = > web.page title="w6b"**
**app = > web.app page=`pg` port=18111**
**app = > `app`.route_ws path="/room" mode="broadcast"**
**routes = [ws_routes](app)**
**room = [/room](routes)**
**mode = [mode](room)**
1. `mode` == "broadcast"
  > print text=ws-broadcast-route-ok
2. *
  > print text=ws-broadcast-route-fail

**app = > `app`.configure access_log=True compress=False**
**mw = [middleware](app)**
**al = [access_log](mw)**
1. `al`
  > print text=access-log-ok
2. *
  > print text=access-log-fail
