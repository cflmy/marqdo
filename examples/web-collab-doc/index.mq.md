---
title: web-collab-doc
description: Two-browser collaborative plain-text editor over WS room (LWW snapshot sync)
import web:ext/web/web.mq.md
---

# main

Collaborative notepad: open the page in two browsers; edits fan out via
`/live/{room}` WebSocket (mode=room + presence). Build WASM first:

```
marqdo wasm build -o examples/web-collab-doc/static
```

**`embed` = > web.client_embed bridge="/static/marqdo-bridge.js" wasm="/static/marqdo_wasm.wasm" source="/static/client.mq.md" boot=True**

**`intro` = "<p class='lede'>Room <code>demo</code> · open twice to co-edit.</p><p id='presence'>presence: —</p><textarea id='doc' rows='16' cols='72' placeholder='Type together…'></textarea><pre id='log'></pre>" + embed**

**page = > web.page title="Collab doc" intro=intro layout="bare"**
**app = > web.app page=page host="127.0.0.1" port=18094**
**app = > app.static dir="static" mount="/static"**
**app = > `app`.route_ws path="/live/{id}" mode="room" room_key="doc.{id}" presence=True**
> `app`.listen
