---
title: Row records
description: First header @ / row makes a list of maps
---

# main

When the first header is `@` or `row`, each data row is a map (marker column is not a field).

`orders` =

| @ | name | qty |
|---|------|-----|
| 1 | apple | 2 |
| 2 | pear | 3 |

*`n` = `orders`[^1][^name] *

> print text=`n`

*`q` = `orders`[^2][^qty] *

> print text=`q`
