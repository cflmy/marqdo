---
markup: occlusion
title: ext/web/client
description: Browser Runtime helpers (WASM bridge). Prefer lib/browser for effect tables (ADR 0007).
import:
  cap: ext/web/_capability/index.mq.md
  table: lib/table.mq.md
---

Browser helpers describe the official Marqdo WASM bridge and small effects.
They construct text/maps without a native server. Keep initial page layout in
[layout components](../layout/index.mq.md), and use lib/browser for client logic.

```sh
marqdo call ext/web/client/index.mq.md embed --json
marqdo call ext/web/client/index.mq.md text_patch 'sel="#status"' 'text="Ready"' --json
```

The bridge and WASM assets must actually be hosted at the chosen URLs. embed
returns a loading descriptor; it does not install assets or implement business
JavaScript. Native HTML snippets here are runtime integration, not an invitation
to put complete navigation/footer markup in authored content. Changes require
browser bridge tests and matching Chinese entry points.

## embed

| 参数 | 类型 | 说明 |
|------|------|------|
| bridge | text | bridge script URL |
| wasm | text | wasm URL |
| source | text | optional source URL |
| boot | bool | auto boot |

HTML snippet that loads the official browser Marqdo bridge (route C/D). Defaults:
`bridge`="/static/marqdo-bridge.js", `wasm`="/static/marqdo_wasm.wasm",
`source`="", `boot`=True . With boot and non-empty source, auto-mounts. No author
JS. Not a substitute for file-partitioned Document layout.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | text | HTML script tag |

1. `boot`
    1. `source` != ""
        *"<script type=\"module\" src=\"" + bridge + "\" data-mq-wasm=\"" + wasm + "\" data-mq-source-url=\"" + source + "\"></script>"*
    2. *
        *"<script type=\"module\" src=\"" + bridge + "\"></script>"*
2. *
    *"<script type=\"module\" src=\"" + bridge + "\" data-mq-no-boot=\"1\"></script>"*

## text_patch

| 参数 | 类型 | 说明 |
|------|------|------|
| sel | text | CSS selector |
| text | text | text content |

Build a `set_text` effects map for selector `sel` with string `text`. Prefer
lib/browser in client .mq.md programs; this helper is for call-side sketches.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | effects map |

**`m` = > table.put in=None at=sel value=text**
*> table.put in=None at="set_text" value=m*

## dom_patch

| 参数 | 类型 | 说明 |
|------|------|------|
| sel | text | CSS selector |
| html | text | optional HTML |
| attrs | any | optional attrs map |
| remove | bool | remove node |

Build a `dom` effects map for selector `sel`. Optional `html`="", `attrs`=None,
`remove`=False . Prefer View/Markdown over raw html patches for page shells.

| 返回 | 类型 | 说明 |
|------|------|------|
| result | any | effects map |

**`spec` = > table.put in=None at="html" value=html**
1. `attrs`
  **`spec` = > table.put in=`spec` at="attrs" value=attrs**
2. *
  **_ = 1**
1. `remove`
  **`spec` = > table.put in=`spec` at="remove" value=True**
2. *
  **_ = 1**
**`m` = > table.put in=None at=sel value=spec**
*> table.put in=None at="dom" value=m*
