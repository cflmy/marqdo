---
markup: occlusion
import:
  component: ext/web/component/index.mq.md
  layout: ext/web/layout/index.mq.md
  docs: ./docs/index.mq.md
  community: ./community/index.mq.md
---

## nodes

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Nested child rows |

This parent arranges documentation links and community links. Content stays
in their Markdown leaves; this file owns their one placement relationship. Continue into each child index when another independent layout
remains. Preserve node lists rather than rendering children into HTML strings.

**docs_nodes = > docs.nodes**
**community_nodes = > community.nodes**
**siblings = component.merge a=`docs_nodes` b=`community_nodes`**
*> layout.columns children=`siblings` slot="footer-links"*
