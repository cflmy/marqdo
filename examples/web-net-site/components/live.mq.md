---
markup: occlusion
title: Native WebSocket widget
import:
  component: ext/web/component/index.mq.md
  client: ext/web/client/index.mq.md
  table: lib/table.mq.md
---

This component ends at simple Markdown and native controls. It owns no network listener or hand-written HTML. The client program supplies events through the official bridge; serve separately binds the native /live echo resource. Keep control ids stable when editing wording so the client contract stays readable.

## nodes

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | WebSocket heading, status, input/send controls, message log and bridge resource |

The heading explains the interaction. Status and output use independent plain-text targets, input supplies the send value, and the button starts disabled until a real connection opens. Values and attributes remain typed data. Each echoed message is appended by the Marqdo client state and shown as text, so message content cannot become HTML. This construction does not open a socket.

The bridge addresses are explicit resources under the site's static mount. Prepare the current WASM and bridge in public/generated before serving; copying an older bridge can change event behavior even when the page still renders. The source at public/client.mq.md is served as UTF-8 client code and runs in the official browser runtime. To inspect construction alone, call this file's nodes function with no effects: it returns typed structure and performs no network request.

We **heading = [Describe the widget in ordinary Markdown](component.markdown(text="## Live WebSocket", slot="live-heading"))**.

`status_attrs` =

| id | aria-live |
|---|---|
| live-status | polite |

`input_attrs` =

| id | placeholder | type |
|---|---|---|
| live-msg | type a message… | text |

`button_attrs` =

| id | type | disabled |
|---|---|---|
| live-send | button | True |

`output_attrs` =

| id | aria-live |
|---|---|
| live-out | polite |

We **status = [Keep connection state separate from messages](component.element(kind="text", value="connecting…", attrs=`status_attrs`, slot="live-status"))** and **input = [Describe the native message input](component.element(kind="input", attrs=`input_attrs`, slot="live-input"))** and **button = [Describe the initially disabled send action](component.element(kind="button", value="Send", attrs=`button_attrs`, slot="live-send"))**.
We **controls = [Keep the input beside its send button](table.concat(a=`input`, b=`button`))** and **controls = [Declare responsive control geometry](component.wrap(children=`controls`, kind="row", slot="live-controls"))** and **output = [Describe the plain message log](component.element(kind="pre", attrs=`output_attrs`, slot="live-output"))**.
We **body = [Start with the explanatory heading](table.concat(a=`heading`, b=`status`))** and **body = [Place interaction after status](table.concat(a=`body`, b=`controls`))** and **body = [Keep the echoed messages after controls](table.concat(a=`body`, b=`output`))**.
We **id = [Name the widget's existing fragment destination](table.put(in=None, at="id", value="live"))** and **widget = [Keep the terminal pieces in one semantic region](component.wrap(children=`body`, kind="section", attrs=`id`, slot="live"))**.
We **runtime = [Declare the official runtime and Marqdo client source](client.nodes(source="/static/client.mq.md", wasm="/static/generated/marqdo_wasm.wasm", bridge="/static/generated/marqdo-bridge.js"))**.
Return *[Retain widget and runtime as structured nodes](table.concat(a=`widget`, b=`runtime`))*.
