---
markup: occlusion
title: Network-site echo client
import:
  browser: lib/browser.mq.md
  table: lib/table.mq.md
  text: lib/text.mq.md
---

The client owns connection state and a plain text message log. Native controls are declared by components/live.mq.md. The relative socket path follows the page's actual host and port, while the server resource owns echo behavior.

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Native event wiring, socket opening and initial status effects |

The button click carries the current input value to send. A connection opening enables the button; closure or error disables it. Incoming text appends to retained entry state and never becomes HTML. This is a live echo demonstration, not durable storage or an automatic reconnection loop. BrowserSession keeps entry bindings for callbacks, so they are state rather than fictitious public parameters.

We **ws_id = "live"** and **messages = ""** before describing the transport.

`wire` =

| @ | 选择器 | 事件 | 调用 | 值选择器 |
|---|---|---|---|---|
| 1 | "#live-send" | click | send | "#live-msg" |
| 2 | "#live-msg" | keydown | on_key | "#live-msg" |

`open` =

| op | id | url | then_open | then_message | then_error | then_close |
|---|---|---|---|---|---|---|
| open | `ws_id` | /live | on_open | on_message | on_error | on_close |

We **bindings = [Bind the existing send control](table.put(in=None, at="wire", value=`wire`))** and **socket = [Describe the same-origin WebSocket connection](browser.ws(spec=`open`))**.
Return *[Keep transport and native control wiring together](browser.merge(a=`bindings`, b=`socket`))*.

## _status

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Plain connection status to display |
| disabled | bool | required | Actual native disabled state for the send button |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Combined status-text and button-attribute effects |

Use `text` and `disabled` to keep connection feedback consistent with the send control. The attribute must remain a boolean: the bridge removes disabled when False, while the string "false" would still be a present HTML boolean attribute. These effects describe a DOM update and do not send or retry a message.

We **attributes = [Keep the actual boolean attribute value](table.put(in=None, at="disabled", value=`disabled`))** and **status = [Describe plain connection feedback](browser.set_text(sel="#live-status", text=`text`))** and **button = [Describe actual send availability](browser.set_attr(sel="#live-send", attrs=`attributes`))**.
Return *[Keep the independent effect kinds together](browser.merge(a=`status`, b=`button`))*.

## on_open

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| ok | bool | True | Actual browser transport opening result |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Connected feedback and enabled send, or failed feedback and disabled send |

Use `ok`=True from the opening callback. A successful opening proves only the transport is ready; an echoed message is observed separately. Failed opening keeps send disabled and visible as failure. The client does not publish a fake initial echo or silently retry.

1. `ok`
  Return *[Enable sending only after transport opens](_status(text="connected", disabled=False))*.
Return *[Keep failed opening visible](_status(text="connection failed", disabled=True))*.

## _send

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | text | required | Input text supplied by a click or Enter callback |
| channel | text | required | Existing socket id retained by the client entry |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | One actual send instruction, or None for empty trimmed input |

Use `value` and `channel` to share the original widget's send policy. Trim surrounding whitespace and ignore an empty result. A nonempty message remains plain text, never a command or HTML. The official bridge reports transport errors separately. This helper receives its channel explicitly rather than assuming an ordinary nested call automatically captures entry state.

We **payload = [Keep the original trim-before-send behavior](text.str_trim(s=`value`))**.
1. `payload` == ""
  Return *None* because an empty or whitespace-only input performs no send.

`operation` =

| op | id | data |
|---|---|---|
| send | `channel` | `payload` |

Return *[Send one nonempty message through the existing connection](browser.ws(spec=`operation`))*.

## send

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | text | "" | Current text supplied by the send-button wiring |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Send effect for nonempty trimmed input, otherwise None |

Use `value`="" from a button click. The direct browser callback can read retained entry state and passes that socket id explicitly to the pure send helper. The log updates only after an actual echo arrives, so clicking cannot fabricate successful delivery. Whitespace-only input retains the original widget's no-send behavior.

Return *[Apply the shared send policy to this click](_send(value=`value`, channel=`ws_id`))*.

## on_key

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| key | text | "" | Actual key name supplied by the official bridge |
| value | text | "" | Current message input supplied by this control's value selector |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Enter uses the same send policy as the button; other keys return None |

Use `key`="" and `value`="" from the keyboard wiring. Preserve the original Enter shortcut without sending on every keydown. Enter and click share trimming and empty-input behavior, and neither appends a local echo before a real frame arrives. This callback does not install another DOM handler or start a second socket.

1. `key` == "Enter"
  Return *[Apply exactly the button's send policy](_send(value=`value`, channel=`ws_id`))*.
Return *None* for an ordinary editing key.

## on_message

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| data | text | "" | Actual echoed text frame from the server |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Plain text update containing the retained echo log |

Use `data`="" from the message callback. Append each real frame and a newline to entry state, preserving earlier messages as the old widget did. set_text escapes content through the bridge; a message containing markup is shown literally. Reloading starts a fresh demonstration log and no durable history is claimed.

We **messages = [Retain the next observed echo](messages + "echo: " + data + "\n")**.
Return *[Display the complete observed log as text](browser.set_text(sel="#live-out", text=`messages`))*.

## on_error

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| error | text | "" | Browser transport error description |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Error feedback and disabled sending |

Use `error`="" to retain the actual transport failure beside the controls. No previous message is erased and no new send is attempted. The page can be reloaded for a fresh connection; this demonstration has no hidden reconnect policy.

We **message = "error: " + error**.
Return *[Keep failure and send availability consistent](_status(text=`message`, disabled=True))*.

## on_close

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Closed feedback with disabled send control |

A close notification keeps the message log readable while marking the socket unavailable. It neither clears entry state nor silently creates another connection. Subsequent sends require a newly opened page session rather than a fabricated successful transport.

Return *[Keep closure visible](_status(text="closed", disabled=True))*.
