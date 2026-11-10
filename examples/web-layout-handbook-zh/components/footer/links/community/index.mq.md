---
markup: occlusion
import:
  文件: lib/文件.mq.md
  组件: ext/web/网页/组件/索引.mq.md
---

## 节点

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Markdown leaf rows |

此叶子只负责footer/links/community 的 Markdown 内容，父组件决定排列。修改文字只改相邻 content.md，
不添加 HTML 标签或内部分栏；出现第二种布局职责时再开子目录。

We first **body = [Compute body](文件.读文本(路径="components/footer/links/community/content.md"))** for the next step.
After these steps, return *组件.正文(文本=`body`, 槽="footer-links-community")*.
