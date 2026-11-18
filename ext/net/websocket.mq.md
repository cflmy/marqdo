---
markup: occlusion
title: ext/net/websocket
description: WebSocket client Resource (ADR 0007).
import:
  plugin: lib/plugin.mq.md
  sys: lib/sys.mq.md
  table: lib/table.mq.md
---

This resource performs one bounded native WebSocket exchange and closes it. The [network entry](index.mq.md) shows the input contract and the distinction from pure URL encoding. Configure a controlled ws/wss endpoint and inspect ok/error, timeout and reply data before acting. New transport behavior belongs in the web plugin, with local tests for failed connections and oversized replies; this resource is not a persistent subscription service.

## _load

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | No business value on completion; a missing native plugin reports the cause and exits this run. |

Resolve and load the native web plugin before resource operations; missing binaries stop this run.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="web"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/net: native web plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.
Return *None*.

## route

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| app | any | required | Prepared application returned by web.prepare |
| path | text | required | Declared WebSocket path, including optional path captures |
| mode | text | "echo" | echo, drain, broadcast or room |
| room_key | text | "" | Room template required by room mode |
| presence | bool | False | Publish native room presence messages |
| require_auth | any | None | None keeps private broadcast/room defaults; False explicitly opens a local demo |
| on_message | text | "" | Optional declared Marqdo callback member |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Updated app with this explicitly declared WebSocket route |

Use `app`, `path`, `mode`="echo", `room_key`="", `presence`=False, `require_auth`=None and `on_message`="" to bind a server transport before web.listen. This function belongs to the network resource; it does not change Document layout or start a socket. Retain the returned app because native resource updates produce a new value. Room mode requires a room_key template; captures such as {id} select the room at request time. Broadcast and room routes require login unless the caller explicitly opts out. Public local demos should say why require_auth=False is intentional. Presence and callbacks retain the native transport's bounded message handling; they are not a client subscription object.

We **[Load the native transport implementation](_load())**.
Return *[Register this declared WebSocket transport](web_app_route_ws(app=`app`, path=`path`, echo=None, mode=`mode`, room_key=`room_key`, on_message=`on_message`, presence=`presence`, require_auth=`require_auth`))*.


# ws

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| timeout_sec | any | 30 | Request/response timeout in seconds |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | WebSocket client helper. |

Use `timeout_sec`=30 as the inputs for this operation.

WebSocket client helper.

We **[Load the native plugin required by this operation](_load())**.

`out` =

| timeout_sec | _type |
|-------------|-------|
| `timeout_sec` | ws |

Return *out*.

## connect

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| url | any | required | Resource connection URL; scheme selects the documented backend |
| message | any | "" | Outgoing WebSocket or application message |
| headers | any | None | HTTP request header record or None |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Single request–response: connect to `url`, send `message`, collect up to 256 server text replies (each up to 1 MiB), close. Transport errors and timeouts return `{ok:false, error}` instead of looking like an empty successful reply. |

Use `url`, `message`="", `headers`=None as the inputs for this operation.

Single request–response: connect to `url`, send `message`, collect up to 256 server text replies (each up to 1 MiB), close. Transport errors and timeouts return `{ok:false, error}` instead of looking like an empty successful reply.

We retain **`timeout` = [Read timeout_sec field](access(`self`, "timeout_sec"))**.
Return *[Single request–response: connect to url, send message, collect up to 256 server text replies (each up to 1 MiB), close](web_ws_connect(url=`url`, message=`message`, headers=`headers`, timeout_sec=`timeout`))*.
