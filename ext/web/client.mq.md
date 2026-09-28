---
title: ext/web/client
description: Browser Runtime helpers (WASM bridge). Prefer lib/browser for effect tables (ADR 0007).
import cap:ext/web/_capability.mq.md
import table:lib/table.mq.md
---

## embed
    + `bridge`="/static/marqdo-bridge.js"
    + `wasm`="/static/marqdo_wasm.wasm"
    + `source`=""
    + `boot`=True

HTML snippet that loads the official browser Marqdo bridge (route C/D). With `boot` and non-empty `source`, auto-mounts. No author JS.

> cap.load
1. `boot`
    1. `source` != ""
        *"<script type=\"module\" src=\"" + bridge + "\" data-mq-wasm=\"" + wasm + "\" data-mq-source-url=\"" + source + "\"></script>"*
    2. *
        *"<script type=\"module\" src=\"" + bridge + "\"></script>"*
2. *
    *"<script type=\"module\" src=\"" + bridge + "\" data-mq-no-boot=\"1\"></script>"*

## text_patch
    + `sel`
    + `text`

`set_text` effects map. Prefer `lib/browser` in client `.mq.md`.

**`m` = > table.put in=None at=sel value=text**
*> table.put in=None at="set_text" value=m*

## dom_patch
    + `sel`
    + `html`=""
    + `attrs`=None
    + `remove`=False

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
