---
title: 内置 len / str / int
description: 长度与类型转换（S0 最小标准库）
---

# main

len 接受文本（按 Unicode 标量计）或表/列表：

*`s` = hello*

*`n` = > len `s`*

> print text=`n`

str 把值转成显示文本；int 解析整数（或把 True / False 当成 1 / 0）：

*`t` = > str 7*

*`i` = > int `t`*

> print text=`i`
