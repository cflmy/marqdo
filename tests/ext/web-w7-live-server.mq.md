---
title: web W7 live server — 404, redirect, sitemap, robots, cache-control
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

`items` =

| loc | note |
|-----|------|
| / | home |
| /about | about |

*pg = > web.page title="w7-live"*
*nf = > web.page title="Not Found Page"*
*app = > web.app page=`pg` port=18121*
*app = > `app`.configure cache_control="public, max-age=120"*
*app = > `app`.redirect from="/legacy" to="/" permanent=True*
*app = > `app`.error_page status=404 page=`nf`*
*app = > `app`.sitemap path="/sitemap.xml" base="http://127.0.0.1:18121" items=`items`*
*app = > `app`.robots sitemap="http://127.0.0.1:18121/sitemap.xml"*
> `app`.listen
