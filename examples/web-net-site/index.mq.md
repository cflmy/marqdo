---
markup: occlusion
type: web
view: main
route: /
title: Marqdo Net Site
import:
  layout: components/index.mq.md
  home: components/home.mq.md
  database: db/index.mq.md
  forms: forms.mq.md
  styles: styles/shell.mq.md
  web: ext/web/web.mq.md
  form: ext/data/form.mq.md
  admin: ext/data/admin.mq.md
  auth: ext/security/auth.mq.md
  socket: ext/net/websocket.mq.md
  fs: lib/fs.mq.md
  result: lib/result.mq.md
---

Read this entry, then the layout index, content leaves, database entry, form resource and browser client. Home, About, Publish and Tools are declared Documents. Native administration remains enabled and login-gated. The /live socket is a separate network resource. No page uses the removed compose APIs or a raw HTML shell.

## main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Shared recursive frame around introduction, echo controls and current articles |

This startup function describes structure, reading only the terminal Markdown leaves. The database query remains a typed resource resolved per request, while the client uses native controls and the official bridge. Rendering the page cannot initialize or clear business records. Follow each component import to edit one responsibility instead of reproducing the complete page.

We **body = [Assemble the three independent home responsibilities](home.nodes())**.
Return *[Place them in the shared recursive site frame](layout.frame(body=`body`))*.

## serve

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| root | text | "examples/web-net-site" | Explicit discovery directory relative to the checkout |
| port | number | 18083 | Local demonstration listen port |
| url | any | None | None chooses the site-root SQLite file; tests use an owned temporary URI |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Blocking listener with retained article data, forms, authentication and echo |

Use `root`="examples/web-net-site", `port`=18083 and `url`=None from the checkout, with --fs-root examples/web-net-site and plugin,fs.write,network.write. Build current browser assets into public/generated through the development handbook first. Missing runtime assets stop before listening rather than leaving the widget apparently ready.

Initialization preserves existing articles and seeds only an empty table. Theme compilation uses the original property declarations and writes only the explicit generated stylesheet. Discovery assembles the readable Documents, then the workflow enables native administration, attaches local fixture credentials, mounts the public article form and binds anonymous loopback echo. Real deployment must replace fixture credentials and its network policy. Keep each returned app before listening.

`assets` =

| file |
|---|
| public/generated/marqdo_wasm.wasm |
| public/generated/marqdo-bridge.js |

- [Require the actual runtime resources before serving](for assets as file)
  1. not [The selected runtime asset exists](fs.exists(path=`file`))
    Return *[Explain the required browser preparation](result.error(code="net_site.browser_assets", message="build current browser assets into public/generated before serve", operation="serve"))*.
We **store = [Initialize the retained article database](database.open(url=`url`))** and **css = [Compile the actual declarative theme](styles.css())** and **[Create only the generated resource directory](fs.make_dirs(path="public/generated"))** and **[Write that explicit stylesheet through native IO](fs.atomic_write(path="public/generated/theme.css", text=`css`))**.
We **app = [Discover all declared Documents and the static root](web.prepare(root=`root`, host="127.0.0.1", port=`port`, db=`store`, static_dir="public"))** and **app = [Retain the existing native administration capability](admin.enable(app=`app`))**.

`users` =

| @ | username | password | role |
|---|---|---|---|
| 1 | admin | secret | admin |

We **app = [Attach actual login sessions and the native admin gate](auth.attach(app=`app`, users=`users`, login_path="/admin/login", login_redirect="/admin", logout_redirect="/"))** and **article = [Build the shared validation resource](forms.article())** and **app = [Mount the public creation form explicitly](form.mount(app=`app`, id="article", form=`article`))**.
We **app = [Bind the original local WebSocket echo route](socket.route(app=`app`, path="/live", mode="echo", require_auth=False))**.
Return *[Listen only after data, resources and policy are ready](web.listen(app=`app`))*.
