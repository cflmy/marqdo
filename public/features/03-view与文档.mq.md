---
title: view 与用户文档站
description: 本地浏览与静态导出由解释器生成
---

# main

里程碑：`marqdo view` 与 `view output`（v1.3.3 文档面）。

本地浏览：`marqdo view public`

静态导出：`marqdo view output public -o public`

Browse Structure 保真叙述、高亮可执行标记，并挂载运行结果（变量值、分支命中臂、调用/循环帧）。
用户站与 live view 同源渲染；CI 发布到 `gh-pages`。

> 打印 内容=文档站：由 Marqdo 解释器生成，不是手写 HTML。
