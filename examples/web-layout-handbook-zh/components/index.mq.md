---
markup: occlusion
import:
  组件: ext/web/网页/组件/索引.mq.md
  布局: ext/web/布局/索引.mq.md
  nav: ./nav/index.mq.md
  content: ./main/index.mq.md
  footer: ./footer/index.mq.md
---

## 节点

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Nested child rows |

本层只安排导航、主体、页脚之间的关系，正文仍留在对应 Markdown 叶子中。
若子块仍含独立分栏，就继续沿对应目录的 index.mq.md 分解。
不要把渲染后的字符串放回值列，子节点在整个装配过程中保持结构。

**nav_nodes = > nav.节点**
**content_nodes = > content.节点**
**footer_nodes = > footer.节点**
**upper_regions = 组件.合并 甲=`nav_nodes` 乙=`content_nodes`**
**home_regions = 组件.合并 甲=`upper_regions` 乙=`footer_nodes`**
*> 布局.纵排 子节点=`home_regions` 槽="home"*
