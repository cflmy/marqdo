---
title: web-spa-app
description: Native Marqdo SPA — list/detail/edit via WASM client routes + memory store
import web:ext/web/web.mq.md
---

# main

Full-page client app (zero author JS). Build assets first:

```
marqdo wasm build -o examples/web-spa-app/static
```

**`embed` = > web.client_embed bridge="/static/marqdo-bridge.js" wasm="/static/marqdo_wasm.wasm" source="/static/client.mq.md" boot=True**

**`intro` = "<nav class='spa-nav'><button type='button' id='nav-list'>Items</button> <button type='button' id='nav-new'>New</button></nav><p id='title' class='spa-title'>Items</p><section id='view-list'><ul id='items'></ul><p class='hint'>Click an item for detail.</p></section><section id='view-detail' hidden><p id='detail-body'></p><p><button type='button' id='btn-edit'>Edit</button> <button type='button' id='btn-back'>Back</button></p></section><section id='view-edit' hidden><label>Title <input id='edit-title' /></label><p><button type='button' id='btn-save'>Save</button> <button type='button' id='btn-cancel'>Cancel</button></p></section><pre id='log'></pre>" + embed**

**page = > web.page title="Marqdo SPA" intro=intro layout="bare"**
**app = > web.app page=page host="127.0.0.1" port=18093**
**app = > app.static dir="static" mount="/static"**
**app = > app.redirect from="/item" to="/" permanent=False**
> `app`.listen
