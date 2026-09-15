---
title: view 与用户文档站
description: 本地浏览与静态导出由解释器生成
---

# main

里程碑：`marqdo view` 与 `view output`。

本地浏览：`marqdo view public`

静态导出：`marqdo view output public -o public`

用户可执行文档在 `public/`；生成的 HTML（`index.html` / `pages/`）供静态托管，CI 发布到 `gh-pages`。

> 打印 内容=文档站：由 Marqdo 解释器生成，不是手写 HTML。
