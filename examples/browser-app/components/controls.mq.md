---
markup: occlusion
title: Native browser control leaves
import:
  component: ext/web/component/index.mq.md
  table: lib/table.mq.md
---

## button

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | text | required | Event target shared with the client wire table |
| label | text | required | Literal visible action |
| secondary | bool | False | Use the retained secondary palette |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | One disabled native button awaiting successful runtime boot |

Use `id`, `label` and `secondary`=False for one leaf. The button starts disabled, carries type button and is enabled only after the official runtime boots successfully. Its label remains plain text. The caller chooses placement; this helper neither binds a click handler nor hides a sibling layout. Keep its id aligned with static/app.mq.md.

`attrs` =

| id | type | disabled |
|---|---|---|
| `id` | button | True |

We **style = ""**.
1. `secondary`
  We **style = "secondary"**.
Return *[Describe one literal native action](component.element(kind="button", value=`label`, attrs=`attrs`, style=`style`, slot=`id`))*.

## input

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | text | required | Client value target |
| hint | text | required | Plain placeholder text |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | One disabled text input |

Use `id` and `hint` for the draft or socket message. A native text input preserves its ordinary editing semantics and starts disabled until WASM is ready. The hint is placeholder text rather than a markup template. The component owns one control; the client selects its value through the wire table when the adjacent action is clicked.

`attrs` =

| id | type | placeholder | disabled |
|---|---|---|---|
| `id` | text | `hint` | True |

Return *[Describe this native value input](component.element(kind="input", attrs=`attrs`, slot=`id`))*.

## text

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | text | required | Stable output target |
| value | text | required | Literal initial text |
| kind | text | "text" | Native text or pre output |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | One escaped text leaf |

Use `id`, `value` and `kind`="text" for an output target. Select pre for the runtime log and ordinary text for the path, tick count or network outcome. Initial and later content stays literal: the renderer escapes this value and callbacks use set_text. This leaf declares no timer or network operation; those belong to the browser program.

We **attrs = [Retain the exact output target](table.put(in=None, at="id", value=`id`))**.
Return *[Describe the literal output leaf](component.element(kind=`kind`, value=`value`, attrs=`attrs`, slot=`id`))*.
