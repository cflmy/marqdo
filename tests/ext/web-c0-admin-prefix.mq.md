---
title: web C0 admin prefix / route release
description: admin=False frees /admin; custom admin_prefix reserves only that mount.
import web:ext/web/web.mq.md
import json:lib/json.mq.md
import sys:lib/sys.mq.md
---

# main

*首页 = > web.page title="home"*
*页 = > web.page title="news"*

*应用 = > web.app page=`首页` admin=False*
*应用 = > `应用`.route path="/admin/news" page=`页`*
*routes = > json.get value=`应用` key="routes"*
*hit = > json.get value=`routes` key="/admin/news"*
1. `hit`
  > print text=free-admin-ok
2. *
  > print text=free-admin-fail
  > sys.exit code=1

*应用3 = > web.app page=`首页` admin=True admin_prefix="/desk"*
*应用3 = > `应用3`.route path="/admin/news" page=`页`*
*routes3 = > json.get value=`应用3` key="routes"*
*hit3 = > json.get value=`routes3` key="/admin/news"*
1. `hit3`
  > print text=prefix-frees-admin-ok
2. *
  > print text=prefix-frees-admin-fail
  > sys.exit code=1

*prefix = > json.get value=`应用3` key="admin_prefix"*
1. `prefix` == "/desk"
  > print text=prefix-stored-ok
2. *
  > print text=prefix-stored-fail
  > sys.exit code=1

`用户表` =
| 用户名 | 密码 | 角色 |
|--------|------|------|
| admin | secret | admin |

*应用4 = > web.app page=`首页` admin=True*
*应用4 = > `应用4`.auth users=`用户表` admin_prefix="/desk" login_redirect="/desk" logout_redirect="/desk/login"*
*prefix4 = > json.get value=`应用4` key="admin_prefix"*
*login4 = > json.get value=`应用4` key="login_redirect"*
*logout4 = > json.get value=`应用4` key="logout_redirect"*
*gates = > json.get value=`应用4` key="gates"*
*g0 = > at value=`gates` index=0*
*gpath = > json.get value=`g0` key="path"*
1. `prefix4` == "/desk"
  1. `login4` == "/desk"
    1. `logout4` == "/desk/login"
      1. `gpath` == "/desk"
        > print text=auth-prefix-ok
      2. *
        > print text=auth-gate-fail
        > sys.exit code=1
    2. *
      > print text=auth-logout-fail
      > sys.exit code=1
  2. *
    > print text=auth-login-fail
    > sys.exit code=1
2. *
  > print text=auth-prefix-fail
  > sys.exit code=1

> print text=c0-admin-ok
