---
type: web
title: Marqdo 博客
route: /
method: GET
description: >-
  Document Artifact 示例（ADR 0007）：首页即文档。
  子页/API 见 pages/ 与 api/；入口可 web.serve 扫描注册。
导入 网页:ext/web/网页.mq.md
---

# Marqdo 博客

可执行文档即网页。本站用 Artifact Metadata 声明路由，正文即内容。

## 文章

从 `data_source`（元信息）或库表加载列表；本示例入口直接服务文档页。

---

运行（仓库根目录）：

```text
marqdo run examples/marqdo-blog/serve.mq.md
```
