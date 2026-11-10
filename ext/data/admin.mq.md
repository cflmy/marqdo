---
markup: occlusion
title: Native administration resource
import:
  table: lib/table.mq.md
---

This single-purpose resource enables the existing native administration surface on a prepared app. It does not construct a page or configure unrelated middleware. Attach real authentication afterwards so the native runtime adds its administration gate before listening. Custom Document desks remain ordinary declared routes and do not need this switch.

## enable

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| app | map | required | Prepared application retaining its native admin_prefix, normally /admin |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Updated app with native administration enabled; no listener starts |

Use `app` before attaching authentication and before listen. This pure resource composition retains the app's database, discovered Documents and other bindings, and changes only the existing native admin flag. The runtime lists only the business tables associated with explicitly mounted forms, shows up to 100 current rows per table, and links to those actual form resources. It does not enumerate unrelated credential/session tables or expose arbitrary SQL. The native renderer owns this landing page and its reserved route namespace. It does not create a second application configuration API.

Authentication must be configured separately. When it is attached after enabling this surface, the existing native admin policy installs the default administration gate. The caller keeps the returned application and verifies both denied and accepted requests. Enabling a UI is not proof that credentials or database access have been authorized; the explicit server workflow supplies those dependencies.

Return *[Enable only the native administration surface](table.put(in=`app`, at="admin", value=True))*.
