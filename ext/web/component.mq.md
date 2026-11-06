---
markup: occlusion
title: ext/web/component
description: >-
  Reusable Document fragments as View node tables (ADR 0007). One file, one duty;
  merge simple parts instead of writing HTML.
import:
  table: lib/table.mq.md
---

Components are how file-partitioned layouts stay simple. Each component file
should expose one View table (or short Markdown consumed elsewhere). When a
footer needs two columns, use two component files and `merge` — do not embed a
mini-HTML layout in one cell.

## For AI agents

Call example (plain text):
  marqdo call ext/web/layout/index.mq.md accept_component name="footer-links" duty="right link column only" --json

Prefer authoring separate tables then merge. No plugin required for nodes/merge.

*None*

## nodes
    + `table`

Identity helper — a component is a View table `|type|slot|value|attrs|style|`.
Keep the table small enough that a human sees a single region.

| 参数 | 类型 | 说明 |
|------|------|------|
| table | any | View table |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | same table |

*table*

## merge
    + `a`
    + `b`

Concatenate two View node lists (list-of-maps or column tables). Use after
splitting regions into files. Prefer many small tables over one private dialect.

| 参数 | 类型 | 说明 |
|------|------|------|
| a | any | first nodes |
| b | any | second nodes |

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | concatenated nodes |

*> table.concat a=`a` b=`b`*
