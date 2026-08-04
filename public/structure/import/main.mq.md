---
title: 跨文件导入
description: frontmatter 中 `> 文件.mq.md` 导入模块
> utils.mq.md
---

# main

文件头 YAML 里写 `> utils.mq.md` 即可导入同目录模块。

导入后：用返回值绑定，再输出；并直接调用问候。

*`y` = > 加一 n=41*

> print text=`y`

> 问候 谁=Marqdo
