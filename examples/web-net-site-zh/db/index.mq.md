---
title: db/index
description: 打开 sqlite、建表、幂等种子。
导入 网页:ext/web/网页.mq.md
import articles:articles.mq.md
---

## 打开

*store = > 网页.数据库 地址="sqlite:data/site-net-zh.db"*
*字段 = > articles.结构*
> `store`.初始化 名=articles 字段=`字段`
*行 = > store.查询 表="articles" 上限=1*
1. `行`
  **store**
2. *
  *种子 = > articles.种子*
  > `store`.插入 表=articles 行=`种子`
  **store**