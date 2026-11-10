---
markup: occlusion
title: Three browser panel destinations
import:
  component: ext/web/component/index.mq.md
  table: lib/table.mq.md
---

## nodes

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Structured View rows for this component |

Navigation contains three ordinary links with stable ids. Their fragment destinations leave a readable fallback without issuing a server page request. The official wire table intercepts each click and requests one deliberate history entry. Back and Forward only repaint the already selected panel. This component does not own client state or generate navigation JavaScript.

`destinations` =

| @ | id | href | label |
|---|---|---|---|
| 1 | nav-home | "#/" | 首页 |
| 2 | nav-about | "#/about" | 关于 |
| 3 | nav-net | "#/net" | 网络 |

We **links = [Start the ordered native navigation](table.empty_list())**.
- [Declare each independently named destination](for destinations as destination)
  We **label = [Read its literal label](access(destination, "label"))** and **id = [Read its stable event target](access(destination, "id"))** and **href = [Read its fallback destination](access(destination, "href"))**.

  `attrs` =

  | id | href |
  |---|---|
  | `id` | `href` |

  We **link = [Describe this native destination](component.element(kind="link", value=`label`, attrs=`attrs`))** and **links = [Preserve navigation order](table.concat(a=`links`, b=`link`))**.
Return *[Place those links in a navigation region](component.wrap(children=`links`, kind="nav", slot="navigation"))*.
