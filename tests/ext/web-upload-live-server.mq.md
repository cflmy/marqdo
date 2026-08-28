---
title: web upload live server (W5)
description: Listen with upload + download routes for curl gold.
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

*blob = > web.storage url="file:tests/ext/web-fixtures/data/upload-live-blobs"*

`types` =

| 类型 | 扩展名 |
|------|--------|
| text/plain | txt |
| image/png | png |

*pg = > web.page title="upload-live"*
*app = > web.app page=`pg` port=18101*
*app = > `app`.configure body_limit=1048576*
*app = > `app`.upload path="/_upload" field="file" storage=`blob` prefix="live/" max_bytes=1024 types=`types`*
*app = > `app`.download path="/_media/{*key}" storage=`blob` disposition="attachment"*
> `app`.listen
