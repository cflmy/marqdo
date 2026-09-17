---
title: web-ws-room demo
description: Dynamic room WS with presence (G-WS1/2 bag). Open / and connect via bridge or wscat.
import web:ext/web/web.mq.md
---

# main

**page = > web.page title="WS room" intro="<p class='lede'>Chat room <code>lobby</code> via <code>/chat/lobby</code>. Presence joins <code>chat.room.lobby.presence</code>.</p><pre id='log'></pre>"**
**app = > web.app page=`page` host="127.0.0.1" port=18091 admin=False**
**app = > `app`.route_ws path="/chat/{id}" mode="room" room_key="chat.room.{id}" presence=True**
> `app`.listen
