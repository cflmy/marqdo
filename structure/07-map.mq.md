---
markup: occlusion
title: Maps (horizontal tables)
description: Multi-column tables are maps; link-index get
---

# main

A multi-column table with one data row is a map. Read values with `[key](map)`.

`kinds` =

| apple | cucumber |
|-------|----------|
| fruit | vegetable |

**`k` = [apple](access(kinds, "apple"))**

> print text=`k`

**`k` = [cucumber](access(kinds, "cucumber"))**

> print text=`k`
