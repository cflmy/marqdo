---
markup: occlusion
import:
  fs: lib/fs.mq.md
  component: ext/web/component/index.mq.md
---

## nodes

| 返回 | 类型 | 说明 |
|------|------|------|
| result | list | Markdown leaf rows |

This leaf owns only the main Markdown content; its parent owns placement.
Edit adjacent content.md to change prose. Add a child directory when a second
layout responsibility appears instead of introducing HTML into the content.

**body = fs.read_text path="components/main/content.md"**
*> component.markdown text=`body` slot="main"*
