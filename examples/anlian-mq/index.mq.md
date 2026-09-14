---
title: 暗恋见君 · Marqdo
description: 对齐 Django 暗恋见君：原站 CSS/DOM、首页四段、列表详情、发帖门禁、聊天室、专题、搜索、API 指南。
导入 网页:ext/web/网页.mq.md
import table:lib/table.mq.md
import text:lib/text.mq.md
import chrome:components/chrome.mq.md
import page:components/page.mq.md
import home:components/home_body.mq.md
import feed:components/feed.mq.md
import fragments:components/fragments.mq.md
import db:db/index.mq.md
---

# main

路径对齐原站：`/post`、`/news`、`/post/public`、`/chat`、`/topics`、`/search`、`/accounts/login`。

`用户表` =

| 行 | 用户名 | 密码 | 角色 |
|----|--------|------|------|
| 1 | admin | anlian | admin |

`发帖字段` =

| 字段 | 标签 | 类型 | 必填 |
|------|------|------|------|
| title | 标题 | text | true |
| slug | 短链 | text | true |
| summary | 摘要 | text | false |
| body | 正文 | textarea | true |
| board | 板块 | text | false |
| board_id | 板块ID | number | false |
| author | 署名 | text | false |

`发帖规则` =

| 字段 | 规则 | 消息 |
|------|------|------|
| title | required | 标题不能为空 |
| slug | required | 需要短链 |
| body | required | 正文不能为空 |

`帖子详情` =

| 属性 | 值 | 样式 |
|------|-----|------|
| title | posts.title | |
| meta | posts.author | |
| body | posts.body | |

`新闻详情` =

| 属性 | 值 | 样式 |
|------|-----|------|
| title | news.title | |
| body | news.body | |

`帖子条件` =

| 字段 | 操作 | 值 |
|------|------|-----|
| slug | = | {slug} |

`新闻条件` =

| 字段 | 操作 | 值 |
|------|------|-----|
| slug | = | {slug} |

`地图项` =

| loc | 更新 |
|-----|------|
| / | 2026-09-10 |
| /post | 2026-09-10 |
| /news | 2026-09-10 |
| /chat | 2026-09-10 |
| /topics | 2026-09-10 |
| /search | 2026-09-10 |
| /guide/automation-api | 2026-09-10 |

`图标` =

| 路径 | 关系 | 类型 | 尺寸 | 地址 |
|------|------|------|------|------|
| "public/img/favicon-32.png" | icon | "image/png" | 32x32 | "/favicon.ico" |

**store = > db.打开**
**头 = > chrome.头资源**
**顶 = > chrome.导航HTML**
**底 = > chrome.页脚HTML**
**鉴权补丁 = "<style>.nav-when-auth{display:none}</style><script>(function(){var c=document.cookie.indexOf('marqdo_sid=')>=0;document.querySelectorAll('.nav-when-guest').forEach(function(el){el.style.display=c?'none':'';});document.querySelectorAll('.nav-when-auth').forEach(function(el){el.style.display=c?'':'none';});})();</script>"**

**主页体 = > home.装配 库=`store`**
**主页文 = > page.包装 正文=`主页体` 体类="page-home"**
**首页 = > 网页.页面 标题="暗恋见君论坛" 引言=`主页文` 壳样式="off" 布局="bare"**
**首页 = > 首页.头装配 表=`头`**

**板块行 = > store.查询 表="boards" 上限=50**
**帖子行 = > store.查询 表="posts" 上限=50 排序="-created_at"**
**帖子栏 = > feed.板块栏HTML 板块=板块行 前缀="/post" 查询键="board_id" 全部类="feed-tab"**
**帖子表 = > feed.帖子列表HTML 行=帖子行**
**帖子列表体 = "<section class=\"feed-list-page\"><div class=\"section-wrap\"><h1 class=\"feed-list-title\">论坛交流</h1>" + 帖子栏 + 帖子表 + "<p style=\"margin-top:1rem\"><a href=\"/post/public\">发布帖子</a></p></div></section>"**
**帖子列表文 = > page.包装 正文=`帖子列表体` 体类="page-post-list"**
**帖子列表 = > 网页.页面 标题="帖子 · 暗恋见君" 引言=`帖子列表文` 壳样式="off" 布局="bare"**
**帖子列表 = > 帖子列表.头装配 表=`头`**

**新闻板块行 = > store.查询 表="news_boards" 上限=50**
**新闻行 = > store.查询 表="news" 上限=50 排序="-created_at"**
**新闻栏 = > feed.板块栏HTML 板块=新闻板块行 前缀="/news" 查询键="news_board_id" 全部类="feed-tab"**
**新闻表 = > feed.新闻时间线HTML 行=新闻行**
**新闻列表体 = "<section class=\"feed-list-page home-news\"><div class=\"section-wrap\"><h1 class=\"feed-list-title\">新闻动态</h1>" + 新闻栏 + 新闻表 + "</div></section>"**
**新闻列表文 = > page.包装 正文=`新闻列表体` 体类="page-news-list"**
**新闻列表 = > 网页.页面 标题="新闻 · 暗恋见君" 引言=`新闻列表文` 壳样式="off" 布局="bare"**
**新闻列表 = > 新闻列表.头装配 表=`头`**

**帖子页 = > 网页.页面 标题="帖子详情" 引言=`鉴权补丁` 壳样式="off" 布局="stacked"**
**帖子页 = > 帖子页.头装配 表=`头`**
**帖子页 = > 帖子页.壳HTML 导航=`顶` 页脚=`底` 体类="page-post-detail"**
**帖子页 = > 帖子页.主体装配 主体=`帖子详情`**
**帖子页 = > 帖子页.查询条件 条件=`帖子条件`**
**帖子页 = > 帖子页.详情 详情=True**

**新闻页 = > 网页.页面 标题="新闻详情" 引言=`鉴权补丁` 壳样式="off" 布局="stacked"**
**新闻页 = > 新闻页.头装配 表=`头`**
**新闻页 = > 新闻页.壳HTML 导航=`顶` 页脚=`底` 体类="page-news-detail"**
**新闻页 = > 新闻页.主体装配 主体=`新闻详情`**
**新闻页 = > 新闻页.查询条件 条件=`新闻条件`**
**新闻页 = > 新闻页.详情 详情=True**

**发帖表 = > 网页.表单 表="posts" 动作="插入"**
**发帖表 = > 发帖表.字段 字段=`发帖字段`**
**发帖表 = > 发帖表.规则 规则=`发帖规则`**
**发帖引言 = > page.包装 正文="<section class=\"feed-list-page\"><div class=\"section-wrap\"><h1>发布帖子</h1><p>需登录（admin / anlian）</p></div></section>" 体类="page-post-public"**
**发帖页 = > 网页.页面 标题="发帖 · 暗恋见君" 引言=`发帖引言` 壳样式="off" 布局="bare"**
**发帖页 = > 发帖页.头装配 表=`头`**
**发帖页 = > 发帖页.表单装配 id="post" 表单=`发帖表`**

**房间行 = > store.查询 表="chat_rooms" 上限=50**
**房间段 = None**
- [房](房间行)
  **名 = [name](房)**
  **链 = [slug](房)**
  **摘 = [summary](房)**
  **项 = "<article class=\"chat-room-card\"><h3><a href=\"/chat/" + 链 + "\">" + 名 + "</a></h3><p>" + 摘 + "</p></article>"**
  **房间段 = > table.append list=房间段 item=项**
**房间内 = > text.str_join xs=房间段 sep=""**
**聊天列表体 = "<section class=\"feed-list-page\"><div class=\"section-wrap\"><h1>聊天室</h1><div class=\"chat-room-grid\">" + 房间内 + "</div></div></section>"**
**聊天列表文 = > page.包装 正文=`聊天列表体` 体类="page-chat"**
**聊天列表 = > 网页.页面 标题="聊天 · 暗恋见君" 引言=`聊天列表文` 壳样式="off" 布局="bare"**
**聊天列表 = > 聊天列表.头装配 表=`头`**

**大厅体 = "<section class=\"feed-list-page\"><div class=\"section-wrap\"><h1>大厅</h1><div class=\"chat-box\"><div id=\"chat-log\" class=\"chat-log\"></div><p class=\"search-bar\"><input id=\"chat-msg\" placeholder=\"说点什么…\" aria-label=\"消息\"><button id=\"chat-send\" type=\"button\">发送</button></p></div><script src=\"/static/chat.js\"></script></div></section>"**
**大厅文 = > page.包装 正文=`大厅体` 体类="page-chat-room"**
**大厅页 = > 网页.页面 标题="大厅 · 暗恋见君" 引言=`大厅文` 壳样式="off" 布局="bare"**
**大厅页 = > 大厅页.头装配 表=`头`**

**闲聊体 = "<section class=\"feed-list-page\"><div class=\"section-wrap\"><h1>闲聊</h1><div class=\"chat-box\"><div id=\"chat-log\" class=\"chat-log\"></div><p class=\"search-bar\"><input id=\"chat-msg\" placeholder=\"说点什么…\" aria-label=\"消息\"><button id=\"chat-send\" type=\"button\">发送</button></p></div><script src=\"/static/chat.js\"></script></div></section>"**
**闲聊文 = > page.包装 正文=`闲聊体` 体类="page-chat-room"**
**闲聊页 = > 网页.页面 标题="闲聊 · 暗恋见君" 引言=`闲聊文` 壳样式="off" 布局="bare"**
**闲聊页 = > 闲聊页.头装配 表=`头`**

**专题行 = > store.查询 表="topics" 上限=50**
**专题段 = None**
- [题](专题行)
  **题名 = [title](题)**
  **链 = [slug](题)**
  **摘 = [summary](题)**
  **项 = "<article class=\"topic-card\"><h3>" + 题名 + "</h3><p>" + 摘 + "</p><p class=\"text-muted\">slug: " + 链 + "</p></article>"**
  **专题段 = > table.append list=专题段 item=项**
**专题内 = > text.str_join xs=专题段 sep=""**
**专题体 = "<section class=\"feed-list-page\"><div class=\"section-wrap\"><h1>专题书架</h1><div class=\"topic-shelf\">" + 专题内 + "</div></div></section>"**
**专题文 = > page.包装 正文=`专题体` 体类="page-topics"**
**专题页 = > 网页.页面 标题="专题 · 暗恋见君" 引言=`专题文` 壳样式="off" 布局="bare"**
**专题页 = > 专题页.头装配 表=`头`**

**搜索体 = "<section class=\"feed-list-page\"><div class=\"section-wrap\"><h1>搜索</h1><form class=\"search-bar\" method=\"get\" action=\"/search\"><input name=\"q\" placeholder=\"关键词\" aria-label=\"关键词\"><button type=\"submit\">搜</button></form><p class=\"text-muted\">全文检索走 SQLite FTS（posts_fts）；结果面板后续可按 q 扩展。</p></div></section>"**
**搜索文 = > page.包装 正文=`搜索体` 体类="page-search"**
**搜索页 = > 网页.页面 标题="搜索 · 暗恋见君" 引言=`搜索文` 壳样式="off" 布局="bare"**
**搜索页 = > 搜索页.头装配 表=`头`**

**指南体 = "<section class=\"feed-list-page\"><div class=\"section-wrap\"><h1>API 自动化指南</h1><p>使用 Bearer / X-Api-Key 调用自动化接口。本验收切片提供 <code>api_keys</code> 表与 <code>web_api_key_check</code> 原语；业务 scopes 写在 .mq.md。</p><pre>Authorization: Bearer &lt;token&gt;</pre></div></section>"**
**指南文 = > page.包装 正文=`指南体` 体类="page-guide"**
**指南页 = > 网页.页面 标题="API 指南 · 暗恋见君" 引言=`指南文` 壳样式="off" 布局="bare"**
**指南页 = > 指南页.头装配 表=`头`**

**应用 = > 网页.应用 页面=`首页` 数据库=`store` 后台=True 主机="127.0.0.1" 端口=18180**
**应用 = > 应用.路由 路径="/post" 页面=`帖子列表`**
**应用 = > 应用.路由 路径="/post/public" 页面=`发帖页`**
**应用 = > 应用.路由 路径="/post/{slug}" 页面=`帖子页`**
**应用 = > 应用.路由 路径="/news" 页面=`新闻列表`**
**应用 = > 应用.路由 路径="/news/{slug}" 页面=`新闻页`**
**应用 = > 应用.路由 路径="/chat" 页面=`聊天列表`**
**应用 = > 应用.路由 路径="/chat/lobby" 页面=`大厅页`**
**应用 = > 应用.路由 路径="/chat/casual" 页面=`闲聊页`**
**应用 = > 应用.路由 路径="/topics" 页面=`专题页`**
**应用 = > 应用.路由 路径="/search" 页面=`搜索页`**
**应用 = > 应用.路由 路径="/guide/automation-api" 页面=`指南页`**
**应用 = > 应用.静态 目录="public" 挂载="/static"**
**应用 = > 应用.图标 表=`图标`**
**应用 = > 应用.鉴权 用户表=`用户表` 会话时长=7200 登录路径="/accounts/login" 登录回跳="/post/public" 登出回跳="/"**
**应用 = > 应用.门禁 路径="/post/public" 角色="admin" 匹配="prefix" 拒绝="redirect" 排除="/accounts/login"**
**应用 = > 应用.路由实时 路径="/chat/ws" 模式="broadcast"**
**应用 = > 应用.调用 路径="/post/ten" 方法="GET" 函数="fragments.帖子十" 正文="query" 返回="html"**
**应用 = > 应用.调用 路径="/news/five" 方法="GET" 函数="fragments.新闻五" 正文="query" 返回="html"**
**应用 = > 应用.站点地图 路径="/sitemap.xml" 基址="http://127.0.0.1:18180" 条目=`地图项`**
**应用 = > 应用.爬虫协议 站点地图="http://127.0.0.1:18180/sitemap.xml"**
> `应用`.监听
