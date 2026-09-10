---
title: components/fragments
description: AJAX 片段 — /post/ten、/news/five。
导入 网页:ext/web/网页.mq.md
import db:../db/index.mq.md
import feed:feed.mq.md
---

## 帖子十
    + `board_id`="0"

*store = > db.打开*
*行 = > store.查询 表="posts" 上限=10 排序="-created_at"*
**> feed.帖子列表HTML 行=`行`**

## 新闻五
    + `news_board_id`="0"

*store = > db.打开*
*行 = > store.查询 表="news" 上限=5 排序="-created_at"*
**> feed.新闻时间线HTML 行=`行`**
