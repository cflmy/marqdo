---
title: 内置 input
description: 从 stdin 读一行；可选 prompt。可用 --stdin-file 或 live view 预置输入。
---

# main

CLI 可用管道或 --stdin-file 喂入一行；live marqdo view 在 Execution 区可填预置输入后重跑。静态站点默认无 stdin，会看到 capture 提示。

*`name` = > input prompt=Name: *

> print text=Hello `name`!
