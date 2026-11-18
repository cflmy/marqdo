---
markup: occlusion
title: Collaborative client — native text snapshots and presence
import:
  browser: lib/browser.mq.md
  table: lib/table.mq.md
  text: lib/text.mq.md
---

This program owns document text and event transitions. The Document supplies the native textarea and status leaves; the network resource supplies a same-origin room. All business state stays in Marqdo.

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Client wiring, room connection and initial status effects |

The client keeps only the last locally observed document text in native entry state. Each edit sends a complete DOC-tab-text snapshot; this is a last-received-message demonstration, not a CRDT, document database or history synchronization service. The relative socket path follows the page's host and port, so a different loopback listen port does not require editing client code. Presence and document messages use separate displayed targets. The official bridge changes textarea.value without emitting a new input event, so applying remote text needs no flag that would discard the next real local edit. BrowserSession retains entry bindings between event callbacks.

We **ws_id = "doc"** and **doc = ""** before connecting.

`wire` =

| @ | 选择器 | 事件 | 调用 | 值选择器 |
|---|---|---|---|---|
| 1 | "#doc" | input | on_edit | "#doc" |

`open` =

| op | id | url | then_open | then_message | then_error | then_close |
|---|---|---|---|---|---|---|
| open | `ws_id` | /live/demo | on_open | on_msg | on_err | on_close |

We **wiring = [Bind the existing native textarea](table.put(in=None, at="wire", value=`wire`))** and **socket = [Describe the same-origin room connection](browser.ws(spec=`open`))** and **boot = [Keep wiring and transport together](browser.merge(a=`wiring`, b=`socket`))** and **ready = [Describe the initial connection status](browser.set_text(sel="#log", text="connecting…"))**.
Return *[Keep initial status with the boot instructions](browser.merge(a=`boot`, b=`ready`))*.

## on_open

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| ok | bool | True | Success flag from the bridge's open callback |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Plain connection status effect |

Use `ok`=True from the open callback. A successful transport opening says that the room is connected; it does not prove that previous document content was replayed. This callback updates only the log. Presence frames arrive independently through on_msg, and actual editing remains available through the existing textarea binding.

1. `ok`
  Return *[Show that the native room connection opened](browser.set_text(sel="#log", text="ws open · room demo"))*.
Return *[Keep an unsuccessful open visible](browser.set_text(sel="#log", text="ws open failed"))*.

## on_err

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| error | text | "" | Error text supplied by the browser transport |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Plain error text effect |

Use `error`="" from the bridge. Preserve its text in the status log rather than treating a failed connection as an empty successful document. This callback neither rewrites shared text nor retries a send, so the caller can observe the actual transport state without a hidden duplicate update.

We **message = "ws error: " + error**.
Return *[Display the actual transport failure as text](browser.set_text(sel="#log", text=`message`))*.

## on_close

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Closed connection status effect |

A close notification updates the log while retaining the last displayed document text. This demonstration does not silently reconnect or persist data elsewhere. Reopening the page starts a fresh client session, and subsequent edits need a live socket before peers can receive them.

Return *[Keep closure visible beside the editor](browser.set_text(sel="#log", text="ws closed"))*.

## on_edit

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | text | "" | Current plain textarea contents, including an intentionally empty document |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | One complete snapshot send instruction, or None for unchanged text |

Use `value`="" from the input binding. A changed value replaces the native observed text and becomes one DOC-tab snapshot; an empty value is a valid deletion, not a missing edit. Identical input produces no send. Incoming set_value effects do not emit input, so the next real user edit after a remote update must be sent immediately. The callback describes a browser WebSocket action and opens no native socket itself.

1. `value` == `doc`
  Return *None* because the observed text is unchanged.
We **doc = [Retain the complete new native text](value)** and **payload = "DOC\t" + value**.

`send` =

| op | id | data |
|---|---|---|
| send | `ws_id` | `payload` |

Return *[Send the complete snapshot through the existing room connection](browser.ws(spec=`send`))*.

## on_msg

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| data | text | "" | One text frame from the room or its presence channel |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Presence text, a remote textarea value, or None for ignored/echoed frames |

Use `data`="" from the transport callback. Presence records are displayed separately and never interpreted as document text. A DOC-tab frame contains the entire replacement document, including a possible empty string. Retain that observed text before returning its set_value instruction; an echoed snapshot equal to local state then becomes a no-op. Unknown messages and empty frames leave content unchanged. No JavaScript, HTML or external interpreter parses the document payload.

1. `data` == ""
  Return *None* for an empty frame.
1. [A native presence record](text.starts_with(text=`data`, prefix="{"))
  We **message = "presence: " + data**.
  Return *[Display room presence independently](browser.set_text(sel="#presence", text=`message`))*.
1. [A complete document snapshot](text.starts_with(text=`data`, prefix="DOC\t"))
  We **remote = [Remove only the leading protocol marker](text.replace(text=`data`, old="DOC\t", new="", count=1))**.
  1. `remote` != `doc`
    We **doc = [Retain the received native text](remote)**.
    Return *[Apply remote text without manufacturing an input event](browser.set_value(sel="#doc", value=`remote`))*.
Otherwise return *None* for unrelated frames or an already observed snapshot.
