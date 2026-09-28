---
type: web
title: Marqdo 博客
route: /
method: GET
data_source: posts
data_order: -created_at
description: >-
  Document Artifact 首页（ADR 0007）：正文即文档，列表来自 data_source。
导入 网页:ext/web/网页.mq.md
---

# Marqdo 博客

可执行文档即网页。本站用 Artifact Metadata 声明路由；文章列表由 `data_source: posts` 在服务时加载。
