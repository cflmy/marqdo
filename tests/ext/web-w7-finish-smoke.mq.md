---
title: web W7 finish smoke — unique index, sitemap, cache-control (offline)
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

**store = > web.db url="sqlite:web-fixtures/data/w7-smoke.db"**
> `store`.exec sql="DROP TABLE IF EXISTS posts"

`fields` =

| 字段 | 类型 | 可空 | 唯一 | 索引 |
|------|------|------|------|------|
| id | int | 否 | | |
| slug | text | 否 | 是 | 是 |
| title | text | 否 | | |

> `store`.init name=posts fields=`fields`

**idx = > `store`.query sql="SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='posts'"**
**nidx = > len value=[rows](idx)**
1. `nidx` >= 1
  > print text=unique-ok
2. *
  > print text=unique-fail

`items` =

| loc | 更新 |
|-----|------|
| / | 2026-08-28 |
| /about | 2026-08-28 |

**sm = > web_sitemap_build base="https://example.com" items=`items`**
**xml = [xml](sm)**
1. `xml`
  > print text=sitemap-ok
2. *
  > print text=sitemap-fail

**pg = > web.page title="w7"**
**notfound = > web.page title="Missing"**
**app = > web.app page=`pg` port=18120**
**app = > `app`.configure cache_control="public, max-age=60" access_log=False**
**app = > `app`.redirect from="/old" to="/new" permanent=True**
**app = > `app`.error_page status=404 page=`notfound`**
**app = > `app`.sitemap path="/sitemap.xml" base="https://example.com" items=`items`**
**app = > `app`.robots sitemap="https://example.com/sitemap.xml"**
**rd = [redirects](app)**
**has = [/old](rd)**
1. `has`
  > print text=redirect-ok
2. *
  > print text=redirect-fail

**rb = [robots_body](app)**
1. `rb`
  > print text=robots-ok
2. *
  > print text=robots-fail

**mw = [middleware](app)**
**cc = [cache_control](mw)**
1. `cc` == "public, max-age=60"
  > print text=cache-control-ok
2. *
  > print text=cache-control-fail

**p404 = [page_404](app)**
1. `p404`
  > print text=error-page-ok
2. *
  > print text=error-page-fail
