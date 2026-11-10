---
markup: occlusion
title: Home panel assembly
import:
  editor: editor.mq.md
  timer: timer.mq.md
  component: ext/web/component/index.mq.md
  table: lib/table.mq.md
---

## nodes

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Structured View rows for this component |

Home has two independent children: the list editor and the named timer. This parent expresses only that ordering and the panel identity used by the client. Start with Home visible; About and Network are hidden in their own definitions. State and transport do not belong in layout tables, and changing one child does not require copying the entire page.

We **list = [Assemble the editor responsibility](editor.nodes())** and **clock = [Assemble the timer responsibility](timer.nodes())** and **children = [Keep the two home regions ordered](table.concat(a=`list`, b=`clock`))** and **attrs = [Name the client visibility target](table.put(in=None, at="id", value="panel-home"))**.
Return *[Describe the initially visible Home panel](component.wrap(children=`children`, kind="section", attrs=`attrs`, slot="home"))*.
