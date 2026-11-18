---
markup: occlusion
title: Native collaborative editor leaf
import:
  component: ext/web/component/index.mq.md
  table: lib/table.mq.md
---

This intrinsic label/textarea pair contains no socket, page shell or generated HTML.

## nodes

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Accessible native multiline editor rows |

The label names the same doc id used by the client input binding. Rows and columns are typed native attributes rather than interpolated markup. Initial content is empty and ordinary text; the bridge later changes its value. Geometry outside this intrinsic field pair belongs to the parent page component.

We **label_attrs = [Associate the visible label with the editor](table.put(in=None, at="for", value="doc"))** and **label = [Describe the plain editor label](component.element(kind="label", value="Shared text", attrs=`label_attrs`, slot="editor-label"))**.

`attrs` =

| id | rows | cols | aria-label |
|---|---|---|---|
| doc | 16 | 72 | Shared text |

We **editor = [Describe the native multiline control](component.element(kind="textarea", attrs=`attrs`, slot="editor"))**.
Return *[Keep the field label before its native control](table.concat(a=`label`, b=`editor`))*.
