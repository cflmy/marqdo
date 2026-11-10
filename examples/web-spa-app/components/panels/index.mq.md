---
markup: occlusion
title: SPA panel assembly
import:
  listing: list.mq.md
  detail: detail.mq.md
  editor: edit.mq.md
  table: lib/table.mq.md
---

This parent owns only the three sibling panel relationships.

## nodes

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Ordered list, detail and editor panel rows |

Each child owns one panel and names its own native controls. This function retains their structured values and stable order; the client independently selects visibility using hidden attributes. It does not render a second page or concatenate HTML. Changes to editor fields therefore remain inside the editor component instead of growing a single application string.

We **list = [Assemble the listing component](listing.nodes())** and **view = [Assemble the detail component](detail.nodes())** and **edit = [Assemble the editor component](editor.nodes())** and **first = [Retain list and detail order](table.concat(a=`list`, b=`view`))**.
Return *[Keep the editor as the third sibling](table.concat(a=`first`, b=`edit`))*.
