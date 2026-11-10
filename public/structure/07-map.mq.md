---
markup: occlusion
title: Maps (horizontal tables)
description: Multi-column tables are maps; link-index get
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

A multi-column table with one data row is a map. Read values with `[key](map)`.

`kinds` =

| apple | cucumber |
|-------|----------|
| fruit | vegetable |

We first **k = [apple](access(`kinds`, "apple"))** for the next step.

We now **[Display this step's result](> print text=`k`)**.

We first **k = [cucumber](access(`kinds`, "cucumber"))** for the next step.

We now **[Display this step's result](> print text=`k`)**.
