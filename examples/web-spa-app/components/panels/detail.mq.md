---
markup: occlusion
title: Detail panel
import:
  controls: ../controls.mq.md
  component: ext/web/component/index.mq.md
  table: lib/table.mq.md
---

Detail separates its changing text from the reusable Edit and Back action leaves.

## nodes

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Initially hidden detail panel |

The body has id detail-body and receives plain text after selection. Edit retains the selected stable id; Back returns to the list. Neither action's behavior is embedded in markup. The panel starts hidden and the shared client paint function explicitly controls its hidden attribute, using the same ids as the other panels.

We **body = [Describe the selected record's text target](controls.text(id="detail-body"))** and **edit = [Describe the edit action](controls.button(id="btn-edit", label="Edit"))** and **back = [Describe the list action](controls.button(id="btn-back", label="Back"))**.
We **buttons = [Keep the action order](table.concat(a=`edit`, b=`back`))** and **actions = [Group only these detail actions](component.wrap(children=`buttons`, kind="row", slot="detail-actions"))** and **children = [Keep data before actions](table.concat(a=`body`, b=`actions`))**.

`attrs` =

| id | hidden |
|---|---|
| view-detail | True |

Return *[Describe the independent detail panel](component.wrap(children=`children`, kind="section", attrs=`attrs`, slot="detail-panel"))*.
