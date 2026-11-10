---
markup: occlusion
title: List panel
import:
  component: ext/web/component/index.mq.md
  table: lib/table.mq.md
---

The existing list container is populated by the official bridge's structured render_list effect.

## nodes

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Initially visible list panel with its Markdown hint |

The empty native list provides id items for delegated clicks. Its actual entries come from native client records and use stable data-id attributes. The hint is ordinary Markdown and knows nothing about item data or routing. Only this parent sets the list panel's visibility; detail and editor remain separate components.

We **empty = [Start with no server-owned item rows](table.empty_list())** and **target = [Name the client-owned list container](table.put(in=None, at="id", value="items"))** and **list = [Describe the native unordered list](component.wrap(children=`empty`, kind="list", attrs=`target`, slot="items"))**.
We **hint = [Keep the instruction as ordinary Markdown](component.markdown(text="Click an item for detail.", slot="list-hint"))** and **children = [Place the hint after its list](table.concat(a=`list`, b=`hint`))**.

`attrs` =

| id | hidden |
|---|---|
| view-list | False |

Return *[Describe the visible list panel](component.wrap(children=`children`, kind="section", attrs=`attrs`, slot="list-panel"))*.
