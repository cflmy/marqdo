---
markup: occlusion
type: web
view: main
route: /
title: Room transport composed from Documents and Resources
import:
  web: ext/web/web.mq.md
  socket: ext/net/websocket.mq.md
  home: home.mq.md
---

This local demonstration serves a Markdown introduction and a native log control. The page is a Document; the room transport is an explicitly bound network resource. From the checkout root, call serve with --effects plugin,fs.read,network.write and --fs-root examples/web-ws-room. It blocks until the supervising process stops it. Connect a WebSocket client to /chat/lobby to join chat.room.lobby; presence uses the same native room protocol as other transports. This loopback-only example explicitly permits anonymous room access; a deployed site should attach authentication and retain the default private route policy.

## main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | The homepage View rows, without starting a server |

The Document's main function only assembles content. The introduction remains a Markdown leaf and the event log is one native control with a stable id. The parent keeps those responsibilities as separate nodes, so changing the transport does not require rewriting the page as HTML. The View renderer receives structured values, and reading the content requires the granted site root.

After selecting the homepage components, return *[Assemble the readable room page](home.nodes())*.

## serve

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| root | text | "examples/web-ws-room" | Explicit site directory relative to the current working directory |
| port | number | 18091 | Loopback server port |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Blocking listener result for the declared page and room route |

Use `root`="examples/web-ws-room" and `port`=18091 from the checkout root. Preparing the site performs Document discovery and one startup assembly before any socket opens. Then the network resource binds /chat/{id} using the chat.room.{id} room template and publishes presence messages. This intentionally public local demo sets require_auth=False; it is not a recommendation to expose anonymous production rooms. Keep each returned app value because binding updates are immutable. The final listener retains the original site root for callbacks and resources.

We **app = [Discover the site's Documents and Endpoints](web.prepare(root=`root`, host="127.0.0.1", port=`port`))**.
We **app = [Bind the room and its presence channel](socket.route(app=`app`, path="/chat/{id}", mode="room", room_key="chat.room.{id}", presence=True, require_auth=False))**.
After all bindings are ready, return *[Listen with the complete site resources](web.listen(app=`app`))*.
