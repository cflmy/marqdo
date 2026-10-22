---
markup: occlusion
title: Row records
description: First header @ / row makes a list of maps
---

# main

When the first header is `@` or `row`, each data row is a map (marker column is not a field). Nested get: `[field]([1](orders))`.

`orders` =

| @ | name | qty |
|---|------|-----|
| 1 | apple | 2 |
| 2 | pear | 3 |

**`n` = [name](access([1](access(orders, "1")), "name"))**

> print text=`n`

**`q` = [qty](access([2](access(orders, "2")), "qty"))**

> print text=`q`
