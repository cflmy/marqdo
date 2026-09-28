---
title: ext/security rbac smoke
import rbac:ext/security/rbac.mq.md
---

# main

**gate = > rbac.rbac**
**r = > `gate`.can permissions="desk:access,posts:read" needed="desk:access"**
**ok = [ok](r)**
**allowed = [allowed](r)**
1. `ok`
  1. `allowed`
    > print text=rbac-ok
  2. *
    > print text=rbac-fail
2. *
  > print text=rbac-fail
