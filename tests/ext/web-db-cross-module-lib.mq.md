---
title: web db cross-module lib
description: Opens sqlite, inits a tiny table, returns the db handle.
import web:ext/web/web.mq.md
---

## open

*store = > web.db url="sqlite::memory:"*

`fields` =

| name | type |
|------|------|
| title | text |

> `store`.init name=items fields=`fields`

`seed` =

| @ | title |
|---|-------|
| 1 | hello |

> `store`.insert table=items rows=`seed`
**store**
