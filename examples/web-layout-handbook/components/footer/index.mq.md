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

We first **brand_nodes = [Compute brand_nodes](brand.nodes())** for the next step.
We first **links_nodes = [Compute links_nodes](links.nodes())** for the next step.
We first **siblings = [Compute siblings](component.merge(a=`brand_nodes`, b=`links_nodes`))** for the next step.
We first **footer_columns = [Compute footer_columns](layout.columns(children=`siblings`, slot="footer"))** for the next step.
After these steps, return *component.wrap(children=`footer_columns`, kind="footer", slot="footer")*.
