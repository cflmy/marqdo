---
title: web-auth-custom-page
description: Custom branded login owns GET /login; POST failure re-renders same shell with CSRF; no author JS.
import 网页:ext/web/网页.mq.md
---

# main

`用户` =

| 行 | 用户名 | 密码 |
|----|--------|------|
| 1 | alice | secret12 |

**page = > 网页.页面 标题="Home" 引言="<p>home</p>"**
**login = > 网页.页面 标题="登录" 引言="<div class='panel'><h1>登录</h1><div id='auth-mount'></div></div>"**
**login = > login.鉴权表单 动作="/login" 提交="登录" 表单id="login-form" 错误id="login-err" 表单插槽="#auth-mount" 种类="login"**

**db = > 网页.数据库**
**app = > 网页.应用 页面=page 数据库=db 后台=假 主机="127.0.0.1" 端口=0**
**app = > app.路由 路径="/login" 页面=login**
**app = > app.鉴权 用户表=`用户` 会话时长=3600 登录路径="/login" 登录回跳="/"**

> print text=web-auth-custom-page ok
*None*
