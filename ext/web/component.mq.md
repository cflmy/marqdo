---
title: ext/web/component
description: Reusable Document fragments as View node tables (ADR 0007).
import table:lib/table.mq.md
---

## nodes
    + `table`

Identity helper — a component is a View table `|type|slot|value|attrs|style|`.

*table*

## merge
    + `a`
    + `b`

Concatenate two View node lists (list-of-maps or column tables). Prefer authoring one table.

*> table.concat a=`a` b=`b`*
