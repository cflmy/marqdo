---
markup: occlusion
title: SEO, font and theme resource declarations
import:
  component: ext/web/component/index.mq.md
  head: ext/web/head.mq.md
---

This leaf preserves the original site metadata and Google Fonts resource addresses. It declares links, never fetches them during assembly. Theme CSS is explicitly generated from style tables by serve and mounted at the declared static path.

## nodes

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| homepage | bool | False | Include the original home-only SEO rows |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Pure head resource for real SEO, fonts and the generated theme |

Use `homepage`=False without effects during page assembly. Only the home Document opts into its original SEO rows; fonts and the theme remain shared. Metadata remains key/value records and the links retain native rel/href/crossorigin fields. The renderer collects them into the actual head in both preview and HTTP output, deduplicating equal declarations. They do not become body placeholders or fragment markup.

Font delivery still depends on browser access to the external font service; the page uses its declared fallbacks when unavailable. Declaring the theme path does not build it or prove it exists. The serving workflow compiles and writes that exact file before opening a listener, while a static preview caller owns asset preparation.

`meta` =

| @ | key | value |
|---|---|---|
| 1 | description | Tables as the full stack — Marqdo Web demo with zero author business JavaScript. |
| 2 | og:type | website |
| 3 | og:title | Marqdo Web Site |

`links` =

| @ | rel | href | crossorigin |
|---|---|---|---|
| 1 | preconnect | https://fonts.googleapis.com | |
| 2 | preconnect | https://fonts.gstatic.com | anonymous |
| 3 | stylesheet | https://fonts.googleapis.com/css2?family=Great+Vibes&family=Noto+Sans+SC:wght@300;400;500;700&family=Noto+Serif+SC:wght@400;600&display=swap | |
| 4 | stylesheet | /static/generated/theme.css | |

We **selected_meta = None** because non-home pages retain only their originally shared head links.
1. `homepage`
  We **selected_meta = meta** to retain the original home-specific SEO.
We **resource = [Retain actual metadata and native head links](head.resource(meta=`selected_meta`, links=`links`))**.
Return *[Place one pure head resource alongside the layout](component.resource(value=`resource`, slot="document-head"))*.
