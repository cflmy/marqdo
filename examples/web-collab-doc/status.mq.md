---
markup: occlusion
title: Presence and transport status leaves
import:
  component: ext/web/component/index.mq.md
  table: lib/table.mq.md
---

Presence information and connection diagnostics have separate native text targets.

## nodes

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Ordered presence and log leaves |

Presence frames update one plain-text region, while connection lifecycle messages update a preformatted log. Their stable ids match client callbacks, and neither contains document text or performs transport operations. The parent places these leaves around the independent editor; messages are escaped by the native renderer and later set_text effects.

`presence_attrs` =

| id | role | aria-live |
|---|---|---|
| presence | status | polite |

`log_attrs` =

| id | role |
|---|---|
| log | log |

We **presence = [Describe the plain presence target](component.element(kind="text", value="Waiting for peers.", attrs=`presence_attrs`, slot="presence"))** and **log = [Describe connection feedback separately](component.element(kind="pre", value="connecting…", attrs=`log_attrs`, slot="log"))**.
Return *[Keep presence before transport feedback](table.concat(a=`presence`, b=`log`))*.
