---
title: web entry-dir relative path smoke
description: Relative db path resolves against the entry script dir, not cwd.
import web:ext/web/web.mq.md
---

# main

<!--
  Run from a different cwd than the script dir (e.g. repo root):
  the sqlite file must land under <script dir>/entrydir-fixtures/data,
  NOT under the process cwd.
-->

*store = > web.db url="sqlite:entrydir-fixtures/data/entrydir.db"*

`schema` =

| 字段 | 类型 |
|------|------|
| title | text |

> `store`.init name=items fields=`schema`

`one` =

| 行 | title |
|----|-------|
| 1 | hello |

> `store`.insert table=items rows=`one`

*rows = > `store`.select table="items" limit=10*
*n = > len value=`rows`*

1. `n` == 1
  > print text=entrydir-db-ok
2. *
  > print text=entrydir-db-fail
