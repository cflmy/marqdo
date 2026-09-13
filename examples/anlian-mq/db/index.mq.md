---
title: db/index
description: 打开库、建全表、幂等种子、帖子全文索引。
导入 网页:ext/web/网页.mq.md
import schema:schema.mq.md
import seed:seed.mq.md
---

## 打开

打开 sqlite:data/anlian-mq.db，初始化全部表；帖子建 FTS；若 posts 为空则写入种子。返回 store。

**store = > 网页.数据库 地址="sqlite:data/anlian-mq.db"**
**帖子字段 = > schema.posts**
**新闻字段 = > schema.news**
**板块字段 = > schema.boards**
**新闻板块字段 = > schema.news_boards**
**公告字段 = > schema.notices**
**评论字段 = > schema.comments**
**房间字段 = > schema.chat_rooms**
**专题字段 = > schema.topics**
**密钥字段 = > schema.api_keys**
> `store`.初始化 名=posts 字段=`帖子字段`
> `store`.初始化 名=news 字段=`新闻字段`
> `store`.初始化 名=boards 字段=`板块字段`
> `store`.初始化 名=news_boards 字段=`新闻板块字段`
> `store`.初始化 名=notices 字段=`公告字段`
> `store`.初始化 名=comments 字段=`评论字段`
> `store`.初始化 名=chat_rooms 字段=`房间字段`
> `store`.初始化 名=topics 字段=`专题字段`
> `store`.初始化 名=api_keys 字段=`密钥字段`

`全文列` =

| 列 |
|----|
| title |
| summary |
| body |

> `store`.全文 表=posts 列=`全文列` 名=posts_fts

**已有 = > store.查询 表="posts" 上限=1**
1. `已有`
  *store*
2. *
  **板块 = > seed.boards**
  **新闻板块 = > seed.news_boards**
  **帖子 = > seed.posts**
  **新闻 = > seed.news**
  **公告 = > seed.notices**
  **评论 = > seed.comments**
  **房间 = > seed.chat_rooms**
  **专题 = > seed.topics**
  > `store`.插入 表=boards 行=`板块`
  > `store`.插入 表=news_boards 行=`新闻板块`
  > `store`.插入 表=posts 行=`帖子`
  > `store`.插入 表=news 行=`新闻`
  > `store`.插入 表=notices 行=`公告`
  > `store`.插入 表=comments 行=`评论`
  > `store`.插入 表=chat_rooms 行=`房间`
  > `store`.插入 表=topics 行=`专题`
  *store*
