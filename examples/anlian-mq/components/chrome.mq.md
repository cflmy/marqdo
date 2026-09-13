---
title: components/chrome
description: 暗恋见君壳层 — head 资源表、导航 HTML、页脚 HTML（对齐 Django base / site_footer）。
---

## 头资源

首页与浏览页共用的 CSS / JS 链接表（挂到 `页面.头装配`）。路径对齐 `public/` 静态树。

`头` =

| 关系 | 地址 | 推迟 |
|------|------|------|
| stylesheet | "/static/vendor/bootstrap-anlian.min.css" | |
| stylesheet | "/static/css/base.css" | |
| stylesheet | "/static/css/navbar_search.css" | |
| stylesheet | "/static/css/footer.css" | |
| stylesheet | "/static/css/icons.css" | |
| stylesheet | "/static/css/feed_common.css" | |
| stylesheet | "/static/css/feed_list.css" | |
| stylesheet | "/static/css/index.css" | |
| stylesheet | "/static/css/index/hero.css" | |
| stylesheet | "/static/css/index/notice.css" | |
| stylesheet | "/static/css/index/home_news_timeline.css" | |
| stylesheet | "/static/css/index/home_posts.css" | |
| stylesheet | "/static/css/anlian_forms.css" | |
| stylesheet | "/static/css/login.css" | |
| stylesheet | "/static/css/rich_content.css" | |
| stylesheet | "/static/css/detail_reading.css" | |
| stylesheet | "/static/css/search.css" | |
| stylesheet | "/static/css/chat.css" | |
| stylesheet | "/static/css/topic_index.css" | |
| script | "/static/vendor/bootstrap.bundle.min.js" | 真 |
| script | "/static/js/anlian_partial.js" | 真 |
| script | "/static/js/feed_board.js" | 真 |
| script | "/static/js/index.js" | 真 |

*`头`*

## 导航HTML

对齐 Django `front/base.html` 顶栏：品牌「暗恋见君论坛」、主导航、搜索、登录/退出（`nav-when-guest` / `nav-when-auth` 由 CSS/JS 切换）。

**"<nav class=\"navbar navbar-expand-lg navbar-dark bg-dark\"><div class=\"container-fluid navbar-shell\"><div class=\"navbar-start\"><div class=\"navbar-logo\"><img src=\"/static/img/logo.svg\" alt=\"logo\" width=\"30\" height=\"30\" decoding=\"async\"></div><a class=\"navbar-brand\" href=\"/\">暗恋见君论坛</a></div><button class=\"navbar-toggler\" type=\"button\" data-bs-toggle=\"collapse\" data-bs-target=\"#navbarNav\" aria-controls=\"navbarNav\" aria-expanded=\"false\" aria-label=\"Toggle navigation\"><span class=\"navbar-toggler-icon\"></span></button><div class=\"collapse navbar-collapse\" id=\"navbarNav\"><ul class=\"navbar-nav navbar-nav-main\"><li class=\"nav-item\"><a class=\"nav-link\" href=\"/\">首页</a></li><li class=\"nav-item\"><a class=\"nav-link\" href=\"/post\">帖子</a></li><li class=\"nav-item\"><a class=\"nav-link\" href=\"/news\">新闻</a></li><li class=\"nav-item\"><a class=\"nav-link\" href=\"/topics\">专题</a></li><li class=\"nav-item\"><a class=\"nav-link\" href=\"/chat\">聊天</a></li><li class=\"nav-item\"><a class=\"nav-link\" href=\"/post/public\">发帖</a></li></ul><div class=\"navbar-end\"><form class=\"navbar-search d-flex\" action=\"/search\" method=\"get\" role=\"search\"><input class=\"form-control\" type=\"search\" name=\"q\" placeholder=\"搜索帖子、新闻、用户\" aria-label=\"搜索\" maxlength=\"100\" autocomplete=\"off\"><button class=\"btn btn-search-submit\" type=\"submit\">搜索</button></form><ul class=\"navbar-nav navbar-auth\"><li class=\"nav-item nav-when-guest\"><a class=\"nav-link nav-link-auth\" href=\"/accounts/login\">登录</a></li><li class=\"nav-item nav-when-auth\"><a class=\"nav-link nav-link-btn\" href=\"/admin/logout\">退出</a></li></ul></div></div></div></nav>"**

## 页脚HTML

对齐 `site_footer`：站点列 + 作者列；标语、版权、API 指南、作者 cflmy。

**"<footer class=\"site-footer\"><div class=\"site-footer-inner\"><div class=\"footer-col footer-col-site\"><img src=\"/static/img/logo.svg\" alt=\"\" class=\"footer-site-logo\" width=\"40\" height=\"40\" loading=\"lazy\"><p class=\"footer-site-name\">暗恋见君论坛</p><p class=\"footer-site-tagline\">期待人们能够继续相信爱情</p><p class=\"footer-copy\">&copy; 2026 暗恋见君论坛</p><p class=\"footer-meta-sm footer-api-guide\"><a href=\"/guide/automation-api\" class=\"footer-guide-btn\" title=\"面向开发者的 API Key 与自动化对接说明\"><span class=\"anlian-icon anlian-icon--book\" aria-hidden=\"true\"><svg viewBox=\"0 0 24 24\" xmlns=\"http://www.w3.org/2000/svg\" focusable=\"false\" aria-hidden=\"true\"><path d=\"M6 2a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V4a2 2 0 0 0-2-2H6Zm0 2h5v7.5L8.5 10 6 11.5V4Zm7 0h5v16h-5V4Z\"/></svg></span> API 自动化指南</a></p></div><div class=\"footer-col footer-col-author\"><img src=\"/static/img/author_avatar-144.webp\" alt=\"\" class=\"footer-author-avatar\" width=\"72\" height=\"72\" loading=\"lazy\"><div class=\"footer-author-info\"><p class=\"footer-author-name\">cflmy</p><p class=\"footer-author-title\">站长</p></div></div></div></footer>"**
