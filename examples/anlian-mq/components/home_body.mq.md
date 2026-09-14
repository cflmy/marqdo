---
title: components/home_body
description: 首页主体 HTML — hero + 公告 + 新闻时间线 + 论坛帖子（对齐 Django home.html，无外层导航）。
导入 网页:ext/web/网页.mq.md
import table:lib/table.mq.md
import text:lib/text.mq.md
import feed:feed.mq.md
---

## 装配
    + `库`

查询 notices / boards / news_boards / news(5) / posts(10)，拼装首页四段 HTML（不含导航与页脚）。调用方传入已打开的 store。

**公告行 = > 库.查询 表="notices" 上限=20 排序="-created_at"**
**板块行 = > 库.查询 表="boards" 上限=50**
**新闻板块行 = > 库.查询 表="news_boards" 上限=50**
**新闻行 = > 库.查询 表="news" 上限=5 排序="-created_at"**
**帖子行 = > 库.查询 表="posts" 上限=10 排序="-created_at"**

**公告段 = None**
- [告](公告行)
  **文 = [content](告)**
  **时 = [created_at](告)**
  **一项 = "<div class=\"notice-item\"><article class=\"notice-content\">" + 文 + "</article><p class=\"notice-create-time\">" + 时 + "</p></div>"**
  **公告段 = > table.append list=公告段 item=一项**
1. 公告段
  **公告内 = > text.str_join xs=公告段 sep=""**
2. *
  **公告内 = "<p class=\"text-muted\">暂无公告</p>"**

**新闻栏 = > feed.板块栏HTML 板块=新闻板块行 前缀="/news/five" 查询键="news_board_id" 全部类="home-posts-tab js-home-news-tab"**
**新闻板 = > feed.新闻时间线HTML 行=新闻行**
**帖子栏 = > feed.板块栏HTML 板块=板块行 前缀="/post/ten" 查询键="board_id" 全部类="home-posts-tab js-home-post-tab"**
**帖子表 = > feed.帖子列表HTML 行=帖子行**

**段 = None**
**英雄 = "<section class=\"hero\"><div class=\"section-wrap\"><header><img src=\"/static/img/logo.svg\" alt=\"logo\" width=\"30\" height=\"30\" decoding=\"async\"><a class=\"head-logo\" href=\"/\"><span>暗恋</span>见君</a></header><div class=\"hero-content\"><div class=\"left\"><h1 class=\"main-text\">期待人们能够继续相信<span style=\"color:#ffd0d0\">爱情</span></h1><h2 class=\"sub-text\">粉玻璃社区 · 匿名与实名同在</h2></div><div class=\"right\"><img src=\"/static/img/bg-hero-640.webp\" alt=\"首页展示图\" width=\"640\" height=\"482\" fetchpriority=\"high\" decoding=\"async\"></div></div><div class=\"social\"></div></div></section>"**
**段 = > table.append list=段 item=英雄**
**公告区 = "<section class=\"notice\"><div class=\"section-wrap\"><h2 class=\"notice-title\">公告信息</h2><div class=\"notice-content-container\"><div class=\"notice-list\">" + 公告内 + "</div></div></div></section>"**
**段 = > table.append list=段 item=公告区**
**新闻区 = "<section class=\"news home-news\" id=\"news\"><div class=\"section-wrap home-news-stage\"><h2 class=\"news-title\">新闻动态</h2>" + 新闻栏 + "<div class=\"news-timeline-wrap\"><div id=\"homeNewsPanel\" class=\"news-timeline-panel home-panel-swap\">" + 新闻板 + "</div></div></div></section>"**
**段 = > table.append list=段 item=新闻区**
**更多 = "<a href=\"/post\" class=\"home-posts-more-link\">查看更多帖子</a>"**
**帖子区 = "<section class=\"home-posts\" id=\"post\"><div class=\"section-wrap home-posts-stage\"><h2 class=\"home-posts-title\">论坛交流</h2>" + 帖子栏 + "<div id=\"tenPostContainer\" class=\"home-posts-panel home-panel-swap\">" + 帖子表 + 更多 + "</div></div></section>"**
**段 = > table.append list=段 item=帖子区**
*> text.str_join xs=段 sep=""*
