---
title: components/feed
description: 帖子列表 / 新闻时间线 / 板块栏 HTML 片段（对齐 Django feed 与 home 局部模板）。
import table:lib/table.mq.md
import text:lib/text.mq.md
---

## 帖子列表HTML
    + `行`

给定帖子行记录列表，输出 `ul.home-posts-list` 标记（首页与列表页共用）。

**段 = None**
- [帖](行)
  **板 = [board](帖)**
  **题 = [title](帖)**
  **链 = [slug](帖)**
  **作 = [author](帖)**
  **评 = [comments_count](帖)**
  **时 = [created_at](帖)**
  1. not 作
    **作 = "匿名"**
  2. *
  1. not 评
    **评 = 0**
  2. *
  1. not 时
    **时 = ""**
  2. *
  **一项 = "<li><article class=\"home-post-card feed-post-card\"><a href=\"/post/" + 链 + "\" class=\"home-post-card-link feed-post-card-link\"><span class=\"home-post-board feed-post-board\">" + 板 + "</span><div class=\"home-post-body feed-post-body\"><h3 class=\"home-post-title feed-post-title\">" + 题 + "</h3><div class=\"home-post-meta feed-post-meta\"><span class=\"home-post-meta-item feed-post-meta-item\"><span class=\"anlian-icon anlian-icon--user\" aria-hidden=\"true\"></span> " + 作 + "</span><span class=\"home-post-meta-item feed-post-meta-item\"><span class=\"anlian-icon anlian-icon--comment\" aria-hidden=\"true\"></span> " + 评 + "</span><time>" + 时 + "</time></div></div><span class=\"home-post-arrow feed-post-arrow\" aria-hidden=\"true\"><span class=\"anlian-icon anlian-icon--chevron-right\"></span></span></a></article></li>"**
  **段 = > table.append list=段 item=一项**
1. 段
  **内 = > text.str_join xs=段 sep=""**
  *"<ul class=\"home-posts-list feed-posts-list\">" + 内 + "</ul>"*
2. *
  *"<ul class=\"home-posts-list feed-posts-list\"><li class=\"home-posts-empty feed-empty\">暂无帖子，<a href=\"/post/public\">去发帖</a></li></ul>"*

## 新闻时间线HTML
    + `行`

给定新闻行记录列表，输出 `.news-timeline` 标记。

**段 = None**
- [讯](行)
  **题 = [title](讯)**
  **链 = [slug](讯)**
  **板 = [board](讯)**
  **时 = [created_at](讯)**
  1. not 板
    **板 = ""**
  2. *
  1. not 时
    **时 = ""**
  2. *
  **一项 = "<li class=\"news-timeline-item\"><span class=\"news-timeline-node\" aria-hidden=\"true\"></span><article class=\"news-timeline-card\"><div class=\"news-timeline-meta\"><time class=\"news-timeline-date\">" + 时 + "</time><span class=\"news-timeline-board\">" + 板 + "</span></div><h3 class=\"news-timeline-title\"><a href=\"/news/" + 链 + "\" class=\"news-timeline-link\">" + 题 + "</a></h3></article></li>"**
  **段 = > table.append list=段 item=一项**
1. 段
  **内 = > text.str_join xs=段 sep=""**
  *"<div class=\"news-timeline\"><div class=\"news-timeline-rail\" aria-hidden=\"true\"><span class=\"news-timeline-line\"></span><span class=\"news-timeline-arrow\" aria-hidden=\"true\"><span class=\"anlian-icon anlian-icon--chevron-right news-timeline-arrow-icon--right\"></span><span class=\"anlian-icon anlian-icon--chevron-down news-timeline-arrow-icon--down\"></span></span></div><ol class=\"news-timeline-items\">" + 内 + "</ol></div>"*
2. *
  *"<div class=\"news-timeline\"><div class=\"news-timeline-rail\" aria-hidden=\"true\"><span class=\"news-timeline-line\"></span><span class=\"news-timeline-arrow\" aria-hidden=\"true\"></span></div><ol class=\"news-timeline-items\"><li class=\"news-timeline-item news-timeline-item-empty\"><article class=\"news-timeline-card\"><p class=\"news-timeline-empty\">暂无新闻</p></article></li></ol></div>"*

## 板块栏HTML
    + `板块`
    + `前缀`
    + `查询键`
    + `全部类`

板块 Tab 栏。`前缀` 如 `/post/ten/`；`查询键` 为 `board_id` 或 `news_board_id`；`全部类` 为 Tab 共用 class（含 js hook）。

**数据属 = "data-board"**
**全标签 = "全部"**
1. 查询键 == "news_board_id"
  **数据属 = "data-news-board"**
  **全标签 = "所有板块"**
2. *

**段 = None**
**全链 = 前缀 + "?" + 查询键 + "=0"**
**全项 = "<a href=\"" + 全链 + "\" class=\"" + 全部类 + " active\" " + 数据属 + "=\"0\">" + 全标签 + "</a>"**
**段 = > table.append list=段 item=全项**
- [板](板块)
  **名 = [name](板)**
  **号 = [id](板)**
  **链 = 前缀 + "?" + 查询键 + "=" + 号**
  **项 = "<a href=\"" + 链 + "\" class=\"" + 全部类 + "\" " + 数据属 + "=\"" + 号 + "\">" + 名 + "</a>"**
  **段 = > table.append list=段 item=项**
1. 查询键 == "news_board_id"
  **更多 = "<a href=\"/news\" class=\"home-posts-tab home-posts-tab-more\">全部新闻</a>"**
  **段 = > table.append list=段 item=更多**
2. *
**内 = > text.str_join xs=段 sep=""**
**栏类 = "home-posts-board-bar"**
1. 查询键 == "news_board_id"
  **栏类 = "home-posts-board-bar home-news-board-bar"**
2. *
*"<div class=\"" + 栏类 + "\" data-feed-board><button type=\"button\" class=\"home-posts-scroll home-posts-scroll-left\" aria-label=\"向左滚动\"><span class=\"anlian-icon anlian-icon--chevron-left\"></span></button><div class=\"home-posts-segment-outer\"><div class=\"home-posts-segment\"><div class=\"home-posts-spotlight\" aria-hidden=\"true\"></div><div class=\"home-posts-tabs-scroll\"><div class=\"home-posts-tabs\" role=\"tablist\">" + 内 + "</div></div></div></div><button type=\"button\" class=\"home-posts-scroll home-posts-scroll-right\" aria-label=\"向右滚动\"><span class=\"anlian-icon anlian-icon--chevron-right\"></span></button></div>"*
