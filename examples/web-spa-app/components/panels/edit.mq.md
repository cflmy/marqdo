---
markup: occlusion
title: Editor panel
import:
  controls: ../controls.mq.md
  component: ext/web/component/index.mq.md
  table: lib/table.mq.md
---

The editor is a native input plus reusable action leaves. Saving remains a client transition, not an implicit HTTP form.

## nodes

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Initially hidden editor with accessible label and native input |

The label's for attribute names edit-title, the same input read by the Save binding. The input keeps a plain text value. Save validates nonempty text in Marqdo before changing the record list; Cancel changes the route without calling that update. This component describes controls only and starts hidden. The parent keeps field content separate from the action row, allowing the existing responsive layout renderer to place them.

We **label_attrs = [Associate the label with its input](table.put(in=None, at="for", value="edit-title"))** and **label = [Describe the visible field label](component.element(kind="label", value="Title", attrs=`label_attrs`, slot="title-label"))**.

`input_attrs` =

| id | name | type |
|---|---|---|
| edit-title | title | text |

We **input = [Describe the native title input](component.element(kind="input", attrs=`input_attrs`, slot="title-input"))** and **field = [Keep label and input together](table.concat(a=`label`, b=`input`))**.
We **save = [Describe the save action](controls.button(id="btn-save", label="Save"))** and **cancel = [Describe the cancel action](controls.button(id="btn-cancel", label="Cancel"))** and **buttons = [Keep these editor actions ordered](table.concat(a=`save`, b=`cancel`))** and **actions = [Group the action row](component.wrap(children=`buttons`, kind="row", slot="editor-actions"))** and **children = [Place the field before its actions](table.concat(a=`field`, b=`actions`))**.

`attrs` =

| id | hidden |
|---|---|
| view-edit | True |

Return *[Describe the independent editor panel](component.wrap(children=`children`, kind="section", attrs=`attrs`, slot="editor-panel"))*.
