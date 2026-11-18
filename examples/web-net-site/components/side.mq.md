---
markup: occlusion
title: side Markdown component
import:
  component: ext/web/component/index.mq.md
  fs: lib/fs.mq.md
---

This leaf module owns side content and its semantic region. Parent layout only places the returned nodes. Edit the adjacent Markdown leaf to change wording or links; the component does not query records, authenticate users or generate a raw HTML string.

## side

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | One aside region containing terminal Markdown content |

Read the Markdown leaf relative to the granted site root with fs.read. Its links keep the original example's destinations; actual routing and access policy remain in the serving entry. This region is already simple enough to express as Markdown, so further wrappers or a custom table dialect would add no responsibility. The semantic container and class preserve its role for the native layout and theme.

We **text = [Read this region's terminal Markdown](fs.read_text(path="components/content/side.md"))** and **leaf = [Retain content as native Markdown](component.markdown(text=`text`, slot="side-content"))**.
Return *[Place that leaf in its semantic region](component.wrap(children=`leaf`, kind="aside", slot="side", style="side"))*.
