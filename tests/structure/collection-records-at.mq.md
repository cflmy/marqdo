---
title: row records with @ marker
---

# main

`orders` =

| @ | name | qty |
|---|------|-----|
| 1 | apple | 2 |
| 2 | pear | 3 |

**n = [name]([1](orders))**

> print text=`n`

**q = [qty]([2](orders))**

> print text=`q`
