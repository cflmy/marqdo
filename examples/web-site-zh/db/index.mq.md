---
title: db/index
description: 打开 sqlite、建表、幂等种子。
导入 网页:ext/web/网页.mq.md
import articles:articles.mq.md
---

## open

*`store` = > 网页.数据库 地址="sqlite:data/site-zh.db" *
*`fields` = > articles.schema *
> `store`.初始化 名=articles 字段=`fields`
*`rows` = > `store`.查询 表=articles 上限=1 *
1. `rows`
  **`store`**
2. *
  *`seed` = > articles.seed *
  > `store`.插入 表=articles 行=`seed`
  **`store`**
