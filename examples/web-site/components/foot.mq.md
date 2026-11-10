---
markup: occlusion
title: Public footer Markdown leaf
import:
  component: ext/web/component/index.mq.md
  fs: lib/fs.mq.md
---

The public footer is one simple set of project and source links. It has no independent internal columns, so its terminal Markdown is the stopping point for decomposition. The parent controls where it appears; the theme supplies its existing footer appearance.

## foot

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Semantic footer containing its terminal Markdown links |

Read with fs.read relative to the granted site root. The leaf retains the original project and source destinations; it does not query business rows or expose session credentials. Changing wording belongs in content/footer.md, while changing layout belongs in the parent. This module never renders a child to HTML before composition.

We **text = [Read the terminal project links](fs.read_text(path="components/content/footer.md"))** and **leaf = [Keep the links in Markdown](component.markdown(text=`text`, slot="footer-content"))**.
Return *[Preserve the footer's semantic region](component.wrap(children=`leaf`, kind="footer", slot="footer", style="foot"))*.

