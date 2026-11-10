---
markup: occlusion
title: Read-only JSON query routes for a database-backed site
import:
  plugin: lib/plugin.mq.md
---

Use this leaf when a prepared web application needs native read-only database JSON routes. Database connections remain owned by db.mq.md; request tenant selection remains owned by the security tenant resource. This module only binds an explicit query table before listening. It does not introduce a second application configuration language.

## mount

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| app | any | required | Prepared application with its database resource |
| routes | any | required | GFM rows with path, method, table, order, limit and optional tenant_scope |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Updated app containing the selected JSON query route set |

Use `app` and `routes` with plugin. The table uses site-relative paths, database table names, a native order expression and row limits. This binding replaces the application's JSON query route set; supply all desired query routes in one readable table. Other middleware settings are retained. Keep the returned app for subsequent resource bindings and listen.

Set tenant_scope to True only together with tenant.attach from ext/security/tenant.mq.md. With tenant resolution enabled, a scoped query requires a resolved tenant and adds an equality filter on its configured column. Tenant selectors do not authenticate users or prove membership. Protect private data through real authentication and trusted tenant selection; a query-string demonstration deliberately permits choosing either demo tenant.

The native route surface selects rows and returns a JSON object with a rows list; it does not implement arbitrary SQL, write endpoints or form submission. Reading this source grants no network access and mounting does not listen. Extend native query behavior in the existing web middleware implementation, keep database policy visible in the route table, and test missing-tenant as well as cross-tenant responses.

We **native = [Resolve the maintained database/web boundary](plugin.native_path(name="web"))** and **[Load its existing route implementation](plugin.load(path=`native`))**.
Return *[Bind this explicit JSON query route table](web_app_middleware(app=`app`, cors=None, security=None, compress=None, body_limit=None, json_routes=`routes`, access_log=None, cache_control=None, proxy=None, invoke=None))*.
