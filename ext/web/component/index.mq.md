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
| attrs | any | None | Attribute record for the container, such as id or hidden |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | One parent containing its child rows |

Use `children`, `kind`="column", `slot`="", `style`="", `attrs`=None as the inputs for this operation.

Wrap `children` in one structural row. Default `kind`="column", `slot`=""
and `style`="" . Use row or column for geometry, nav/header/footer/section/aside; form/list/item/select preserve native controls
for a semantic region. The value cell holds the child list itself, not rendered
HTML. The native renderer diagnoses unknown child shapes at the render boundary;
this pure constructor does not claim to validate a full site.

`nodes` =

| @ | type | slot | value | attrs | style |
|---|------|------|-------|-------|-------|
| 1 | `kind` | `slot` | `children` | `attrs` | `style` |

Return *nodes*.

## element

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| kind | text | required | Native leaf kind: input, button, textarea, label, pre, option, text or link |
| value | text | "" | Plain control text or initial input value |
| attrs | any | None | Typed attribute record; legacy text href also works for links |
| slot | text | "" | Stable layout slot |
| style | text | "" | Optional CSS class names |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | One leaf in the existing five-column View schema |

Use `kind`, `value`="", `attrs`=None, `slot`="" and `style`="" when Markdown cannot express a native input or client event target. Supply ids and field names as a record, boolean flags as booleans, and labels as plain text. The renderer escapes values and attribute content; value is never interpreted as HTML. For forms, lists and semantic regions, use wrap with child rows. The constructor is pure; supported kinds, scalar attribute values and forbidden event/style attributes are checked when rendering. Client state belongs in data/aria attributes, while style names reference the page's stylesheet. This keeps controls inspectable without embedding a complete page as HTML.

`nodes` =

| @ | type | slot | value | attrs | style |
|---|---|---|---|---|---|
| 1 | `kind` | `slot` | `value` | `attrs` | `style` |

After describing this control we return *nodes*.



## asset

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| url | text | required | Explicit script or stylesheet URL served by the application |
| kind | text | "script" | script or stylesheet |
| attrs | any | None | Typed attributes such as module type, integrity or data-mq options |
| slot | text | "" | Optional stable resource slot |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | One structured resource leaf in the same five-column View schema |

Use `url`, `kind`="script", `attrs`=None and `slot`="" for an existing resource that the browser must load. A script leaf carries only a URL, never inline author JavaScript; a stylesheet leaf references a separate CSS artifact. Its address stays in value, while attrs holds scalar metadata. The native renderer escapes the URL and attributes, forbids duplicate src/href/rel attributes and rejects script-protocol URLs. Construction is pure and fetches nothing. Keep resource declarations beside the Document component that needs them, after layout and content have been split. The browser loads the actual files when that Document is opened; a node record is not proof that the resource exists.

`nodes` =

| @ | type | slot | value | attrs | style |
|---|---|---|---|---|---|
| 1 | `kind` | `slot` | `url` | `attrs` | "" |

After declaring this resource we return *nodes*.



## resource

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | map | required | Typed data_query, form_ref, view_navigation or pure document_head Resource |
| slot | text | "" | Stable component slot for this dynamic region |
| style | text | "" | Optional wrapper class names |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | One resource node in the existing five-column View schema |

Use `value`, `slot`="" and `style`="" to place a resource supplied by its responsible module. Database query resources come from ext/data/view.mq.md; form references from ext/data/form.mq.md; navigation uses the helper below; pure head declarations come from ext/web/head.mq.md. The resource remains a typed value in the value cell, preserving readable composition and its actual input contract.

Construction is pure and does not query data, load a plugin or submit a form. Serving resolves the resource after preparing request parameters, session state and CSRF. Unknown resources fail before listening; missing runtime resources remain request errors. Offline rendering without a request resolver reports missing context for query, form and session navigation. Pure document_head resources are supported by offline preview and collected into the actual head, never shown as body content.

`nodes` =

| @ | type | slot | value | attrs | style |
|---|---|---|---|---|---|
| 1 | resource | `slot` | `value` | None | `style` |

Return *nodes* for the parent layout to place alongside static Markdown and controls.

## navigation

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| items | any | required | Link records with label, href and optional when, class or media |
| region | text | "nav" | nav, sidebar or footer |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Pure navigation Resource, to be placed with component.resource |

Use `items` and `region`="nav" for session-aware link presentation. Link rows describe destinations; when=guest or auth uses the request's actual session. Empty conditions keep links visible. The native navigation renderer retains its existing classes and conditional visibility, while the recursive parent component still owns placement and geometry.

This constructor does not authenticate a user or register a route. Hiding a link cannot protect its destination from direct access. The server must separately attach actual authentication and authorization gates. Reading or constructing navigation never logs in or out; that behavior belongs to the destination's explicit resource and route.

`resource` =

| _type | items | region |
|---|---|---|
| view_navigation | `items` | `region` |

Return *resource* for explicit placement in the same five-column View.
