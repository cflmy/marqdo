---
title: web drivers smoke (W4)
description: memory cache + file storage + postgres URL shape (offline).
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

<!-- cache: memory backend -->

*c = > web.cache url="memory:drivers-smoke"*
> `c`.set key="k1" value="v1" ttl=60
*g = > `c`.get key="k1"*
*gv = g[^value]*
1. `gv` == "v1"
  > print text=cache-ok
2. *
  > print text=cache-fail

> `c`.del key="k1"
*ex = > `c`.exists key="k1"*
*eok = ex[^ok]*
1. `eok`
  > print text=cache-del-fail
2. *
  > print text=cache-del-ok

<!-- storage: file backend -->

*blob = > web.storage url="file:tests/ext/web-fixtures/data/drivers-blobs"*
> `blob`.put key="notes/a.txt" body="hello" content_type="text/plain"
*got = > `blob`.get key="notes/a.txt"*
*body = got[^body]*
1. `body` == "hello"
  > print text=storage-ok
2. *
  > print text=storage-fail

*ls = > `blob`.list prefix="notes/"*
*n = ls[^count]*
1. `n` == 1
  > print text=storage-list-ok
2. *
  > print text=storage-list-fail

> `blob`.delete key="notes/a.txt"

<!-- db: postgres URL is accepted as a handle (no live server required) -->

*pg = > web.db url="postgres://marqdo:marqdo@127.0.0.1:5432/marqdo"*
*purl = pg[^url]*
1. `purl` == "postgres://marqdo:marqdo@127.0.0.1:5432/marqdo"
  > print text=postgres-url-ok
2. *
  > print text=postgres-url-fail

<!-- s3 URL shape validates at open -->

*s3 = > web.storage url="s3://bucket?endpoint=http://127.0.0.1:9000"*
*sb = s3[^backend]*
1. `sb` == "s3"
  > print text=s3-open-ok
2. *
  > print text=s3-open-fail
