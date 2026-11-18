---
markup: occlusion
title: Request tenant selection for database-backed web resources
import:
  plugin: lib/plugin.mq.md
---

Tenant selection identifies a request context; authentication and membership are separate decisions. Pair this leaf with explicit scoped routes in ext/data/api.mq.md. Use it before listening and retain the returned application after every binding. The implementation reuses the existing native tenant resolver.

## attach

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| app | any | required | Prepared application to receive tenant context |
| mode | text | "path" | path, subdomain or header |
| param | text | "t" | Path capture/query key or header name; use X-Tenant-Id explicitly for header mode |
| column | text | "tenant_id" | Database column used by scoped queries |
| default_scope | bool | False | Native default scoping for supported database-backed request handlers |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Updated application with native request tenant resolution |

Use `app`, `mode`="path", `param`="t", `column`="tenant_id" and `default_scope`=False. Path mode first resolves the named route capture, then its query key, then the documented path segments. Subdomain mode uses the first label of a host with at least three labels. Header mode reads the selected header. The native resolver lowercases and keeps ASCII letters, digits, underscores and hyphens; it is not a general tenant-name codec.

Grant plugin and keep the returned application. A later scoped JSON route rejects a missing resolved tenant and adds the tenant-column filter. The default flag applies only to native handlers that support it; ordinary direct database calls do not gain automatic request context. Tenant ids arriving from query parameters, headers or hostnames must be checked against authenticated membership by the application's actual policy.

The local two-tenant example intentionally allows anonymous selection to demonstrate filtering. That behavior cannot establish production access isolation between users. No database row is changed and no socket opens in this binding. Develop resolver behavior in the native tenant package, and verify both selected tenants plus a request without a selector.

We **native = [Resolve the maintained tenant implementation](plugin.native_path(name="web"))** and **[Load the existing request-context boundary](plugin.load(path=`native`))**.
Return *[Attach the explicit request tenant policy](web_app_tenant(app=`app`, mode=`mode`, param=`param`, column=`column`, default_scope=`default_scope`))*.
