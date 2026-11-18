---
markup: occlusion
title: ext/web/layout
description: Split structure first, recursively down to simple Markdown leaves.
import:
  component: ext/web/component/index.mq.md
---

Engineering practice: draw the region tree before writing content. Home splits
into navigation, main and footer. Footer splits into brand and links; if links
has two columns, split again into docs and community. Continue while a region
needs independent internal layout. Stop at one readable Markdown responsibility,
not at an arbitrary number of files or one file per sentence.

Follow the real [reference site](../../../examples/web-layout-handbook/index.mq.md).
Parent index.mq.md files import children and declare one layout relationship;
leaf content.md files contain ordinary Markdown. Splitting an HTML string into
several files does not make the structure document-native.

```sh
marqdo call ext/web/layout/index.mq.md rules --json
marqdo call ext/web/layout/index.mq.md columns 'children=[{"type":"markdown","value":"Left"},{"type":"markdown","value":"Right"}]' 'slot="footer"' --json
```

columns preserves child rows in value and renders a responsive grid; narrow
screens collapse automatically. stack preserves top-to-bottom region order.
The style column references class names, not raw CSS or a theme definition.
Merging alone creates siblings, not columns. Use component.markdown for content
and component.wrap for semantic regions such as nav or footer.

Development and acceptance: name each leaf by one content responsibility. If a
parent contains long prose or HTML tags, split further. Preview with page.render,
then serve and verify home, about and the API. New structural kinds belong in
plugins/web/internal/artifact with bilingual helpers and web_handbook tests.

## rules

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Column table with aligned key/value lists describing layout rules; no source inspection |

Return the engineering convention and stopping rule. No parameters or plugin.
This is guidance rather than evidence that a particular source file passes;
acceptance also requires reading the leaves and exercising the resulting page.

Apply the stopping rule to responsibilities rather than file size. A footer with two independent groups still needs child regions even if its text is short. A leaf containing several ordinary paragraphs may remain one Markdown responsibility when it needs no internal geometry. The parent imports those children and places them; the renderer supplies the resulting container structure. This table records that convention but does not inspect your files or certify that a layout works.

`out` =

| key | value |
|-----|-------|
| paradigm | recursive file-partitioned layout |
| partition | home to nav/main/footer; footer to brand/links; links to docs/community |
| until_simple | one simple Markdown responsibility with no independent internal layout |
| view_schema | type slot value attrs style; parent value holds child rows |
| parent | import child components and choose one relationship |
| leaf | ordinary Markdown content without an HTML shell |
| geometry | merge gives siblings; stack and columns give placement |
| verify | offline render; live routes; narrow and wide browser geometry |

Return *out*.

## view_schema

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Public column names |

Return the five public View columns. A container value holds more rows of
the same schema; a Markdown value holds content text. This keeps recursive
layout within one representation instead of adding a private fragment dialect.

`cols` =

| col |
|-----|
| type |
| slot |
| value |
| attrs |
| style |

Return *cols*.

## merge_nodes

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| a | list | required | First node records |
| b | list | required | Second node records |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Ordered sibling nodes |

Use `a`, `b` as the inputs for this operation.

Concatenate `a` and `b` as siblings. This helper preserves structure and order
but does not imply geometry; use columns for side-by-side placement or stack
for a vertical page. The @ record lists retain nested values throughout.

Return *[Merge](component.merge(a=`a`, b=`b`))*.

## stack

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| children | list | required | Child component records |
| slot | text | "" | Parent region name |
| style | text | "" | Optional class name |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Parent container rows |

Use `children`, `slot`="", `style`="" as the inputs for this operation.

Declare one vertical relationship for `children`. Default `slot`="" and

`style`="" . The parent owns placement while each imported child owns content
or its next decomposition level. Return View rows so another parent can wrap
them without first serializing any HTML.

Return *[Wrap](component.wrap(children=`children`, kind="column", slot=`slot`, style=`style`))*.

## columns

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| children | list | required | Child component records |
| slot | text | "" | Parent region name |
| style | text | "" | Optional class name |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Parent container rows |

Use `children`, `slot`="", `style`="" as the inputs for this operation.

Declare one responsive multi-column relationship for `children`. Default `slot`="" and

`style`="" . The parent owns placement while each imported child owns content
or its next decomposition level. Return View rows so another parent can wrap
them without first serializing any HTML.

Return *[Wrap](component.wrap(children=`children`, kind="row", slot=`slot`, style=`style`))*.

## accept_component

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| name | text | required | Component name |
| duty | text | required | One responsibility |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | Checklist record with status, name, duty and check; it does not certify a component |

Use `name`, `duty` as the inputs for this operation.

Return a review checklist for component `name` and stated `duty`. Two strings
cannot establish source quality or a working layout, so this tool does not
return an unconditional ok=True. Verify the actual source and rendered page
before accepting the component. It never writes or scans files.

`row` =

| status | name | duty | check |
|--------|------|------|-------|
| checklist | `name` | `duty` | one Markdown leaf or one parent relationship; no raw HTML shell |

Return *row*.
