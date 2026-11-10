---
markup: occlusion
type: web
view: main
route: /
title: Tenant SaaS — explicit database and request resources
import:
  component: ext/web/component/index.mq.md
  fs: lib/fs.mq.md
  web: ext/web/web.mq.md
  api: ext/data/api.mq.md
  tenant: ext/security/tenant.mq.md
  database: db/index.mq.md
---

Read the Markdown leaf for the visible demonstration, the database entry for initialization, and the two resource leaves for query and tenant binding. The old startup DELETE is removed: restarting a demo must preserve existing rows. This loopback site intentionally allows choosing a demo tenant; it does not claim authenticated membership.

## main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Native View nodes containing the Markdown explanation and query links |

The page is a single content leaf because its internal structure is already simple Markdown. No raw HTML shell or browser script is needed. Site discovery reads it relative to the chosen site root; the returned node data is assembled before any listener starts. The API and database lifecycle remain in serve, so rendering the explanation cannot clear or recreate business data.

We **intro = [Read the terminal content through the granted site root](fs.read_text(path="intro.md"))**.
Return *[Describe the explanation as a Markdown leaf](component.markdown(text=`intro`, slot="tenant-introduction"))*.

## serve

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| root | text | "examples/web-tenant-saas" | Explicit site discovery directory relative to the checkout |
| port | number | 18092 | Local listener port |
| url | any | None | None selects the site-root database; tests pass a disposable SQLite URI |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Blocking listener with explicit tenant-filtered JSON queries |

Use `root`="examples/web-tenant-saas", `port`=18092 and `url`=None with plugin, fs.write and network.write. Set --fs-root to this site's directory so default database files belong to the site. The discovery root is explicit and independent of that filesystem grant. Preparation initializes retained records, discovers the explanation, then binds request tenant selection and the read-only query table before listening.

The route explicitly scopes its query by tenant_id. With the resolver attached, missing tenant context is rejected rather than returning both tenants. Selection via t is intentionally open for this local example; real user access requires a separate authentication and membership policy. Keep both bindings together when editing the flow.

We **store = [Initialize without deleting existing user rows](database.open(url=`url`))** and **app = [Discover the page with its retained database resource](web.prepare(root=`root`, host="127.0.0.1", port=`port`, db=`store`))** and **app = [Resolve the explicit request tenant selector](tenant.attach(app=`app`, mode="path", param="t", default_scope=False))**.

`routes` =

| path | method | table | order | limit | tenant_scope |
|---|---|---|---|---|---|
| api/items | GET | items | id | 50 | True |

We **app = [Bind the explicit tenant-scoped read-only query](api.mount(app=`app`, routes=`routes`))**.
Return *[Listen after data and request policy are ready](web.listen(app=`app`))*.
