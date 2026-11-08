---
markup: occlusion
title: ext/security/rbac
description: RBAC Resource (ADR 0007).
import:
  plugin: lib/plugin.mq.md
  sys: lib/sys.mq.md
  table: lib/table.mq.md
---

Role and permission data are explicit inputs to the native web checker. The [security entry](index.mq.md) executes allow and deny cases without an identity provider. The can tool is associated with rbac rather than a receiver method because it reads only its arguments. Held wildcard and any-match needed permissions follow the native policy; do not reinterpret that as an all-permissions check.

## _load

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | No business value on completion; a missing native plugin reports the cause and exits this run. |

Resolve and load the native web plugin before resource operations; missing binaries stop this run.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="web"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/security: native web plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.
Return *None*.

# rbac

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Offline helpers for role ⊥ permission. Prefer `app.enable_rbac` + gates for HTTP; use these for scripts/tests. |

Offline helpers for role ⊥ permission. Prefer `app.enable_rbac` + gates for HTTP; use these for scripts/tests.

## can

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| permissions | any | required | Comma-separated permission names held by the current role or subject |
| needed | any | required | Permission names required by the operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Return `{ok, allowed}` — whether held permission CSV includes any of `needed` (CSV). Held `*` allows all. |

Use `permissions`, `needed` as the inputs for this operation.

Return `{ok, allowed}` — whether held permission CSV includes any of `needed` (CSV). Held `*` allows all.

We **[Load the native plugin required by this operation](_load())**.
Return *[Return {ok, allowed} — whether held permission CSV includes any of needed (CSV). Held * allows all.](web_rbac_can(permissions=`permissions`, needed=`needed`))*.

## assign_role

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| url | any | required | Resource connection URL; scheme selects the documented backend |
| username | any | required | User name in the configured user store |
| role | any | required | Role name within the selected user store |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Bind `username` (in `web_users`) to role name. |

Use `url`, `username`, `role` as the inputs for this operation.

Bind `username` (in `web_users`) to role name.

We **[Load the native plugin required by this operation](_load())**.
Return *[Bind username (in web_users) to role name.](web_rbac_assign_role(url=`url`, username=`username`, role=`role`))*.

## create_role

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| url | any | required | Resource connection URL; scheme selects the documented backend |
| name | any | required | Name registered or looked up by this operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Create a non-system role; returns `{ok, id, name}`. |

Use `url`, `name` as the inputs for this operation.

Create a non-system role; returns `{ok, id, name}`.

We **[Load the native plugin required by this operation](_load())**.
Return *[Create a non-system role; returns {ok, id, name}.](web_rbac_create_role(url=`url`, name=`name`))*.

## set_role_permissions

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| url | any | required | Resource connection URL; scheme selects the documented backend |
| role | any | required | Role name within the selected user store |
| permissions | any | required | Comma-separated permission names held by the current role or subject |
| grantable | any | "" | Permission catalog limiting which permissions may be granted |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Replace permissions on a non-system role (CSV). `grantable` restricts grants (anti-escalation); empty = full catalog. |

Use `url`, `role`, `permissions`, `grantable`="" as the inputs for this operation.

Replace permissions on a non-system role (CSV). `grantable` restricts grants (anti-escalation); empty = full catalog.

We **[Load the native plugin required by this operation](_load())**.
Return *[Replace permissions on a non-system role (CSV). grantable restricts grants (anti-escalation); empty = full catalog.](web_rbac_set_role_permissions(url=`url`, role=`role`, permissions=`permissions`, grantable=`grantable`))*.
