---
markup: occlusion
title: Security resources — authentication, roles and OIDC
import:
  permissions_api: ext/security/rbac.mq.md
---

Use [auth](auth.mq.md) for credential/session operations, [rbac](rbac.mq.md) for role permissions,
and [OIDC](oidc.mq.md) for attaching an external identity provider to an existing web app.
Endpoint authorization remains explicit in the web artifact metadata; a script permission check
does not automatically protect an HTTP route.

The resources use the native web plugin. Build it with scripts/build-web-plugin.sh in this checkout.
This entry's deterministic permission check does not contact an identity provider or change users:

```sh
marqdo call ext/security/index.mq.md check_permission 'held="notes:read"' 'needed="notes:read"' --effects plugin --json
```

Constructors create receivers that are used by resource methods. The free function below makes
that pattern available to CLI callers without copying opaque object fields. For OIDC, configure
registered callback URLs and origin restrictions in oidc.mq.md; never copy production credentials
into a source example. Transport success and authorization success are distinct fields.

Develop native security policy in the existing web security implementation, then document the
public contract and exercise both allow and deny cases. Keep permission delimiters, wildcard
behavior, session lifetime and credential handling explicit. Use local fixtures for login and
OIDC tests; the reusable source must not require a live identity service merely to be inspected.

## check_permission

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| held | text | required | Comma-separated permissions held by the subject; wildcard follows the native RBAC policy |
| needed | text | required | Comma-separated required permissions interpreted by the documented RBAC checker |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Native ok/allowed record; inspect allowed to decide authorization |

The inputs `held` and `needed` describe one explicit permission decision, not a role assignment
or a request to modify the user database. The native checker currently accepts any matching
required permission; held wildcard permits all. Use the leaf API when changing a role catalog.

The checker is an associated tool: it does not read self, so this workflow calls its definition
path directly rather than pretending it is an instance method. We return *[Evaluate this subject against the requested permission set](permissions_api.rbac.can(permissions=`held`, needed=`needed`))* .
The returned allowed field is the authorization decision; the outer CLI ok only reports whether
this function call completed. A denied decision remains a valid, observable result.
