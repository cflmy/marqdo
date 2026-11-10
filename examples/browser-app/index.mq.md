---
markup: occlusion
type: web
view: main
route: /
title: Marqdo browser-app
import:
  page: components/index.mq.md
  styles: styles/index.mq.md
  web: ext/web/web.mq.md
  http: ext/net/http.mq.md
  socket: ext/net/websocket.mq.md
  fs: lib/fs.mq.md
  result: lib/result.mq.md
---

This is the callable engineering entry. Follow components/index.mq.md for recursive layout, static/app.mq.md for browser state and effects, and styles/index.mq.md for the retained theme. Plain content is Markdown; controls and resources remain native structured values. There is no authored HTML host or business JavaScript. Serve runs from the checkout root with --fs-root examples/browser-app.

## main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Inspectable native page rows |

The Document delegates page assembly to its component root. Home, About and Network are client panels within this one page, with their own independently readable components. The client controls visibility and history while ordinary content remains Markdown. This function does not start a server, fetch public endpoints or open a socket. Read the returned tree as layout data, and call serve for a real running application.

Return *[Assemble the recursive browser demonstration](page.nodes())*.

## serve

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| root | text | "examples/browser-app" | Explicit site directory relative to the checkout |
| port | number | 18095 | Loopback listen port |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Blocking listener or missing-asset failure |

Use `root`="examples/browser-app" and `port`=18095 with plugin,fs.write,network.write and the site filesystem root. Build current browser assets through the development handbook first. Both generated assets must exist before theme compilation or listening; missing files produce an actionable Result. The native filesystem writes only the explicit compiled stylesheet. No Python static server or generated shell workflow is needed.

Discovery prepares the Document and the static mount. Direct /about and /net visits redirect to the entry page, matching the transient panel-state model; Back and Forward inside a live page retain their actual history positions. A separate local network resource binds /live for anonymous demonstration echo. This loopback example does not authenticate or persist item records. Keep every returned app value and listen only after binding all resources.

`assets` =

| file |
|---|
| static/marqdo_wasm.wasm |
| static/marqdo-bridge.js |

- [Require current official browser resources](for assets as file)
  1. not [The required runtime asset exists](fs.exists(path=`file`))
    Return *[Expose the actual build prerequisite](result.error(code="browser_app.assets", message="call build_browser and stage its assets in examples/browser-app/static before serve", operation="serve"))*.
We **css = [Compile the retained declarative theme](styles.css())** and **[Write only the generated stylesheet](fs.atomic_write(path="static/theme.css", text=`css`))** and **app = [Discover the native Document and static assets](web.prepare(root=`root`, host="127.0.0.1", port=`port`, static_dir="static"))** and **app = [Retain a direct About entry](http.redirect(app=`app`, from="/about", to="/", permanent=False))** and **app = [Retain a direct Network entry](http.redirect(app=`app`, from="/net", to="/", permanent=False))** and **app = [Bind the actual local echo resource](socket.route(app=`app`, path="/live", mode="echo", require_auth=False))**.
Return *[Listen after the real resources are ready](web.listen(app=`app`))*.
