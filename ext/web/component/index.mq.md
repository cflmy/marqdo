---
markup: occlusion
title: ext/web/component
description: Pure components: Markdown leaves and nested View rows.
import:
  table: lib/table.mq.md
---

Components are ordinary functions returning View rows. A leaf contains Markdown;
a parent names its child components and one layout relationship. Stop splitting
when one file expresses one readable content responsibility. If a footer has a
brand area and a link area, split those; if links has two columns, split again.
The executable reference is [web-layout-handbook](../../../examples/web-layout-handbook/index.mq.md).

Pure construction experiment — no native plugin or session is required:

```sh
marqdo call ext/web/component/index.mq.md markdown 'text="## A readable leaf"' 'slot="intro"' --json
marqdo call ext/web/component/index.mq.md wrap 'children=[{"type":"markdown","value":"Left"},{"type":"markdown","value":"Right"}]' 'kind="row"' --json
marqdo call ext/web/component/index.mq.md merge 'a=[{"type":"text","value":"First"}]' 'b=[{"type":"text","value":"Second"}]' --json
```

Use @ record tables to return lists of node maps. merge concatenates these lists;
it does not turn a flat table into columns. wrap keeps child rows in value;
[layout](../layout/index.mq.md) supplies stack and columns helpers for the same
schema. Only [page.render](../page/index.mq.md) needs the native renderer.

Development route: add reusable content in the site's component directories;
add a new structural kind only when stack, columns and Markdown cannot express
it. Update the Go artifact renderer and its tests, then both language helpers.
Never put rendered HTML strings in parent value cells to simulate nesting.

## nodes

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| table | any | required | View table |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | The supplied View table unchanged, preserving its original list or column-table shape |

Use `table` as the inputs for this operation.

Return View `table` unchanged. This preserves the existing identity helper for
column tables as well as record lists. Prefer @ record lists for reusable
components, because list composition can retain typed nested value cells.

Return *table*.

## merge

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| a | list | required | First node record list |
| b | list | required | Second node record list |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Ordered sibling rows |

Use `a`, `b` as the inputs for this operation.

Concatenate `a` and `b` without rendering or flattening child containers.
Both inputs must be lists of View rows, normally @ record tables. To place
two components side by side, concatenate their wrapped columns then use
layout.columns; concatenation alone only establishes sibling order.

Return *[Concat](table.concat(a=`a`, b=`b`))*.

## markdown

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Plain Markdown content |
| slot | text | "" | Stable component name |
| style | text | "" | Optional class name |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | One Markdown leaf |

Use `text`, `slot`="", `style`="" as the inputs for this operation.

Make a leaf from Markdown `text`. Default `slot`="" and `style`="" keep
content independent of placement and theme. The returned list contains one
markdown row; the runtime renders headings, links, lists and tables. No native
plugin is loaded here, and no HTML is inserted into the authored component.

`nodes` =

| @ | type | slot | value | attrs | style |
|---|------|------|-------|-------|-------|
| 1 | markdown | `slot` | `text` | "" | `style` |

Return *nodes*.

## wrap

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| children | list | required | Child node records |
| kind | text | "column" | Container kind |
| slot | text | "" | Stable region name |
| style | text | "" | Optional class name |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | One parent containing its child rows |

Use `children`, `kind`="column", `slot`="", `style`="" as the inputs for this operation.

Wrap `children` in one structural row. Default `kind`="column", `slot`=""
and `style`="" . Use row or column for geometry, nav/header/footer/section/aside
for a semantic region. The value cell holds the child list itself, not rendered
HTML. The native renderer diagnoses unknown child shapes at the render boundary;
this pure constructor does not claim to validate a full site.

`nodes` =

| @ | type | slot | value | attrs | style |
|---|------|------|-------|-------|-------|
| 1 | `kind` | `slot` | `children` | "" | `style` |

Return *nodes*.
