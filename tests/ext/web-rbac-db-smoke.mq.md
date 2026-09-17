---
title: web RBAC db register/assign smoke
description: EnsureSchema path via create_role + assign + can
import web:ext/web/web.mq.md
import json:lib/json.mq.md
import sys:lib/sys.mq.md
---

# main

> print text=rbac-db-start

**db = > web.db url="sqlite::memory:"**
**url = > json.get value=`db` key="url"**
**rbac = > web.rbac**

**seed = > `rbac`.create_role url=`db` name="reviewer"**
**ok1 = > json.get value=`seed` key="ok"**
1. `ok1` == True
  > print text=create-role-ok
2. *
  > print text=create-role-fail
  > sys.exit code=1

**setp = > `rbac`.set_role_permissions url=`db` role="reviewer" permissions="comments:create,desk:access"**
**ok2 = > json.get value=`setp` key="ok"**
1. `ok2` == True
  > print text=set-perms-ok
2. *
  > print text=set-perms-fail
  > sys.exit code=1

**can = > `rbac`.can permissions="comments:create,desk:access" needed="desk:access"**
**allowed = > json.get value=`can` key="allowed"**
1. `allowed` == True
  > print text=can-ok
2. *
  > print text=can-fail
  > sys.exit code=1

**deny = > `rbac`.can permissions="comments:create" needed="roles:manage"**
**denied = > json.get value=`deny` key="allowed"**
1. `denied` == False
  > print text=can-deny-ok
2. *
  > print text=can-deny-fail
  > sys.exit code=1

> print text=rbac-db-done
