---
title: web P3 live — gallery HTML + download ETag
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

*blob = > web.storage url="file:web-fixtures/data/p3-live-blobs"*
> `blob`.put key="uploads/hello.txt" body="etag-body" content_type="text/plain"

*pg = > web.page title="p3-live"*
*app = > web.app page=`pg` port=18131*
*app = > `app`.download path="/_media/{*key}" storage=`blob` disposition="inline"*
*app = > `app`.gallery path="/gallery" storage=`blob` prefix="uploads/" title="P3 Gallery" download_base="/_media"*
> `app`.listen
