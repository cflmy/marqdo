---
title: anlian-mq offline smoke
description: 不 listen：装配库表/页面/门禁/WS 路由袋，验证可读路径可构造。
导入 网页:ext/web/网页.mq.md
import shell:styles/shell.mq.md
import nav:components/nav.mq.md
import foot:components/foot.mq.md
import db:db/index.mq.md
import sys:lib/sys.mq.md
---

# main

`壳` =

| 组件 | 样式 |
|------|------|
| nav.`导航` | |
| foot.`页脚` | |

`帖子卡片` =

| 属性 | 值 |
|------|-----|
| title | posts.title |
| body | posts.summary |
| href | posts.slug |

`用户表` =

| 行 | 用户名 | 密码 | 角色 |
|----|--------|------|------|
| 1 | admin | anlian | admin |

**store = > db.打开**
**壳CSS = > shell.全局**
**首页 = > 网页.页面 标题="暗恋见君" 引言="<h1>ok</h1>"**
**首页 = > 首页.组件装配 组件=`壳`**
**首页 = > 首页.主体装配 主体=`帖子卡片`**
**首页 = > 首页.样式 样式=`壳CSS`**

**应用 = > 网页.应用 页面=`首页` 数据库=`store` 后台=True 主机="127.0.0.1" 端口=18180**
**应用 = > 应用.鉴权 用户表=`用户表`**
**应用 = > 应用.门禁 路径="/write" 角色="admin" 匹配="prefix" 拒绝="redirect"**
**应用 = > 应用.路由实时 路径="/chat/ws" 模式="broadcast"**

**gates = 应用[^gates]**
**n = > len value=`gates`**
1. `n` >= 2
  > print text=gates-ok
2. *
  > print text=gates-fail
  > sys.exit code=1

**ws = 应用[^ws_routes]**
**room = ws[^/chat/ws]**
**mode = room[^mode]**
1. `mode` == "broadcast"
  > print text=ws-ok
2. *
  > print text=ws-fail
  > sys.exit code=1

**list = > store.查询 表="posts" 上限=10**
**nr = > len value=`list`**
1. `nr` >= 1
  > print text=seed-ok
2. *
  > print text=seed-fail
  > sys.exit code=1

> print text=anlian-offline-ok
