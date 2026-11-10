---
markup: occlusion
title: Row records
description: First header @ / row makes a list of maps
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

When the first header is `@` or `row`, each data row is a map (marker column is not a field). Nested get: `[field]([1](orders))`.

`orders` =

| @ | name | qty |
|---|------|-----|
| 1 | apple | 2 |
| 2 | pear | 3 |

We first **n = [name](access([1](access(`orders`, "1")), "name"))** for the next step.

We now **[Display this step's result](> print text=`n`)**.

We first **q = [qty](access([2](access(`orders`, "2")), "qty"))** for the next step.

We now **[Display this step's result](> print text=`q`)**.
