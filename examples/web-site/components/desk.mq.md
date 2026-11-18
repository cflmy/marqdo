---
markup: occlusion
title: Administration navigation and footer
import:
  component: ext/web/component/index.mq.md
  fs: lib/fs.mq.md
---

The custom desk keeps its own brand and closing links, and omits the public side rail. Authentication remains the serving entry's responsibility. Neither constructing these nodes nor hiding their links can authorize a visitor to the administration route.

## nav

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Desk navigation using the real current session |

Construction needs no effects. The first link preserves Admin as the operator's current context, followed by Site, New and Logout. Only the logout link is conditional, matching the original navigation. The entire desk is separately gated, and the actual logout destination remains a native session route. Returned nodes keep their structured resource value so the parent can place them without HTML strings.

`items` =

| @ | label | href | when |
|---|---|---|---|
| 1 | Admin | /admin | |
| 2 | Site | / | |
| 3 | New | /new | |
| 4 | Logout | /admin/logout | auth |

We **resource = [Describe the desk's current navigation](component.navigation(items=`items`))**.
Return *[Keep the same navigation fragment slot](component.resource(value=`resource`, slot="nav"))*.

## foot

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Desk footer with terminal Markdown return and logout links |

Read this leaf with fs.read under the site root. The route gate has already decided access before the renderer resolves this component. Site and Admin retain their return destinations, while Logout ends the session only when followed. The footer has one content responsibility and no internal layout to split further; the parent owns its placement and the original theme owns its appearance.

We **text = [Read the desk's terminal closing links](fs.read_text(path="components/content/desk-footer.md"))** and **leaf = [Retain ordinary Markdown content](component.markdown(text=`text`, slot="footer-content"))**.
Return *[Place it in the semantic desk footer](component.wrap(children=`leaf`, kind="footer", slot="footer", style="foot"))*.
