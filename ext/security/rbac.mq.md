---
title: ext/security/rbac
description: RBAC Resource (ADR 0007).
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
import table:lib/table.mq.md
---

## _load

**p = > plugin.native_path name="web"**
1. `p`
  > plugin.load path=`p`
2. *
  > print text=ext/security: native web plugin not found
  > sys.exit code=1
****

# rbac

Offline helpers for role ⊥ permission. Prefer `app.enable_rbac` + gates for HTTP; use these for scripts/tests.

## can
    + `permissions`
    + `needed`

Return `{ok, allowed}` — whether held permission CSV includes any of `needed` (CSV). Held `*` allows all.

> _load
*> web_rbac_can permissions=`permissions` needed=`needed`*

## assign_role
    + `url`
    + `username`
    + `role`

Bind `username` (in `web_users`) to role name.

> _load
*> web_rbac_assign_role url=`url` username=`username` role=`role`*

## create_role
    + `url`
    + `name`

Create a non-system role; returns `{ok, id, name}`.

> _load
*> web_rbac_create_role url=`url` name=`name`*

## set_role_permissions
    + `url`
    + `role`
    + `permissions`
    + `grantable`=""

Replace permissions on a non-system role (CSV). `grantable` restricts grants (anti-escalation); empty = full catalog.

> _load
*> web_rbac_set_role_permissions url=`url` role=`role` permissions=`permissions` grantable=`grantable`*

