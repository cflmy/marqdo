---
markup: occlusion
title: Document head Resource — SEO, fonts and resource links
---

Use this single-file resource inside the existing five-column View schema. It preserves metadata and native head links when a structured Document owns the page body. Place its returned value through component.resource alongside the layout. The renderer collects it into the actual head, leaving no visible placeholder in body or fragment responses. No new Markdown mark, frontmatter dialect or page mutation chain is involved.

Construction is pure. Rendering validates the closed contract, escapes metadata and uses the same head-link renderer as ordinary Documents. It does not fetch a font, execute a script or prove an external resource exists. Browsers load the declared URLs only when they open the real page. A local pure construction example is ready from the checkout:

```sh
marqdo call ext/web/head.mq.md resource 'meta=[{"key":"description","value":"A readable site"}]' --json
```

## resource

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| meta | any | None | key/value records or a GFM column table; scalar SEO values |
| links | any | None | Native rel/href link records, including optional type, media, crossorigin, defer and async |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Pure document_head descriptor for component.resource; no IO |

Use `meta`=None and `links`=None to declare only what this Document needs. Metadata includes description, og: names and other native SEO names. Native title overrides and canonical/icon link keys retain ordinary Document behavior. Charset, viewport and csrf-token remain owned by the runtime and cannot be redeclared here. An empty descriptor contributes no metadata rather than fabricating a page result.

Links reuse the existing native field aliases, with rel selecting stylesheet, preconnect, preload, icon, script or module behavior. URLs must start with / for site resources or use http(s); protocol-relative URLs, active data schemes and malformed cells fail. Use the official client nodes for the browser bridge rather than inlining business JavaScript. A link does not mount a file or register a server route.

Nested layout modules can contribute head resources. Equal link declarations are deduplicated; conflicting values for the same metadata key are diagnostics rather than silently replacing each other. Fragments omit these declarations because they update an existing Document, while offline preview supports this pure resource without inventing a request session. Request-dependent data, form and navigation resources still require their actual request renderer.

`head` =

| _type | meta | links |
|---|---|---|
| document_head | `meta` | `links` |

After describing the actual metadata and resource addresses, return *head* for explicit placement by the parent component.
