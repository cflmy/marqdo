---
markup: occlusion
title: Retained browser theme compilation
import:
  document: lib/document.mq.md
  web: ext/web/web.mq.md
  text: lib/text.mq.md
---

## css

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Original theme declarations followed by bounded View geometry |

Call with fs.read and plugin under the site root. The original table retains every declaration from the former host page, including fonts, green palette, panel spacing and feedback surfaces. The geometry table only adapts native View wrappers and long text for a narrow viewport. Compilation returns CSS through the existing native style capability; it neither writes a file nor starts a server. Serve owns the explicit generated output. Review the data leaf when adjusting appearance and verify actual computed styles in the browser.

We **original = [Read all retained declarations](document.read_table(path="styles/theme.tables.md", title="original"))** and **geometry = [Read explicit native wrapper rules](document.read_table(path="styles/theme.tables.md", title="geometry"))** and **base = [Compile the retained theme](web.make_style(name="original", table=`original`))** and **wrappers = [Compile bounded View adaptation](web.make_style(name="geometry", table=`geometry`))**.
Return *[Keep original cascade before native adaptation](`base` + "\n" + `wrappers`)*.
