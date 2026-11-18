---
markup: occlusion
title: v0.3 推荐循环头嵌套
description: `- [o](外层)` 内再 `- [i](内层)`
---

# main

`外层` =

| a |
|---|
| x |
| y |

`内层` =

| b |
|---|
| 1 |

- [o](遍历 外层 为 o)
  - [i](遍历 内层 为 i)
    **print text=`o`**

*None*
