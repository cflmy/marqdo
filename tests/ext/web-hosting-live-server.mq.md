---
title: web hosting live server
description: Listen with proxy (→ mock upstream) and invoke routes for gold.rs.
import web:ext/web/web.mq.md
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
import demo:web-hosting-helpers.mq.md
---

# main

*p = > plugin.native_path name="web"*
1. `p`
  > plugin.load path=`p`
2. *
  > sys.exit code=1

*pg = > web.page title="hosting-live"*
*app = > web.app page=`pg` host=127.0.0.1 port=18141*
*app = > `app`.proxy path="/proxy/sse" upstream="http://127.0.0.1:18142/sse" stream=True methods="POST"*
*app = > `app`.invoke path="/api/echo" fn="demo.echo" method="POST" body="json"*
> `app`.listen
