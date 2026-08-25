---
title: db/index
description: Open sqlite, init articles, seed once.
import web:ext/web/web.mq.md
import articles:articles.mq.md
---

## open

*store = > web.db url="sqlite:data/site-net.db"*
*fields = > articles.schema*
> `store`.init name=articles fields=`fields`
*rows = > store.select table="articles" limit=1*
1. `rows`
  **store**
2. *
  *seed = > articles.seed*
  > `store`.insert table=articles rows=`seed`
  **store**