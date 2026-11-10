---
markup: occlusion
type: web
view: main
route: /
title: Marqdo SPA — Document layout and native client records
import:
  home: components/index.mq.md
  web: ext/web/web.mq.md
  http: ext/net/http.mq.md
---

The Document's main assembles View nodes; serve owns the process lifetime. Build current WASM assets into static before starting. From the checkout root, call serve with plugin,fs.read,network.write and --fs-root examples/web-spa-app. List, detail and edit behavior stays in static/client.mq.md, with native row records and the official bridge's list effect.

## main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Homepage View rows |

The page delegates layout to the component root. Follow that root into navigation and the three independent panels until reaching plain text, Markdown or a native control leaf. The runtime is a structured Resource. This function neither starts a server nor mixes client state into a page HTML string; serve later mounts the actual assets.

Return *[Assemble the inspectable homepage](home.nodes())*.

## serve

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| root | text | "examples/web-spa-app" | Explicit site directory from the checkout root |
| port | number | 18093 | Loopback listen port |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Blocking site listener result |

Use `root`="examples/web-spa-app" and `port`=18093. The site prepares declared Documents and Endpoints, mounts static, then binds the existing /item redirect through the network resource. The explicit localhost address matches this demonstration's scope. Browser transitions use history inside the page; the server redirect preserves the old direct /item entry behavior. This function blocks until its supervisor stops it, and missing assets remain a real build prerequisite.

We **app = [Prepare the declared page and static mount](web.prepare(root=`root`, host="127.0.0.1", port=`port`, static_dir="static"))** and **app = [Preserve the declared direct-entry redirect](http.redirect(app=`app`, from="/item", to="/", permanent=False))**.
Return *[Listen after all resources are bound](web.listen(app=`app`))*.
