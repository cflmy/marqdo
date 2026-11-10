---
markup: occlusion
title: Native control leaves
import:
  component: ext/web/component/index.mq.md
  table: lib/table.mq.md
---

These small leaves retain accessible ids and plain text. Parent components decide their arrangement; callbacks use the same ids in the client wiring.

## button

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | text | required | Stable id used by the client event table |
| label | text | required | Plain button text |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | One native button leaf |

Use `id` and `label` for one event target. A button has type button so it does not accidentally submit a surrounding form. Its label stays plain text, including quotes and angle brackets. This function describes a control and performs no click, navigation or DOM mutation. The same leaf is reused by navigation, detail actions and editor actions.

`attrs` =

| id | type |
|---|---|
| `id` | button |

Return *[Describe this native action button](component.element(kind="button", value=`label`, attrs=`attrs`, slot=`id`))*.

## text

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | text | required | Stable client text target |
| value | text | "" | Initial plain text |
| kind | text | "text" | Native text, title or pre kind |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | One native text target |

Use `id`, `value`="" and `kind`="text" to declare a simple display leaf. Initial content is escaped by the renderer, and later updates use browser set_text rather than HTML. Select pre for a log or title for the page heading. The control carries no sibling layout, state machine or event handler; its parent owns placement.

We **attrs = [Retain the stable text target](table.put(in=None, at="id", value=`id`))**.
Return *[Describe the native text leaf](component.element(kind=`kind`, value=`value`, attrs=`attrs`, slot=`id`))*.
