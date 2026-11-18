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

We first **docs_nodes = [Compute docs_nodes](docs.nodes())** for the next step.
We first **community_nodes = [Compute community_nodes](community.nodes())** for the next step.
We first **siblings = [Compute siblings](component.merge(a=`docs_nodes`, b=`community_nodes`))** for the next step.
After these steps, return *layout.columns(children=`siblings`, slot="footer-links")*.
