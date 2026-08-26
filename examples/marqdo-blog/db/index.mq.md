---
title: db/index
description: 打开 sqlite、建三张表、幂等种子。
导入 网页:ext/web/网页.mq.md
import schema:schema.mq.md
import seed:seed.mq.md
---

## 打开

*store = > 网页.数据库 地址="sqlite:data/marqdo-blog.db"*
*字段 = > schema.posts*
*标签字段 = > schema.tags*
*关联字段 = > schema.post_tags*
> `store`.初始化 名=posts 字段=`字段`
> `store`.初始化 名=tags 字段=`标签字段`
> `store`.初始化 名=post_tags 字段=`关联字段`
*行 = > store.查询 表="posts" 上限=1*
1. `行`
  **store**
2. *
  *文章 = > seed.posts*
  *标签 = > seed.tags*
  *关联 = > seed.post_tags*
  > `store`.插入 表=posts 行=`文章`
  > `store`.插入 表=tags 行=`标签`
  > `store`.插入 表=post_tags 行=`关联`
  **store**