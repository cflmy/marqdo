---
title: web ws broadcast live server
import web:ext/web/web.mq.md
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
---

# main

*p = > plugin.native_path name="web"*
1. `p`
  > plugin.load path=`p`
2. *
  > sys.exit code=1

*pg = > web.page title="ws-live"*
*app = > web.app page=`pg` port=18112*
*app = > `app`.configure access_log=True*
*app = > `app`.route_ws path="/room" mode="broadcast"*
> `app`.listen
