---
markup: occlusion
title: Collections (tables)
description: One-column tables are lists
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

A single-column Markdown table is a list; loop to read each item. Prefer link-index get `[1](basket)` (1-based).

`basket` =

| fruit |
|-------|
| apple |
| pear |
| peach |

- [fruit](遍历 basket 为 fruit)
  We now **[Display this step's result](> print text=`fruit`)**.
