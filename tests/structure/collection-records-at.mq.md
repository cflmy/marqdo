---
markup: occlusion
title: row records with @ marker
---

# main

`orders` =

| @ | name | qty |
|---|------|-----|
| 1 | apple | 2 |
| 2 | pear | 3 |

**n = [name](access([1](access(orders, "1")), "name"))**

> print text=`n`

**q = [qty](access([2](access(orders, "2")), "qty"))**

> print text=`q`
