---
title: web artifact render smoke
description: Unified View nodes + document render (ADR 0007); offline, no listen.
import web:ext/web/web.mq.md
import text:lib/text.mq.md
---

# main

`nodes` =

| type | slot | value | attrs | style |
|------|------|-------|-------|-------|
| text | kicker | AI Native | | eyebrow |
| title | title | Marqdo | | title |
| link | action | Read More | /docs | button |

**html = > web.render nodes=`nodes` title="smoke"**
1. `html`
  **ok = > text.contains text=`html` sub="Marqdo"**
  1. `ok`
    > print text=render-nodes-ok
  2. *
    > print text=render-nodes-fail
2. *
  > print text=render-nodes-fail

**doc = > web.render body="# Hello\n\nWorld" title="Doc"**
**ok2 = > text.contains text=`doc` sub="Hello"**
1. `ok2`
  > print text=render-doc-ok
2. *
  > print text=render-doc-fail

**insp = > web.inspect**
1. `insp`
  > print text=inspect-ok
2. *
  > print text=inspect-fail
