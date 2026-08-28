---
title: web db W6 smoke — migrate / FTS5 / published filter
import web:ext/web/web.mq.md
---

# main

*store = > web.db url="sqlite:web-fixtures/data/w6-smoke.db"*

`steps` =

| 版本 | SQL |
|------|-----|
| 1 | CREATE TABLE IF NOT EXISTS posts (id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT NOT NULL, body TEXT, status TEXT NOT NULL DEFAULT 'draft') |
| 2 | CREATE TABLE IF NOT EXISTS comments (id INTEGER PRIMARY KEY AUTOINCREMENT, post_id INTEGER NOT NULL, body TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'pending') |

*m1 = > `store`.migrate steps=`steps`*
*a1 = m1[^applied]*
*n1 = > len value=`a1`*
1. `n1` == 2
  > print text=migrate-ok
2. *
  > print text=migrate-fail

*m2 = > `store`.migrate steps=`steps`*
*a2 = m2[^applied]*
*n2 = > len value=`a2`*
1. `n2` == 0
  > print text=migrate-idempotent-ok
2. *
  > print text=migrate-idempotent-fail

`rows` =

| title | body | status |
|-------|------|--------|
| Alpha note | hello world search | published |
| Beta draft | secret draft body | draft |
| Gamma published | another hello | published |

> `store`.insert table=posts rows=`rows`

`pub` =

| 字段 | 操作 | 值 |
|------|------|-----|
| status | = | published |

*pc = > `store`.count table="posts" where=`pub`*
1. `pc` == 2
  > print text=published-filter-ok
2. *
  > print text=published-filter-fail

`comments` =

| post_id | body | status |
|---------|------|--------|
| 1 | nice post | approved |
| 1 | spam | pending |

> `store`.insert table=comments rows=`comments`
*cc = > `store`.count table="comments"*
1. `cc` == 2
  > print text=comments-ok
2. *
  > print text=comments-fail

> `store`.fts table="posts" columns="title,body"
*hit = > `store`.search table="posts" q="hello" limit=10*
*hn = hit[^count]*
1. `hn` >= 1
  > print text=fts-ok
2. *
  > print text=fts-fail

*rows2 = hit[^rows]*
*first = rows2[^1]*
*ft = first[^title]*
1. `ft`
  > print text=fts-row-ok
2. *
  > print text=fts-row-fail
