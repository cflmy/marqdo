---
title: web db.select where smoke
description: Offline select with map and filter-table where (no listen).
import web:ext/web/web.mq.md
import articles:web-fixtures/db/articles.mq.md
import json:lib/json.mq.md
---

# main

*store = > web.db url="sqlite:web-fixtures/data/select-smoke.db"*
*schema = > articles.schema*
> `store`.init name=articles fields=`schema`
> `store`.exec sql="DELETE FROM articles"

`rows` =

| title | body |
|-------|------|
| Alpha | first |
| Beta | second |
| Alpha again | third |

> `store`.insert table=articles rows=`rows`

`eq` =

| title | body |
|-------|------|
| Alpha | first |

*hit = > `store`.select table="articles" where=`eq` limit=50*
*n = > len value=`hit`*
1. `n` == 1
  > print text=map-where-ok
2. *
  > print text=map-where-fail

`filt` =

| 字段 | 操作 | 值 |
|------|------|-----|
| title | like | %Alpha% |

*like_rows = > `store`.select table="articles" where=`filt` limit=50*
*ln = > len value=`like_rows`*
1. `ln` == 2
  > print text=like-ok
2. *
  > print text=like-fail

*all = > `store`.select table="articles" limit=50*
*an = > len value=`all`*
1. `an` == 3
  > print text=all-ok
2. *
  > print text=all-fail
