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

We first **nav_nodes = [Compute nav_nodes](nav.nodes())** for the next step.
We first **content_nodes = [Compute content_nodes](content.nodes())** for the next step.
We first **footer_nodes = [Compute footer_nodes](footer.nodes())** for the next step.
We first **upper_regions = [Compute upper_regions](component.merge(a=`nav_nodes`, b=`content_nodes`))** for the next step.
We first **home_regions = [Compute home_regions](component.merge(a=`upper_regions`, b=`footer_nodes`))** for the next step.
After these steps, return *layout.stack(children=`home_regions`, slot="home")*.
