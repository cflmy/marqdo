---
title: db/index
description: 打开库、建表、幂等种子、全文索引。
导入 网页:ext/web/网页.mq.md
import schema:schema.mq.md
import seed:seed.mq.md
---

## 打开

*store = > 网页.数据库 地址="sqlite:data/anlian-mq.db"*
*帖子字段 = > schema.posts*
*新闻字段 = > schema.news*
*密钥字段 = > schema.api_keys*
> `store`.初始化 名=posts 字段=`帖子字段`
> `store`.初始化 名=news 字段=`新闻字段`
> `store`.初始化 名=api_keys 字段=`密钥字段`

`全文列` =

| 列 |
|----|
| title |
| summary |
| body |

> `store`.全文 表=posts 列=`全文列` 名=posts_fts

*已有 = > store.查询 表="posts" 上限=1*
1. `已有`
  **store**
2. *
  *帖子 = > seed.posts*
  *新闻 = > seed.news*
  > `store`.插入 表=posts 行=`帖子`
  > `store`.插入 表=news 行=`新闻`
  **store**
