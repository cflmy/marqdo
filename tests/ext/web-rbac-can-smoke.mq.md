---
title: web RBAC smoke
description: app.rbac flag + gate permissions CSV stored on app bag
import web:ext/web/web.mq.md
import json:lib/json.mq.md
---

# main

> print text=rbac-smoke-start

**首页 = > web.page title="home"**
`用户表` =
| 用户名 | 密码 | 角色 |
|--------|------|------|
| admin | secret | admin |

**应用 = > web.app page=`首页` admin=False**
**应用 = > `应用`.enable_rbac**
**authbag = > json.get value=`应用` key="auth"**
**rbac_on = > json.get value=`authbag` key="rbac"**
1. `rbac_on` == True
  > print text=rbac-flag-ok
2. *
  > print text=rbac-flag-fail

**应用 = > `应用`.auth users=`用户表` login_path="/login"**
**authbag2 = > json.get value=`应用` key="auth"**
**rbac_kept = > json.get value=`authbag2` key="rbac"**
1. `rbac_kept` == True
  > print text=rbac-survives-auth-ok
2. *
  > print text=rbac-survives-auth-fail

**应用 = > `应用`.gate path="/desk" permissions="desk:access" match="prefix" on_deny="redirect" exclude="/login"**
**gates = > json.get value=`应用` key="gates"**
**g1 = > at value=`gates` index=1**
**perms = > json.get value=`g1` key="permissions"**
**p0 = > at value=`perms` index=0**
1. `p0` == "desk:access"
  > print text=gate-perm-ok
2. *
  > print text=gate-perm-fail

**应用2 = > web.app page=`首页` admin=False**
**应用2 = > `应用2`.gate path="/desk/roles" permissions="roles:manage" roles="editor" match="exact"**
**gates2 = > json.get value=`应用2` key="gates"**
**g0 = > at value=`gates2` index=0**
**perms2 = > json.get value=`g0` key="permissions"**
**p1 = > at value=`perms2` index=0**
1. `p1` == "roles:manage"
  > print text=gate-roles-manage-ok
2. *
  > print text=gate-roles-manage-fail

> print text=rbac-smoke-done
