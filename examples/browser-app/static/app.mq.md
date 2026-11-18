---
markup: occlusion
title: Browser effects — native records and explicit completion
description: list CRUD, storage, navigate, interval, fetch_all, ws — GFM + lib/browser
import:
  browser: lib/browser.mq.md
  table: lib/table.mq.md
  text: lib/text.mq.md
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Initial browser view/state record |

This entry prepares browser instructions; it does not itself fetch a URL or manipulate a server filesystem. The host page must supply the selectors listed below. List state uses native records with stable ids, keeping duplicate text and literal newlines independent. The wire table maps each UI event to one actual callback, including delegated item buttons, history navigation, timer controls, parallel fetches and a named WebSocket connection.

Event declarations contain no user values interpolated into code. Input values flow through callback parameters while the server page remains a separate document.

The returned boot record combines event registration and a storage lookup, without pushing an initial history entry. The browser bridge applies those instructions and dispatches later events into the callbacks. Draft text, tick count, route and socket identifier belong to that browser instance. Storage callbacks complete asynchronously; the initial return is not proof that a saved draft was found.

We **ticks = 0** as the retained count before the first real timer event.
We **items = [Start native list state](table.empty_list())** and **next_id = 1** for independent record identities.
We **route_path = "/"** as this browser instance’s panel state.
We **ws_id = "echo"** as the one named local connection.

`wire` =

| @ | 选择器 | 事件 | 调用 | 委托 | 值选择器 |
|---|--------|------|------|------|----------|
| 1 | "#add" | click | add | | "#draft" |
| 2 | "#save" | click | save_draft | | "#draft" |
| 3 | "#load" | click | load_draft | | |
| 4 | "#start-tick" | click | start_tick | | |
| 5 | "#stop-tick" | click | stop_tick | | |
| 6 | "#nav-home" | click | go_home | | |
| 7 | "#nav-about" | click | go_about | | |
| 8 | "#nav-net" | click | go_net | | |
| 9 | "window" | popstate | on_pop | | |
| 10 | "#items" | click | on_item | "button[data-id]" | |
| 11 | "#fetch-all" | click | do_fetch_all | | |
| 12 | "#ws-open" | click | ws_open | | |
| 13 | "#ws-send" | click | ws_send | | "#ws-msg" |
| 14 | "#ws-close" | click | ws_close | | |
| 15 | "#copy-log" | click | copy_log | | |
| 16 | "#dl-list" | click | dl_list | | |

`st` =

| op | key | then |
|----|-----|------|
| get | mq-draft | on_draft |

`boot` =

| wire | storage |
|---|---|
| `wire` | `st` |

After these steps, return *boot*.

## add

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | text | "" | Exact draft text supplied by the add control |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Native list refresh, input clear and focus; None for empty text |

Use `value`="" from the input. Retain each nonempty value exactly as one record, including quotes, angle brackets or a literal newline. A monotonically assigned id separates duplicate titles; deleting one must not erase its siblings. This direct callback updates entry state, then passes those records explicitly to the pure renderer. No business value is interpolated into HTML.

The input clears and regains focus after a real local addition. An empty string returns None without changing records or consuming an id. This list is session memory; only the separately saved draft uses browser storage.

1. `value` == ""
  Return *None* because there is no new item.
We **id = [Assign an independent stable identity](str(next_id))** and **next_id = next_id + 1**.

`record` =

| id | text |
|---|---|
| `id` | `value` |

We **items = [Retain the actual text record](table.append(list=`items`, item=`record`))** and **view = [Render typed labels and delete controls](_list(records=`items`))** and **clear = [Clear the completed input](browser.set_value(sel="#draft", value=""))** and **focus = [Return to the draft control](browser.focus(sel="#draft"))**.
We **view = [Keep the independent list and value effects](browser.merge(a=`view`, b=`clear`))**.
Return *[Keep focus with the completed list update](browser.merge(a=`view`, b=`focus`))*.

## on_item

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| data_id | text | "" | Stable id supplied by the delegated delete button |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Remaining native records and removal feedback, or None for an unknown id |

Use `data_id`="" from button[data-id]. Compare identities rather than text, so duplicate values and values containing newlines stay independent. Preserve remaining order and do nothing for an empty or unknown id. The renderer receives retained records explicitly; it does not infer client state inside a normal helper call.

A deletion rebuilds the same list through typed controls and reports the removed id as text. It neither modifies a saved draft nor changes timer, transport or route state. Ids are not recycled after deletion, avoiding a stale event selecting a different later item.

1. `data_id` == ""
  Return *None* for an absent selection.
We **remaining = [Start the retained row order](table.empty_list())**.
- [Keep every record except the selected identity](for items as item)
  1. access(`item`, "id") != `data_id`
    We **remaining = [Keep this independent record](table.append(list=`remaining`, item=`item`))**.
1. len(`remaining`) == len(`items`)
  Return *None* because no selected identity exists.
We **items = remaining** and **view = [Render the actual retained rows](_list(records=`items`))** and **message = "removed " + data_id** and **status = [Describe the observed local removal](browser.set_text(sel="#log", text=`message`))**.
Return *[Keep list and feedback together](browser.merge(a=`view`, b=`status`))*.

## save_draft

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | text | "" | Exact text selected from the draft input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Storage write request; completion arrives at on_saved |

Use `value`="" from the draft control. The storage record names the local key and its completion callback. Return the request immediately; only on_saved can announce a successful write. An intentionally empty draft is valid and differs from an absent key. Storage belongs to this browser origin, while list records stay in session memory. Returning this record does not write server files or grant process access.

`request` =

| op | key | value | then |
|---|---|---|---|
| set | mq-draft | `value` | on_saved |

Return *[Ask the browser to persist the exact draft](browser.storage(spec=`request`))*.

## load_draft

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Storage read request with an explicit completion callback |

Request mq-draft from this browser origin. The callback distinguishes a missing key, an empty saved value and a failed read. No input is cleared while the read is pending, and this operation never replaces the native item list. The same read runs at boot, so a reload restores the saved draft without claiming to persist list records.

`request` =

| op | key | then |
|---|---|---|
| get | mq-draft | on_draft |

Return *[Request the stored draft and its actual outcome](browser.storage(spec=`request`))*.

## on_draft

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| ok | bool | True | Actual storage-read completion |
| value | text | "" | Stored draft text, including an intentionally empty value |
| found | bool | False | Whether the key actually existed |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Restored value and feedback, or actual missing/failure feedback |

Use `ok`=True, `value`="" and `found`=False from storage. A missing key differs from a stored empty draft, and an unsuccessful read must not be described as absence. Restore the exact value only after a successful read with found=True. This changes the draft input, not the native item list, and never interprets the stored text as code.

1. not `ok`
  Return *[Keep a failed read visible](browser.set_text(sel="#log", text="draft read failed"))*.
1. not `found`
  Return *[Keep an absent draft visible](browser.set_text(sel="#log", text="no draft"))*.
We **value_effect = [Restore exact stored draft text](browser.set_value(sel="#draft", value=`value`))** and **status = [Confirm the observed read](browser.set_text(sel="#log", text="draft loaded"))**.
Return *[Keep the actual value and feedback](browser.merge(a=`value_effect`, b=`status`))*.

## start_tick

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | One named repeating timer instruction |

Start tick at one-second intervals. The bridge replaces a timer with the same id, so repeated Start clicks do not accumulate independent clocks. Each callback increments the session counter once. This returns an instruction; it is neither a blocking sleep nor proof that one second has passed. Stop cancels future callbacks and preserves the accumulated display.

Return *[Start or replace the one named timer](browser.interval(ms=1000, then="on_tick", id="tick"))*.

## stop_tick

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Cancellation of the named timer |

Cancel tick through its explicit id. Calling Stop before Start is harmless, and stopping does not reset the visible count or alter other timers. A later Start resumes counting from retained state. The browser owns the timer handle; authors do not keep a JavaScript handle or generate a client program to clear it.

Return *[Cancel future callbacks for this timer](browser.clear_interval(id="tick"))*.

## on_tick

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| ok | bool | True | Whether this timer callback is valid |
| id | text | "" | Actual named timer id |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Updated count, or None for a failed/unrelated callback |

Use `ok`=True and `id`="" from the timer. Count only successful events belonging to tick; unrelated callbacks must not change this counter. Retained ticks belongs to the direct browser callback, rather than being supplied by the event. A manual call to this function constructs the same text effect but does not prove that a real timer fired.

1. not `ok` or `id` != "tick"
  Return *None* for an unrelated timer outcome.
We **ticks = ticks + 1** and **label = [Format the actual accumulated count](str(ticks))**.
Return *[Display that observed timer count](browser.set_text(sel="#tick", text=`label`))*.

## go_home

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Selected panel effects and one actual navigation request |

Select / from its authored navigation control. This direct callback updates the retained panel state and requests exactly one history entry. The helper receives the path explicitly rather than assuming an ordinary call captures entry bindings. Back/Forward uses on_pop, which renders without navigation. No server request or page HTML is constructed here.

We **route_path = "/"**.
Return *[Render and request this one deliberate navigation](_view(route=`route_path`, push=True))*.

## go_about

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Selected panel effects and one actual navigation request |

Select /about from its authored navigation control. This direct callback updates the retained panel state and requests exactly one history entry. The helper receives the path explicitly rather than assuming an ordinary call captures entry bindings. Back/Forward uses on_pop, which renders without navigation. No server request or page HTML is constructed here.

We **route_path = "/about"**.
Return *[Render and request this one deliberate navigation](_view(route=`route_path`, push=True))*.

## go_net

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Selected panel effects and one actual navigation request |

Select /net from its authored navigation control. This direct callback updates the retained panel state and requests exactly one history entry. The helper receives the path explicitly rather than assuming an ordinary call captures entry bindings. Back/Forward uses on_pop, which renders without navigation. No server request or page HTML is constructed here.

We **route_path = "/net"**.
Return *[Render and request this one deliberate navigation](_view(route=`route_path`, push=True))*.

## on_pop

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | "" | Actual browser pathname, including optional query string |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | History-selected panel effects without pushing history again |

Use `path`="" from the real popstate callback. Keep its normalized panel route in the differently named entry state, avoiding accidental shadowing of a callback parameter. The browser has already selected this history entry, so returning navigate would add an incorrect new entry. Query text does not choose a different panel; only the known pathname is used.

We **parts = [Separate the pathname from query text](text.str_split(s=`path`, sep="?"))** and **route_path = [Keep the actual panel path](table.first(list=`parts`))**.
Return *[Reflect the actual history position](_view(route=`route_path`))*.

## show_path

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Current panel visibility and label without history mutation |

This direct browser callback reads retained route_path and delegates pure rendering with explicit input. It never pushes another history entry, so a repaint cannot trap Back/Forward. Call the navigation callbacks to request a new route. Server routing and page structure remain separate responsibilities; unknown browser paths select the home panel.

Return *[Refresh the retained panel without navigating](_view(route=`route_path`))*.

## do_fetch_all

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | The value explicitly returned below; conditional returns follow the selected branch |

Queue two independent browser fetches and return an immediate loading message together with the request instruction. Results arrive through on_fetch_all; this initial return does not contain either HTTP response. The request table keeps both URLs visible and avoids generating JavaScript to construct the batch.

The example endpoint is external and may fail or be unavailable. Consumers must inspect the actual callback records rather than count request declarations as successful responses. Browser network policy still applies; no backend file or shell operation is authorized by this instruction.

`reqs` =

| @ | url |
|---|-----|
| 1 | "https://httpbin.org/uuid" |
| 2 | "https://httpbin.org/uuid" |

We **msg = [Describe the pending batch](browser.set_text(sel="#net-out", text="fetch_all…"))** and **requests = [Request both actual HTTP outcomes](browser.fetch_all(requests=`reqs`, then="on_fetch_all"))**.
Return *[Keep pending feedback with its asynchronous requests](browser.merge(a=`msg`, b=`requests`))*.

## on_fetch_all

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| ok | bool | True | Whether the batch callback itself completed |
| results | any | None | Ordered per-request ok/status/body/error records |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Actual per-request outcomes shown as plain text |

Use `ok`=True and `results`=None from the bridge. A completed batch can still contain failed HTTP or network operations; the outer callback flag does not make each record successful. Show the real ordered records, including status, body and errors, rather than a fixed completion message. Text output cannot become HTML or a browser command.

A missing result list stays visibly unavailable. Public endpoints remain demonstration dependencies; local tests supply controlled responses and a failure. This callback does not retry a request or replace its body with a predetermined answer.

1. not `ok`
  Return *[Keep batch failure visible](browser.set_text(sel="#net-out", text="fetch_all failed"))*.
1. `results` == None
  Return *[Keep absent results distinct](browser.set_text(sel="#net-out", text="fetch_all returned no results"))*.
We **message = [Retain actual request outcomes](str(results))**.
Return *[Display observed successes and failures literally](browser.set_text(sel="#net-out", text=`message`))*.

## ws_open

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Named local WebSocket request and lifecycle callbacks |

Open the site’s /live echo resource under the explicit echo id. A relative URL uses the current origin and its transport security, allowing the example to run without a public echo provider. The server separately binds this native network resource. Returning the record only requests a connection; open, message, error and close callbacks describe actual transport outcomes. Subsequent send and close instructions reuse the same id.

`request` =

| op | id | url | then_open | then_message | then_error | then_close |
|---|---|---|---|---|---|---|---|
| open | `ws_id` | "/live" | on_ws_open | on_ws_msg | on_ws_err | on_ws_close |

Return *[Request the local echo connection](browser.ws(spec=`request`))*.

## on_ws_open

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| ok | bool | True | Actual connection completion |
| id | text | "" | Connected socket id |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Confirmed connection text, or None for a failed/unrelated callback |

Use `ok`=True and `id`="" from the real transport. Accept only the named echo socket; a different resource cannot announce this widget ready. This callback confirms the handshake, while sending still requires a separate user action. It neither invents a received message nor marks a requested connection successful before the bridge reports it.

1. not `ok` or `id` != `ws_id`
  Return *None* for an unrelated connection.
Return *[Confirm the observed handshake](browser.set_text(sel="#net-out", text="ws open"))*.

## on_ws_msg

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| ok | bool | True | Actual message completion |
| data | text | "" | Exact received text |
| id | text | "" | Receiving socket id |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Literal received message, or None for a failed/unrelated callback |

Use `ok`=True, `data`="" and `id`="" from the echo connection. Preserve the complete received text, including quotes and angle brackets. Display it through set_text so a message is data rather than page markup. Receipt is independent of the send click: the remote resource may respond later or close without replying.

1. not `ok` or `id` != `ws_id`
  Return *None* for an unrelated message.
We **message = "ws ← " + data**.
Return *[Display the actual echo as plain text](browser.set_text(sel="#net-out", text=`message`))*.

## on_ws_err

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| ok | bool | False | Failure flag supplied by the bridge |
| error | text | "" | Actual transport failure text |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Visible transport failure |

Use `ok`=False and `error`="" from the transport callback. Keep the actual error text in the output rather than substituting a success message. The callback performs no automatic reconnect and sends no pending message. Reopening is an explicit user operation; failures remain inspectable while the rest of the page can continue operating.

We **message = "ws error: " + error**.
Return *[Expose the actual connection failure](browser.set_text(sel="#net-out", text=`message`))*.

## on_ws_close

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| ok | bool | True | Actual close callback flag |
| id | text | "" | Closed socket id |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Observed close text, or None for an unrelated socket |

Use `ok`=True and `id`="" after the bridge observes closure. The named socket may close because of the user’s request or the server. This is a lifecycle observation and does not itself initiate another close or reopen. Leave list state, draft and timer untouched so transport changes remain independent of local page work.

1. `id` != `ws_id`
  Return *None* for an unrelated socket.
Return *[Report the observed connection closure](browser.set_text(sel="#net-out", text="ws closed"))*.

## ws_send

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | text | "" | Exact text selected from the message input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Send request for the existing named socket |

Use `value`="" as the message payload. Sending preserves literal text and addresses the existing echo socket; it does not create a second hidden connection. Open first and wait for the real connection callback. The request is not an echo result: only on_ws_msg displays a received response, and transport failures remain visible through the bridge.

`request` =

| op | id | data |
|---|---|---|
| send | `ws_id` | `value` |

Return *[Send exact text through the named connection](browser.ws(spec=`request`))*.

## ws_close

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Close request for the named socket |

Ask the bridge to close echo, reusing the same identity as open and send. The close callback supplies the observed lifecycle result. Returning this instruction does not delete page records or cancel the independent timer. Closing an absent connection is harmless; opening a later connection remains an explicit user action.

`request` =

| op | id |
|---|---|
| close | `ws_id` |

Return *[Request closure of this connection](browser.ws(spec=`request`))*.

## copy_log

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Clipboard request and truthful queued feedback |

Export the retained item texts in their current order, including duplicate values and literal newlines. The native records remain unchanged. This browser capability is a request: policy, user activation or the destination environment may prevent completion. The bridge exposes errors, and the immediate feedback says queued because no completion callback confirms a final clipboard write or saved file.

We **plain = [Read the current literal list export](_plain(records=`items`))** and **request = [Request clipboard delivery](browser.clipboard(text=`plain`))** and **status = [Describe only the requested operation](browser.set_text(sel="#log", text="clipboard request queued"))**.
Return *[Keep the request and its truthful feedback](browser.merge(a=`request`, b=`status`))*.

## dl_list

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Download request and truthful queued feedback |

Export the retained item texts in their current order, including duplicate values and literal newlines. The native records remain unchanged. This browser capability is a request: policy, user activation or the destination environment may prevent completion. The bridge exposes errors, and the immediate feedback says queued because no completion callback confirms a final clipboard write or saved file.

We **plain = [Read the current literal list export](_plain(records=`items`))** and **request = [Request the plain-text download](browser.download(body=`plain`, filename="items.txt"))** and **status = [Describe only the requested operation](browser.set_text(sel="#log", text="download request queued"))**.
Return *[Keep the request and its truthful feedback](browser.merge(a=`request`, b=`status`))*.

## _list

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| records | list | required | Native id/text records in visible order |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Structured list effect with literal labels and native delete buttons |

Use `records` from the direct callback's retained state. Each row contains a span label and button as typed children. The button's data-id carries the independent record identity; visible business text stays in text, never html. The official bridge escapes it and preserves delegation without rebuilding the parent page.

This helper is pure and receives records explicitly. It does not read hidden entry state, delete anything or assume that a returned effect has already reached the DOM. The existing host list and event wiring own those later steps.

We **rows = [Start the native visible row records](table.empty_list())**.
- [Describe one label and native delete action](for records as record)
  We **id = [Retain the exact row identity](access(record, "id"))** and **value = [Retain literal business text](access(record, "text"))**.

  `label` =

  | tag | text |
  |---|---|
  | span | `value` |

  `attributes` =

  | type | data-id |
  |---|---|
  | button | `id` |

  `button` =

  | tag | text | attrs |
  |---|---|---|
  | button | 删 | `attributes` |

  `children` =

  | node |
  |---|
  | `label` |
  | `button` |

  `row` =

  | tag | children |
  |---|---|
  | li | `children` |

  We **rows = [Retain this typed row](table.append(list=`rows`, item=`row`))**.
Return *[Update only the existing list](browser.render_list(sel="#items", items=`rows`))*.


## _view

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| route | text | required | Selected panel pathname |
| push | bool | False | Explicit navigation callback requests one history entry |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Actual boolean panel visibility, path label and optional deliberate navigation |

Use `route` and `push`=False. Only /about and /net select secondary panels; other paths render Home. Hidden attributes are actual booleans, while False removes the attribute. The pure helper receives its path explicitly and does not pretend to capture the browser entry state.

History observations use the default without navigation; authored link callbacks opt in once. Keep all three targets inside a single set_attr record because effect merging is shallow and otherwise one target map could overwrite another. This function performs no fetch, storage write or socket operation.

We **selected = "/"**.
1. `route` == "/about" or `route` == "/net"
  We **selected = route**.
We **home = [Describe Home visibility](table.put(in=None, at="hidden", value=(`selected` != "/")))** and **about = [Describe About visibility](table.put(in=None, at="hidden", value=(`selected` != "/about")))** and **network = [Describe Network visibility](table.put(in=None, at="hidden", value=(`selected` != "/net")))**.

`targets` =

| #panel-home | #panel-about | #panel-net |
|---|---|---|
| `home` | `about` | `network` |

We **visibility = [Keep every panel target together](browser.wrap(key="set_attr", value=`targets`))** and **message = "path: " + selected** and **label = [Display the selected path](browser.set_text(sel="#path", text=`message`))** and **effects = [Keep independent visibility and text effects](browser.merge(a=`visibility`, b=`label`))**.
1. `push`
  We **navigation = [Request one intentional history entry](browser.navigate(url=`selected`))** and **effects = [Keep navigation with its actual panel effects](browser.merge(a=`effects`, b=`navigation`))**.
Return *effects* after describing the real selected panel.


## _plain

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| records | list | required | Native text records in retained order |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | text | Literal newline-joined values for copy or download |

Use `records` to retain the original plain-text export purpose without turning the client state into a newline-encoded database. Each item's text remains a single native value; newlines are introduced only in this human-readable export. This pure transformation neither writes the clipboard nor starts a download, and it does not encode HTML.

We **values = [Select actual text fields in order](table.pluck(list=`records`, key="text"))**.
Return *[Form the intended text export](text.str_join(xs=`values`, sep="\n"))*.

## on_saved

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| ok | bool | True | Actual storage-write completion |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Feedback for the completed write |

Use `ok`=True from the storage callback. Confirm saving after the browser reports completion, rather than when the user merely clicks Save. A failed callback remains a failure message. Host storage exceptions also appear through the official bridge error channel; this function cannot turn an unavailable storage service into a successful write.

1. not `ok`
  Return *[Expose the failed write](browser.set_text(sel="#log", text="draft save failed"))*.
Return *[Confirm the completed write](browser.set_text(sel="#log", text="draft saved"))*.
