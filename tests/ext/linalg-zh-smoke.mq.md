---
title: 线性代数公式冒烟
description: (AB)^T 化简为 B^T*A^T；探测 ok。
import la:ext/linalg/线性代数.mq.md
---

# main

*`A` = > la.符号 名="A" 行=2 列=3*
*`B` = > la.符号 名="B" 行=3 列=2*
*`P` = > la.乘 左=`A` 右=`B`*
*`T` = > la.转置 式=`P`*
*`S` = > la.化简 式=`T`*
*`txt` = > la.文本 式=`S`*
> 打印 内容=`txt`

*`ping` = > la.探测*
*`ok` = ping[^ok]*
1. `ok`
  > 打印 内容=探测-ok
2. *
  > 打印 内容=探测-fail
