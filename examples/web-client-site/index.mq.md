---
title: web-client-site
description: SSR shell + client_embed auto-mount; author zero JS (route D)
import web:ext/web/web.mq.md
---

# main

Server page with a counter UI. Client logic lives in `static/client.mq.md`
(loaded by official bridge via `client_embed`).

*`embed` = > web.client_embed bridge="/static/marqdo-bridge.js" wasm="/static/marqdo_wasm.wasm" source="/static/client.mq.md" boot=True*

*`intro` = "<h1>Marqdo client site</h1><p>SSR + WASM session. No author JS.</p><div id=\"count\">0</div><p><button id=\"bump\" type=\"button\" disabled>Bump</button> <button id=\"reset\" type=\"button\" disabled>Reset</button></p><pre id=\"log\">booting…</pre>" + embed*

*page = > web.page title="Marqdo client site" intro=intro*
*app = > web.app page=page host="127.0.0.1" port=18090*
*app = > app.static dir="static" mount="/static"*
> `app`.listen
