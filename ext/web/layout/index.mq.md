---
markup: occlusion
title: ext/web/layout
description: >-
  Web layout engineering skill — partition pages into files until every piece is
  simple. View tables only; HTML/CSS injection is escape hatch, not the method.
import:
  component: ext/web/component.mq.md
---

Layout is how Marqdo web stays maintainable. A homepage is not one HTML blob. It
is a directory of Documents and component files: nav, sidebar, main, footer.
When a region is still complex (for example a two-column footer with brand and
link groups), split again until each file has one job and one simple View table
or short Markdown. Bind values, data, and named styles through
`|type|slot|value|attrs|style|`. Do not reach for `dom.raw` or giant CSS strings
to "finish faster" — that bypasses binding and becomes unreadable.

Canonical design: `doc/design/ext-web-layout-paradigm.md`.

## For AI agents

If you were given only this file, load the rules before writing any site file.
Always pass `--effects plugin` when a later step needs the native web plugin;
these tools are pure documentation helpers and need no plugin.

Call example (plain text):
  marqdo call ext/web/layout/index.mq.md rules --json
  marqdo call ext/web/layout/index.mq.md view_schema --json

When authoring: create `components/*.mq.md` (and nested folders) first, then a
thin `type: web` page that states route and main intent. Prefer
`component.merge` to concatenate View tables. Never make `dom.raw` the home
page implementation.

*None*

## rules

Return the layout engineering rules as a readable map for agents and checklists.
No parameters. This tool does not load the plugin.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | rule map |

`out` =

| key | value |
|-----|-------|
| paradigm | file-partitioned layout |
| partition | homepage splits into nav, sidebar, main, footer files |
| until_simple | split again when a file has multiple jobs or dense styles |
| view_schema | only type slot value attrs style columns |
| forbid_default | HTML string pages; CSS injection; compose_*; json.set DOM bags |
| escape_hatch | dom.raw / page.css only with a written why |
| data_bind | values, db, and named styles via tables |

*out*

## view_schema

Return the only allowed View table column names (English). Chinese aliases are
`类型|槽|值|属性|样式`. No parameters.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | column list |

`cols` =

| col |
|-----|
| type |
| slot |
| value |
| attrs |
| style |

*cols*

## merge_nodes

Concatenate View node tables `a` and `b`. Use after splitting regions into files
so a page can assemble simple parts without HTML.

| 参数 | 类型 | 说明 |
|------|------|------|
| a | any | first View table |
| b | any | second View table |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | concatenated View nodes |

*> component.merge a=`a` b=`b`*

## accept_component

Documentary gate for authors and agents. Pass component `name` (file stem) and
single-duty `duty` (one short sentence). Returns an accept map reminding that
the file must stay single-duty. Does not write disk.

| 参数 | 类型 | 说明 |
|------|------|------|
| name | text | component name, for example nav |
| duty | text | one-job description |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | accept checklist row |

`row` =

| ok | name | duty | must | reject_if |
|----|------|------|------|-----------|
| True | `name` | `duty` | one View table or short Markdown; named styles only | mixes nav+footer · embeds raw HTML as main body · private table dialects |

*row*
