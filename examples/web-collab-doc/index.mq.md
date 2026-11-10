---
markup: occlusion
type: web
view: main
route: /
title: Collaborative document — declared layout and room Resource
import:
  home: home.mq.md
  web: ext/web/web.mq.md
  socket: ext/net/websocket.mq.md
---

Open two controlled browser tabs to observe complete text snapshots and room presence. This loopback demonstration explicitly permits anonymous room access. It is not durable storage, a CRDT or a history replay service. Build the current browser assets into static before serving.

## main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Collaborative page View rows |

The Document delegates to the readable page assembly. Follow its imports to the Markdown introduction, native editor and status leaves; none assembles a raw HTML shell. The bridge is a structured Resource. Page assembly reads the selected site root and returns structure before any listener starts.

Return *[Assemble the native collaborative page](home.nodes())*.

## serve

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| root | text | "examples/web-collab-doc" | Explicit site directory relative to the checkout root |
| port | number | 18094 | Loopback server port |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Blocking listener for the declared page and collaboration room |

Use `root`="examples/web-collab-doc" and `port`=18094. Prepare the page and static mount, bind the network-owned /live/{id} room and its presence channel, then listen with the retained site context. Anonymous access is deliberate for this localhost demonstration; a deployed application should attach authentication and retain private room defaults. The client uses a relative WebSocket address, so changing this port keeps the same page/client contract.

We **app = [Discover the page and its static resources](web.prepare(root=`root`, host="127.0.0.1", port=`port`, static_dir="static"))** and **app = [Bind the declared snapshot and presence room](socket.route(app=`app`, path="/live/{id}", mode="room", room_key="doc.{id}", presence=True, require_auth=False))**.
Return *[Listen after layout and transport are ready](web.listen(app=`app`))*.
