---
markup: occlusion
import:
  component: ext/web/component/index.mq.md
  layout: ext/web/layout/index.mq.md
  brand: ./brand/index.mq.md
  links: ./links/index.mq.md
---

## nodes

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Nested child rows |

This parent arranges brand content and the link-region subtree. Content stays
in their Markdown leaves; this file owns their one placement relationship. Continue into each child index when another independent layout
remains. Preserve node lists rather than rendering children into HTML strings.

**brand_nodes = > brand.nodes**
**links_nodes = > links.nodes**
**siblings = component.merge a=`brand_nodes` b=`links_nodes`**
**footer_columns = layout.columns children=`siblings` slot="footer"**
*> component.wrap children=`footer_columns` kind="footer" slot="footer"*
