---
markup: occlusion
import:
  component: ext/web/component/index.mq.md
  layout: ext/web/layout/index.mq.md
  nav: ./nav/index.mq.md
  content: ./main/index.mq.md
  footer: ./footer/index.mq.md
---

## nodes

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Nested child rows |

This parent arranges navigation, main content and footer. Content stays
in their Markdown leaves; this file owns their one placement relationship. Continue into each child index when another independent layout
remains. Preserve node lists rather than rendering children into HTML strings.

**nav_nodes = > nav.nodes**
**content_nodes = > content.nodes**
**footer_nodes = > footer.nodes**
**upper_regions = component.merge a=`nav_nodes` b=`content_nodes`**
**home_regions = component.merge a=`upper_regions` b=`footer_nodes`**
*> layout.stack children=`home_regions` slot="home"*
