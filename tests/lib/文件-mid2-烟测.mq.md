---
title: 文件 Mid2 M9 烟测
导入 文件:lib/文件.mq.md
---

# main

> 文件.建目录树 路径="m9-zh/子"

> 文件.写文本 路径="m9-zh/a.txt" 内容="A"

*st = > 文件.状态 路径="m9-zh/a.txt"*
> print text=`st`[^is_file]

*w = > 文件.遍历 路径="m9-zh"*
*n = > 长度 `w`*
> print text=`n`

> 文件.删目录树 路径="m9-zh"

*gone = > 文件.存在 路径="m9-zh"*
> print text=`gone`
