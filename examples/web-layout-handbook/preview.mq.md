---
markup: occlusion
title: Callable preview of the actual layout tree
import:
  home: ./components/index.mq.md
  web: ext/web/web.mq.md
---

Preview begins at the site component root and shares its renderer with the server.

## main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | HTML text identical to the existing printed preview |

Call this ordinary main function to obtain the actual HTML value; run still prints the same preview for existing callers. Markdown leaves are read relative to the granted site root, so use --fs-root examples/web-layout-handbook from the checkout root, with fs.read, plugin and output.write. Preview opens no listener and changes no content. The same node renderer serves the Document, including both footer regions and the nested link columns. Inspecting this returned HTML does not replace browser geometry or interaction checks.

We **nodes = [Assemble the complete component tree](home.nodes())**, then **html = [Use the same node renderer as the served Document](web.render(nodes=`nodes`, title="Marqdo layout handbook"))**.
We **[Keep the existing run preview output](print(text=`html`))**, then return *html*.
