---
markup: occlusion
title: SPA navigation
import:
  controls: controls.mq.md
  component: ext/web/component/index.mq.md
  table: lib/table.mq.md
---

Navigation composes reusable native buttons; the client owns their transitions.

## nodes

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Navigation containing Items and New controls |

The two buttons have independent stable ids and shared accessible button semantics. They contain labels only; selecting an item or opening an editor belongs to the client callbacks. Their parent is a nav region, so this file expresses one placement responsibility without duplicating the surrounding page.

We **items = [Describe the list action](controls.button(id="nav-list", label="Items"))** and **create = [Describe the creation action](controls.button(id="nav-new", label="New"))** and **children = [Keep their reading order](table.concat(a=`items`, b=`create`))**.
Return *[Place these actions in navigation](component.wrap(children=`children`, kind="nav", slot="navigation"))*.
