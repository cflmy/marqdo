---
title: web db W2 smoke — 事务 / 分页 / 原生查询 / 计数 / 查询表达力
import web:ext/web/web.mq.md
import json:lib/json.mq.md
---

# main

*store = > web.db url="sqlite:web-fixtures/data/w2-smoke.db"*

`字段` =

| 字段 | 类型 | 可空 |
|------|------|------|
| title | text | 否 |
| body | text | 是 |
| views | int | 是 |

> `store`.init name=posts fields=`字段`
> `store`.exec sql="DELETE FROM posts"

`数据` =

| title | body | views |
|-------|------|-------|
| Alpha | first | 10 |
| Beta | second | 20 |
| Gamma | third | 30 |
| Delta | fourth | 40 |

> `store`.insert table=posts rows=`数据`

*c = > `store`.count table="posts"*
1. `c` == 4
  > print text=count-ok
2. *
  > print text=count-fail

`视图条件` =

| 字段 | 操作 | 值 |
|------|------|-----|
| views | = | 30 |

*c2 = > `store`.count table="posts" where=`视图条件`*
1. `c2` == 1
  > print text=count-where-ok
2. *
  > print text=count-where-fail

*page1 = > `store`.paginate table="posts" limit=2 跳过=0*
*total1 = page1[^total]*
*p1r = page1[^rows]*
*n1 = > len value=`p1r`*
1. `total1` == 4
1. `n1` == 2
  > print text=page-ok
2. *
  > print text=page-fail

*page2 = > `store`.paginate table="posts" limit=2 跳过=2*
*p2r = page2[^rows]*
*n2 = > len value=`p2r`*
1. `n2` == 2
  > print text=page2-ok
2. *
  > print text=page2-fail

`in条件` =

| 字段 | 操作 | 值 |
|------|------|-----|
| title | in | "Alpha,Beta" |

*in_rows = > `store`.select table="posts" where=`in条件` limit=10*
*nin = > len value=`in_rows`*
1. `nin` == 2
  > print text=in-ok
2. *
  > print text=in-fail

`between条件` =

| 字段 | 操作 | 值 |
|------|------|-----|
| views | between | "20,35" |

*bt_rows = > `store`.select table="posts" where=`between条件` limit=10*
*nbt = > len value=`bt_rows`*
1. `nbt` == 2
  > print text=between-ok
2. *
  > print text=between-fail

`or条件` =

| 字段 | 操作 | 值 | 或 |
|------|------|-----|-----|
| title | = | Alpha | |
| title | = | Gamma | 是 |

*or_rows = > `store`.select table="posts" where=`or条件` limit=10*
*nor = > len value=`or_rows`*
1. `nor` == 2
  > print text=or-ok
2. *
  > print text=or-fail

*agg = > `store`.query sql="SELECT title, views FROM posts WHERE views >= 30 ORDER BY views DESC"*
*nagg = agg[^count]*
1. `nagg` == 2
  > print text=query-ok
2. *
  > print text=query-fail

*txn = > `store`.事务*
`更多` =

| title | body | views |
|-------|------|-------|
| Epsilon | fifth | 50 |

> `txn`.insert table=posts rows=`更多`
> `txn`.提交

*c5 = > `store`.count table="posts"*
1. `c5` == 5
  > print text=txn-commit-ok
2. *
  > print text=txn-commit-fail

*txn2 = > `store`.事务*
`更多2` =

| title | body | views |
|-------|------|-------|
| Zeta | sixth | 60 |

> `txn2`.insert table=posts rows=`更多2`
> `txn2`.回滚

*c6 = > `store`.count table="posts"*
1. `c6` == 5
  > print text=txn-rollback-ok
2. *
  > print text=txn-rollback-fail
