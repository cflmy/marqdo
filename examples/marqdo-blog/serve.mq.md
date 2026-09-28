---
title: marqdo-blog serve
description: Scan Artifact documents/endpoints and listen (ADR 0007).
import web:ext/web/web.mq.md
import posts:db/posts.mq.md
---

# main

**store = > posts.open url="sqlite:data/blog.db"**
> web.serve root="." host="127.0.0.1" port=18081 db=`store` static_dir="public"
