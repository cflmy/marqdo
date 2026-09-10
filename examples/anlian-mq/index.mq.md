---
title: 暗恋见君 · Marqdo
description: 用表格装配的论坛验收站点：帖子、新闻、发帖门禁、公聊、站长工具。
导入 网页:ext/web/网页.mq.md
import shell:styles/shell.mq.md
import nav:components/nav.mq.md
import foot:components/foot.mq.md
import hero:components/hero.mq.md
import db:db/index.mq.md
---

# main

站点壳：导航 + 页脚。

`壳` =

| 组件 | 样式 |
|------|------|
| nav.`导航` | |
| foot.`页脚` | |

帖子列表卡片绑定（字段名对应库表列；`href` 走 slug）。

`帖子卡片` =

| 属性 | 值 | 样式 |
|------|-----|------|
| title | posts.title | |
| body | posts.summary | |
| meta | posts.board | |
| href | posts.slug | |

`帖子详情` =

| 属性 | 值 | 样式 |
|------|-----|------|
| title | posts.title | |
| meta | posts.author | |
| body | posts.body | |

`新闻卡片` =

| 属性 | 值 | 样式 |
|------|-----|------|
| title | news.title | |
| body | news.summary | |
| href | news.slug | |

`新闻详情` =

| 属性 | 值 | 样式 |
|------|-----|------|
| title | news.title | |
| body | news.body | |

动态路由条件：`{slug}` 由路径注入。

`帖子条件` =

| 字段 | 操作 | 值 |
|------|------|-----|
| slug | = | {slug} |

`新闻条件` =

| 字段 | 操作 | 值 |
|------|------|-----|
| slug | = | {slug} |

发帖表单（字段 / 规则均为表）。

`发帖字段` =

| 字段 | 标签 | 类型 | 必填 |
|------|------|------|------|
| title | 标题 | text | true |
| slug | 短链 | text | true |
| summary | 摘要 | text | false |
| body | 正文 | textarea | true |
| board | 板块 | text | false |
| author | 署名 | text | false |

`发帖规则` =

| 字段 | 规则 | 消息 |
|------|------|------|
| title | required | 标题不能为空 |
| title | max:200 | 标题太长 |
| slug | required | 需要短链 |
| body | required | 正文不能为空 |
| body | max:20000 | 正文太长 |

站务账号（明文仅用于本地验收；生产请用 `网页.密码哈希`）。

`用户表` =

| 行 | 用户名 | 密码 | 角色 |
|----|--------|------|------|
| 1 | admin | anlian | admin |

站点地图条目。

`地图项` =

| loc | 更新 |
|-----|------|
| / | 2026-09-10 |
| /posts | 2026-09-10 |
| /news | 2026-09-10 |
| /chat | 2026-09-10 |
| /search | 2026-09-10 |

`图标` =

| 路径 | 关系 | 类型 | 尺寸 | 地址 |
|------|------|------|------|------|
| "public/favicon-32.png" | icon | "image/png" | 32x32 | "/favicon.ico" |

*store = > db.打开*
*壳CSS = > shell.全局*
*首页引言 = > hero.首页引言*

*首页 = > 网页.页面 标题="暗恋见君" 引言=`首页引言`*
*首页 = > 首页.组件装配 组件=`壳`*
*首页 = > 首页.主体装配 主体=`帖子卡片`*
*首页 = > 首页.排序 排序="-created_at"*
*首页 = > 首页.链接前缀 前缀="/post/"*
*首页 = > 首页.样式 样式=`壳CSS`*

*帖子列表 = > 网页.页面 标题="帖子" 引言="<div class=\"page-stage\"><h1>帖子</h1><p>粉色毛玻璃卡片 · 对照原站 feed</p></div>"*
*帖子列表 = > 帖子列表.组件装配 组件=`壳`*
*帖子列表 = > 帖子列表.主体装配 主体=`帖子卡片`*
*帖子列表 = > 帖子列表.链接前缀 前缀="/post/"*
*帖子列表 = > 帖子列表.排序 排序="-created_at"*
*帖子列表 = > 帖子列表.样式 样式=`壳CSS`*

*帖子页 = > 网页.页面 标题="帖子详情"*
*帖子页 = > 帖子页.组件装配 组件=`壳`*
*帖子页 = > 帖子页.主体装配 主体=`帖子详情`*
*帖子页 = > 帖子页.查询条件 条件=`帖子条件`*
*帖子页 = > 帖子页.详情 详情=True*
*帖子页 = > 帖子页.样式 样式=`壳CSS`*

*新闻列表 = > 网页.页面 标题="新闻" 引言="<div class=\"page-stage\"><h1>新闻</h1><p>站务与迁移动态</p></div>"*
*新闻列表 = > 新闻列表.组件装配 组件=`壳`*
*新闻列表 = > 新闻列表.主体装配 主体=`新闻卡片`*
*新闻列表 = > 新闻列表.链接前缀 前缀="/news/"*
*新闻列表 = > 新闻列表.排序 排序="-created_at"*
*新闻列表 = > 新闻列表.样式 样式=`壳CSS`*

*新闻页 = > 网页.页面 标题="新闻详情"*
*新闻页 = > 新闻页.组件装配 组件=`壳`*
*新闻页 = > 新闻页.主体装配 主体=`新闻详情`*
*新闻页 = > 新闻页.查询条件 条件=`新闻条件`*
*新闻页 = > 新闻页.详情 详情=True*
*新闻页 = > 新闻页.样式 样式=`壳CSS`*

*发帖表 = > 网页.表单 表="posts" 动作="插入"*
*发帖表 = > 发帖表.字段 字段=`发帖字段`*
*发帖表 = > 发帖表.规则 规则=`发帖规则`*

*发帖页 = > 网页.页面 标题="发帖" 引言="<div class=\"page-stage\"><h1>发帖</h1><p>需登录（admin / anlian）</p></div>"*
*发帖页 = > 发帖页.组件装配 组件=`壳`*
*发帖页 = > 发帖页.表单装配 id="post" 表单=`发帖表`*
*发帖页 = > 发帖页.样式 样式=`壳CSS`*

*搜索页 = > 网页.页面 标题="搜索" 引言="<div class=\"page-stage\"><h1>搜索</h1><p>全文检索入口（SQLite FTS）</p><form class=\"search-bar\" method=\"get\" action=\"/search\"><input name=\"q\" placeholder=\"关键词\" aria-label=\"关键词\"><button type=\"submit\">搜</button></form></div>"*
*搜索页 = > 搜索页.组件装配 组件=`壳`*
*搜索页 = > 搜索页.样式 样式=`壳CSS`*

*聊天页 = > 网页.页面 标题="公聊" 引言="<div class=\"page-stage\"><h1>公聊室</h1><p>WebSocket 广播 · /chat/ws</p></div><div class=\"chat-box\"><div id=\"chat-log\" class=\"chat-log\"></div><p class=\"search-bar\"><input id=\"chat-msg\" placeholder=\"说点什么…\" aria-label=\"消息\"><button id=\"chat-send\" type=\"button\">发送</button></p></div><script src=\"/static/chat.js\"></script>"*
*聊天页 = > 聊天页.组件装配 组件=`壳`*
*聊天页 = > 聊天页.样式 样式=`壳CSS`*

*应用 = > 网页.应用 页面=`首页` 数据库=`store` 后台=True 主机="127.0.0.1" 端口=18180*
*应用 = > 应用.路由 路径="/posts" 页面=`帖子列表`*
*应用 = > 应用.路由 路径="/post/{slug}" 页面=`帖子页`*
*应用 = > 应用.路由 路径="/news" 页面=`新闻列表`*
*应用 = > 应用.路由 路径="/news/{slug}" 页面=`新闻页`*
*应用 = > 应用.路由 路径="/write" 页面=`发帖页`*
*应用 = > 应用.路由 路径="/search" 页面=`搜索页`*
*应用 = > 应用.路由 路径="/chat" 页面=`聊天页`*
*应用 = > 应用.静态 目录="public" 挂载="/static"*
*应用 = > 应用.图标 表=`图标`*
*应用 = > 应用.鉴权 用户表=`用户表` 会话时长=7200 登录回跳="/write" 登出回跳="/"*
*应用 = > 应用.门禁 路径="/write" 角色="admin" 匹配="prefix" 拒绝="redirect" 排除="/admin/login"*
*应用 = > 应用.路由实时 路径="/chat/ws" 模式="broadcast"*
*应用 = > 应用.站点地图 路径="/sitemap.xml" 基址="http://127.0.0.1:18180" 条目=`地图项`*
*应用 = > 应用.爬虫协议 站点地图="http://127.0.0.1:18180/sitemap.xml"*
> `应用`.监听
