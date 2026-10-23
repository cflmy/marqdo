---
markup: occlusion
title: S2 G06 效应传递与未知不纯
status: implemented
---

## save

写入演示文件。

> fs.write_text path=out.txt text=ok
*1*

## wrap

无危险词包装，内部调用 save。

> save
*None*

# main

> wrap
> print text=ok
