---
markup: occlusion
import:
  组件: ext/web/网页/组件/索引.mq.md
  布局: ext/web/布局/索引.mq.md
  docs: ./docs/index.mq.md
  community: ./community/index.mq.md
---

## 节点

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Nested child rows |

本层只安排文档链接与社区链接之间的关系，正文仍留在对应 Markdown 叶子中。
若子块仍含独立分栏，就继续沿对应目录的 index.mq.md 分解。
不要把渲染后的字符串放回值列，子节点在整个装配过程中保持结构。

**docs_nodes = > docs.节点**
**community_nodes = > community.节点**
**siblings = 组件.合并 甲=`docs_nodes` 乙=`community_nodes`**
*> 布局.分栏 子节点=`siblings` 槽="footer-links"*
