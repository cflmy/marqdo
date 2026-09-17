---
title: web WS room + on_message bag smoke
description: route_ws mode=room stores room_key, on_message, presence
import web:ext/web/web.mq.md
import json:lib/json.mq.md
import sys:lib/sys.mq.md
---

# main

**首页 = > web.page title="ws"**
**应用 = > web.app page=`首页` admin=False**
**应用 = > `应用`.route_ws path="/chat/{id}" mode="room" room_key="chat.room.{id}" on_message="chat.persist" presence=True**
**routes = > json.get value=`应用` key="ws_routes"**
**spec = > json.get value=`routes` key="/chat/{id}"**
**mode = > json.get value=`spec` key="mode"**
**rk = > json.get value=`spec` key="room_key"**
**om = > json.get value=`spec` key="on_message"**
**pr = > json.get value=`spec` key="presence"**
1. `mode` == "room"
  1. `rk` == "chat.room.{id}"
    1. `om` == "chat.persist"
      1. `pr` == True
        > print text=ws-room-bag-ok
      2. *
        > print text=presence-fail
        > sys.exit code=1
    2. *
      > print text=on-message-fail
      > sys.exit code=1
  2. *
    > print text=room-key-fail
    > sys.exit code=1
2. *
  > print text=mode-fail
  > sys.exit code=1
