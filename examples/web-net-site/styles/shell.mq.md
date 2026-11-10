---
markup: occlusion
import:
  web: ext/web/web.mq.md
  text: lib/text.mq.md
---

## topnav

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | The value explicitly returned below; conditional returns follow the selected branch |

`topnav` =

| 属性 | 值 |
|------|-----|
| background | #ffffff |
| border-bottom | 1px solid #e7e5e4 |

After these steps, return *topnav*.

## side_panel

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | The value explicitly returned below; conditional returns follow the selected branch |

`side_panel` =

| 属性 | 值 |
|------|-----|
| background | #f5f5f4 |

After these steps, return *side_panel*.

## card_title

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | The value explicitly returned below; conditional returns follow the selected branch |

`card_title` =

| 属性 | 值 |
|------|-----|
| font-size | 1.15rem |
| margin | 0 0 0.35rem |

After these steps, return *card_title*.

## card_body

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | The value explicitly returned below; conditional returns follow the selected branch |

`card_body` =

| 属性 | 值 |
|------|-----|
| color | #57534e |
| margin | 0 |

After these steps, return *card_body*.

## css

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | CSS compiled from the original four declarative property tables |

Call with plugin before discovery. This task preserves the original top navigation background/border, side-panel background, card title size/spacing and card body color/spacing. The selector names target the native components and query-card markup directly. The property functions remain independently callable, so changing one region does not require editing a raw CSS string.

The server writes the returned text through native filesystem IO into its explicit generated resource path. Compilation itself neither writes that file nor mounts static content. These four sections keep their original order; the general View runtime still owns responsive row geometry. A compiler diagnostic stops serving instead of substituting an old stylesheet.

We **nav = [Read the original navigation declarations](topnav())** and **side = [Read the original side-panel declarations](side_panel())** and **title = [Read the original title declarations](card_title())** and **body = [Read the original summary declarations](card_body())**.
We **nav_css = [Compile the navigation selector](web.make_style(name="topnav", table=`nav`))** and **side_css = [Compile the side selector](web.make_style(name="side", table=`side`))** and **title_css = [Compile the native card title selector](web.make_style(name="content.cards h2", table=`title`))** and **body_css = [Compile the native card body selector](web.make_style(name="content.cards p", table=`body`))**.

`parts` =

| css |
|---|
| `nav_css` |
| `side_css` |
| `title_css` |
| `body_css` |

Return *[Keep the compiled sections in order](text.str_join(xs=`parts`, sep="\n"))*.
