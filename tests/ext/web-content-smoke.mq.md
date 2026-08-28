---
title: web content smoke (W4)
description: SEO meta + paginate bag (offline ABI).
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

`meta` =

| 行 | 键 | 值 |
|----|-----|-----|
| 1 | title | My Site |
| 2 | description | A demo site |
| 3 | og:type | website |

*page = > web.page title="Home"*
*page = > page.meta meta=`meta`*
*mm = page[^meta]*
*m = mm[^title]*
1. `m` == "My Site"
  > print text=meta-ok
2. *
  > print text=meta-fail

*page = > page.paginate offset=0 limit=5 path="/"*
*poff = page[^paginate][^offset]*
1. `poff` == 0
  > print text=paginate-ok
2. *
  > print text=paginate-fail

`rows` =

| 行 | title | slug | summary |
|----|-------|------|---------|
| 1 | Hello | /a | First post |

*feed = > web_rss_build title="Blog" link="http://example.com" description="Demo" items=`rows`*
1. `feed` != ""
  > print text=rss-ok
2. *
  > print text=rss-fail
