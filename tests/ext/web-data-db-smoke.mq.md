---
title: ext/data db smoke
description: DB Resource lives in ext/data (ADR 0007).
import data:ext/data/db.mq.md
import fs:lib/fs.mq.md
---

# main

**store = > data.db url="sqlite:web-fixtures/data/artifact-db.db"**
`fields` =

| 字段 | 类型 | 可空 |
|------|------|------|
| title | text | False |
| body | text | True |

> `store`.init name=notes fields=`fields`
`row` =

| title | body |
|-------|------|
| hello | world |

> `store`.insert table=notes rows=`row`
**got = > `store`.select table="notes" limit=1**
1. `got`
  > print text=db-ok
2. *
  > print text=db-fail
