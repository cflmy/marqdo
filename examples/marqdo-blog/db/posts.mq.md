---
title: blog posts schema
import data:ext/data/db.mq.md
---

## schema

`fields` =

| 字段 | 类型 | 可空 |
|------|------|------|
| title | text | False |
| slug | text | False |
| summary | text | True |
| content | text | True |
| tag | text | True |
| created_at | text | True |

*fields*

## seed

`rows` =

| title | slug | summary | content | tag | created_at |
|-------|------|---------|---------|-----|------------|
| Hello Marqdo | hello | Welcome | # Hello\n\nExecutable documents. | intro | 2026-09-01 |
| Artifact Web | artifact-web | Document first | Pages are mq.md files. | design | 2026-09-28 |

*rows*

## open
    + `url`="sqlite:data/blog.db"

**store = > data.db url=`url`**
**fields = > schema**
> `store`.init name=posts fields=`fields`
**existing = > `store`.select table="posts" limit=1**
1. `existing`
  *store*
2. *
  **seed = > seed**
  > `store`.insert table=posts rows=`seed`
  *store*
