---
title: web-client-site serve
import web:ext/web/web.mq.md
---

# main

**`embed` = > web.client_embed source="/static/client.mq.md"**
> print text=embed-ready
> web.serve root="." host="127.0.0.1" port=18090 static_dir="static"
