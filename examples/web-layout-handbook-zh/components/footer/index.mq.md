---
markup: occlusion
import:
  组件: ext/web/网页/组件/索引.mq.md
  布局: ext/web/布局/索引.mq.md
  brand: ./brand/index.mq.md
  links: ./links/index.mq.md
---

## 节点

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Nested child rows |

本层只安排品牌内容与链接子树之间的关系，正文仍留在对应 Markdown 叶子中。
若子块仍含独立分栏，就继续沿对应目录的 index.mq.md 分解。
不要把渲染后的字符串放回值列，子节点在整个装配过程中保持结构。

We first **brand_nodes = [Compute brand_nodes](brand.节点())** for the next step.
We first **links_nodes = [Compute links_nodes](links.节点())** for the next step.
We first **siblings = [Compute siblings](组件.合并(甲=`brand_nodes`, 乙=`links_nodes`))** for the next step.
We first **footer_columns = [Compute footer_columns](布局.分栏(子节点=`siblings`, 槽="footer"))** for the next step.
After these steps, return *组件.包裹(子节点=`footer_columns`, 种类="footer", 槽="footer")*.
