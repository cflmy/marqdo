---
markup: occlusion
title: lib/browser
description: >-
  Readable browser effect helpers for WASM clients. Prefer GFM tables + these
  helpers; do not build effects with json.set chains.
import:
  table: lib/table.mq.md
---

Readable browser effect helpers for WASM clients. Prefer GFM tables + these helpers; do not build effects with json.set chains.

Build browser effects as ordinary structured values.

Behavior and limits: Constructing an effect is pure; the official WASM bridge performs it in a browser. Native construction does not prove a DOM update occurred. Keep business logic in Marqdo.

This single source file is the usage and development manual. Import it directly, or use marqdo call with a function heading below. Parameter tables give types and defaults; required parameters must be supplied. Return tables are progressive contracts; prose describes failure and limits.

Implementation: [crates/marqdo-wasm/js/marqdo-bridge.js](../crates/marqdo-wasm/js/marqdo-bridge.js)

For development, keep EN/ZH behavior aligned and compose public libraries first. Add host primitives only at OS or algorithm boundaries and declare their effects. Run marqdo check --narrative-ratio on this file and the real workflows in tests/stdlib_tasks.rs after edits.

## set_text

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| sel | any | required | CSS selector of the target browser element |
| text | text | required | Text supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Build set_text effect map for the official bridge |

Use `sel`, `text` as the inputs for this operation.

Effects: pure.

Call this function directly:

```sh
marqdo call lib/browser.mq.md set_text 'sel="#status"' 'text="Ready"' --effects none --json
```

Build set_text effect map for the official bridge.

We retain **`m` = [Return the updated structured value](table.put(in=None, at=`sel`, value=`text`))**.
Return *[Set the set_text field in the new record](table.put(in=None, at="set_text", value=`m`))*.

## set_html

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| sel | any | required | CSS selector of the target browser element |
| html | any | required | HTML fragment; use set_text for ordinary untrusted text |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Build set_html effect map |

Use `sel`, `html` as the inputs for this operation.

Effects: pure.

Build set_html effect map.

We retain **`m` = [Return the updated structured value](table.put(in=None, at=`sel`, value=`html`))**.
Return *[Set the set_html field in the new record](table.put(in=None, at="set_html", value=`m`))*.

## set_value

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| sel | any | required | CSS selector of the target browser element |
| value | any | required | Value to transform or store, without mutating the input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Build set_value effect map |

Use `sel`, `value` as the inputs for this operation.

Effects: pure.

Build set_value effect map.

We retain **`m` = [Return the updated structured value](table.put(in=None, at=`sel`, value=`value`))**.
Return *[Set the set_value field in the new record](table.put(in=None, at="set_value", value=`m`))*.

## set_class

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| sel | any | required | CSS selector of the target browser element |
| class | any | required | Space-separated CSS class names |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Build set_class effect map |

Use `sel`, `class` as the inputs for this operation.

Effects: pure.

Build set_class effect map.

We retain **`m` = [Return the updated structured value](table.put(in=None, at=`sel`, value=`class`))**.
Return *[Set the set_class field in the new record](table.put(in=None, at="set_class", value=`m`))*.

## add_class

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| sel | any | required | CSS selector of the target browser element |
| class | any | required | Space-separated CSS class names |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Build add_class effect map (classList.add; space-separated names OK) |

Use `sel`, `class` as the inputs for this operation.

Effects: pure.

Build add_class effect map (classList.add; space-separated names OK).

We retain **`m` = [Return the updated structured value](table.put(in=None, at=`sel`, value=`class`))**.
Return *[Set the add_class field in the new record](table.put(in=None, at="add_class", value=`m`))*.

## remove_class

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| sel | any | required | CSS selector of the target browser element |
| class | any | required | Space-separated CSS class names |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Build remove_class effect map |

Use `sel`, `class` as the inputs for this operation.

Effects: pure.

Build remove_class effect map.

We retain **`m` = [Return the updated structured value](table.put(in=None, at=`sel`, value=`class`))**.
Return *[Set the remove_class field in the new record](table.put(in=None, at="remove_class", value=`m`))*.

## toggle_class

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| sel | any | required | CSS selector of the target browser element |
| class | any | required | Space-separated CSS class names |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Build toggle_class effect map |

Use `sel`, `class` as the inputs for this operation.

Effects: pure.

Build toggle_class effect map.

We retain **`m` = [Return the updated structured value](table.put(in=None, at=`sel`, value=`class`))**.
Return *[Set the toggle_class field in the new record](table.put(in=None, at="toggle_class", value=`m`))*.

## set_style

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| sel | any | required | CSS selector of the target browser element |
| style | any | required | CSS property/value record for the target element |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Build set_style effect map; style is a map of CSS properties or a cssText string |

Use `sel`, `style` as the inputs for this operation.

Effects: pure.

Build set_style effect map; style is a map of CSS properties or a cssText string.

We retain **`m` = [Return the updated structured value](table.put(in=None, at=`sel`, value=`style`))**.
Return *[Set the set_style field in the new record](table.put(in=None, at="set_style", value=`m`))*.

## set_attr

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| sel | any | required | CSS selector of the target browser element |
| attrs | any | required | HTML attribute/value record for the target element |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Build set_attr effect map; attrs is usually a small map of attribute names to values |

Use `sel`, `attrs` as the inputs for this operation.

Effects: pure.

Build set_attr effect map; attrs is usually a small map of attribute names to values.

We retain **`m` = [Return the updated structured value](table.put(in=None, at=`sel`, value=`attrs`))**.
Return *[Set the set_attr field in the new record](table.put(in=None, at="set_attr", value=`m`))*.

## wrap

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| key | text | required | Field name or key required by this operation |
| value | any | required | Value to transform or store, without mutating the input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Single-key effect map wrapper |

Use `key`, `value` as the inputs for this operation.

Effects: pure.

Single-key effect map wrapper.

Return *[Return the updated structured value](table.put(in=None, at=`key`, value=`value`))*.

## merge

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| a | any | required | First browser effect record |
| b | any | required | Browser effect record to merge over a |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Merge two effect maps; right-hand keys win |

Use `a`, `b` as the inputs for this operation.

Effects: pure.

Merge two effect maps; right-hand keys win.

Return *[Merge](table.merge(a=`a`, b=`b`))*.

## canvas

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| sel | any | required | CSS selector of the target browser element |
| commands | any | required | Ordered list of canvas drawing commands |
| clear | any | True | Whether the canvas is cleared before drawing commands |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Build canvas effect; commands is usually an @ GFM table |

Use `sel`, `commands`, `clear`=True as the inputs for this operation.

Effects: pure.

Build canvas effect; commands is usually an @ GFM table.

`spec` =

| sel | clear | commands |
|-----|-------|----------|
| `sel` | `clear` | `commands` |

Return *[Set the canvas field in the new record](table.put(in=None, at="canvas", value=`spec`))*.

## audio

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| op | any | "play" | Audio command: play, pause or stop |
| id | any | "default" | Audio channel identifier |
| src | text | "" | Audio URL, or beep for a synthesized tone |
| freq | any | None | Oscillator frequency in hertz |
| ms | any | None | Duration or delay in milliseconds |
| sel | any | "" | CSS selector of the target browser element |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Build audio effect for the bridge |

Use `op`="play", `id`="default", `src`="", `freq`=None, `ms`=None, `sel`="" as the inputs for this operation.

Effects: pure.

Build audio effect for the bridge.

This function returns an effect record and plays no sound immediately. The bridge selects a source, synthesized frequency or audio element; browsers may require a prior user gesture.

We retain **`spec` = [Set the op field in the new record](table.put(in=None, at="op", value=`op`))**.
We retain **`spec` = [Set the id field in the new record](table.put(in=`spec`, at="id", value=`id`))**.
1. `src` != ""
    We retain **`spec` = [Set the src field in the new record](table.put(in=`spec`, at="src", value=`src`))**.
1. `freq` != None
    We retain **`spec` = [Set the freq field in the new record](table.put(in=`spec`, at="freq", value=`freq`))**.
1. `ms` != None
    We retain **`spec` = [Set the ms field in the new record](table.put(in=`spec`, at="ms", value=`ms`))**.
1. `sel` != ""
    We retain **`spec` = [Set the sel field in the new record](table.put(in=`spec`, at="sel", value=`sel`))**.
Return *[Set the audio field in the new record](table.put(in=None, at="audio", value=`spec`))*.

## read_file

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| sel | any | required | CSS selector of the target browser element |
| then | any | required | Marqdo callback name for the bridge result; empty text omits callback |
| as | any | "text" | File content representation expected by the browser bridge |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Build read_file effect |

Use `sel`, `then`, `as`="text" as the inputs for this operation.

Effects: pure.

Build read_file effect.

`spec` =

| sel | as | then |
|-----|----|------|
| `sel` | `as` | `then` |

Return *[Set the read_file field in the new record](table.put(in=None, at="read_file", value=`spec`))*.

## observe

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| specs | any | required | Effect specifications; use a record or an @ row-record list |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Build observe effect; specs is one map or an @ list of maps |

Use `specs` as the inputs for this operation.

Effects: pure.

Build observe effect; specs is one map or an @ list of maps.

Return *[Set the observe field in the new record](table.put(in=None, at="observe", value=`specs`))*.

## unobserve

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| specs | any | required | Effect specifications; use a record or an @ row-record list |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Build unobserve effect; specs is a map or @ list with id |

Use `specs` as the inputs for this operation.

Effects: pure.

Build unobserve effect; specs is a map or @ list with id.

Return *[Set the unobserve field in the new record](table.put(in=None, at="unobserve", value=`specs`))*.

## focus

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| sel | any | required | CSS selector of the target browser element |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Build a focus effect for the selected element; the browser bridge applies it after the handler returns |

Use `sel` as the inputs for this operation.

Effects: pure.

Build a focus effect for the selected element; the browser bridge applies it after the handler returns.

Build the focus effect record. The browser bridge consumes this record; this function itself does not change the page.

Return *[Set the focus field in the new record](table.put(in=None, at="focus", value=`sel`))*.

## navigate

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| url | text | required | Target URL; HTTP helpers accept http or https |
| replace | any | False | Whether navigation replaces the current history entry |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Build the navigate effect record |

Use `url`, `replace`=False as the inputs for this operation.

Effects: pure.

Build the navigate effect record. The browser bridge consumes this record; this function itself does not change the page.

`spec` =

| url | replace |
|-----|---------|
| `url` | `replace` |

Return *[Set the navigate field in the new record](table.put(in=None, at="navigate", value=`spec`))*.

## store_set

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| key | text | required | Field name or key required by this operation |
| value | any | required | Value to transform or store, without mutating the input |
| scope | any | "memory" | Storage scope: memory, local or session |
| then | any | "" | Marqdo callback name for the bridge result; empty text omits callback |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | In-memory (or local/session) key write for SPA client state |

Use `key`, `value`, `scope`="memory", `then`="" as the inputs for this operation.

Effects: pure.

In-memory (or local/session) key write for SPA client state. Prefer `scope=memory` for tab-local app state.
Optional `then` is called after the write with `{ok, op, key, scope}`.

`spec` =

| op | key | value | scope |
| ---- | ----- | ------- | ------- |
| set | `key` | `value` | `scope` |

1. `then` != ""
    We retain **`spec` = [Set the then field in the new record](table.put(in=`spec`, at="then", value=`then`))**.
Return *[Set the storage field in the new record](table.put(in=None, at="storage", value=`spec`))*.

## store_get

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| key | text | required | Field name or key required by this operation |
| then | any | required | Marqdo callback name for the bridge result; empty text omits callback |
| scope | any | "memory" | Storage scope: memory, local or session |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Read a key then call `then` with `{ok, value, found, key, scope}` |

Use `key`, `then`, `scope`="memory" as the inputs for this operation.

Effects: pure.

Read a key then call `then` with `{ok, value, found, key, scope}`.

`spec` =

| op | key | then | scope |
| ---- | ----- | ------ | ------- |
| get | `key` | `then` | `scope` |

Return *[Set the storage field in the new record](table.put(in=None, at="storage", value=`spec`))*.

## spa_goto

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| url | text | required | Target URL; HTTP helpers accept http or https |
| replace | any | False | Whether navigation replaces the current history entry |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | SPA navigation: `history.pushState` / `replaceState` via bridge `navigate` effect |

Use `url`, `replace`=False as the inputs for this operation.

Effects: pure.

SPA navigation: `history.pushState` / `replaceState` via bridge `navigate` effect.

`spec` =

| url | replace |
|-----|---------|
| `url` | `replace` |

Return *[Set the navigate field in the new record](table.put(in=None, at="navigate", value=`spec`))*.

## storage

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| spec | any | required | Storage operation record: op, key, scope and optional value/then |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Wrap one storage specification as a browser effect; the bridge handles op, key, value, scope and then |

Use `spec` as the inputs for this operation.

Effects: pure.

Wrap one storage specification as a browser effect; the bridge handles op, key, value, scope and then.

Build the storage effect record. The browser bridge consumes this record; this function itself does not change the page.

Return *[Set the storage field in the new record](table.put(in=None, at="storage", value=`spec`))*.

## interval

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| ms | any | required | Duration or delay in milliseconds |
| then | any | required | Marqdo callback name for the bridge result; empty text omits callback |
| id | any | "default" | Timer identifier, later passed to clear_interval |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Build the interval effect record |

Use `ms`, `then`, `id`="default" as the inputs for this operation.

Effects: pure.

Build the interval effect record. The browser bridge consumes this record; this function itself does not change the page.

`spec` =

| ms | then | id |
|----|------|----|
| `ms` | `then` | `id` |

Return *[Set the interval field in the new record](table.put(in=None, at="interval", value=`spec`))*.

## clear_interval

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | any | "default" | Identifier returned by the corresponding create/spawn operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Build a cancellation effect using the same identifier previously supplied to interval |

Use `id`="default" as the inputs for this operation.

Effects: pure.

Build a cancellation effect using the same identifier previously supplied to interval.

Build the clear_interval effect record. The browser bridge consumes this record; this function itself does not change the page.

We retain **`spec` = [Set the id field in the new record](table.put(in=None, at="id", value=`id`))**.
Return *[Set the clear_interval field in the new record](table.put(in=None, at="clear_interval", value=`spec`))*.

## fetch

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| url | text | required | Target URL; HTTP helpers accept http or https |
| then | any | required | Marqdo callback name for the bridge result; empty text omits callback |
| method | text | "GET" | HTTP method, for example GET or POST |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Build the fetch effect record |

Use `url`, `then`, `method`="GET" as the inputs for this operation.

Effects: pure.

Build the fetch effect record. The browser bridge consumes this record; this function itself does not change the page.

`spec` =

| url | then | method |
|-----|------|--------|
| `url` | `then` | `method` |

Return *[Set the fetch field in the new record](table.put(in=None, at="fetch", value=`spec`))*.

## fetch_all

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| requests | any | required | Ordered list of browser request specification records |
| then | any | required | Marqdo callback name for the bridge result; empty text omits callback |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Build the fetch_all effect record |

Use `requests`, `then` as the inputs for this operation.

Effects: pure.

Build the fetch_all effect record. The browser bridge consumes this record; this function itself does not change the page.

`spec` =

| requests | then |
|----------|------|
| `requests` | `then` |

Return *[Set the fetch_all field in the new record](table.put(in=None, at="fetch_all", value=`spec`))*.

## after

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| ms | any | required | Duration or delay in milliseconds |
| then | any | required | Marqdo callback name for the bridge result; empty text omits callback |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Build the after effect record |

Use `ms`, `then` as the inputs for this operation.

Effects: pure.

Build the after effect record. The browser bridge consumes this record; this function itself does not change the page.

`spec` =

| ms | then |
|----|------|
| `ms` | `then` |

Return *[Set the after field in the new record](table.put(in=None, at="after", value=`spec`))*.

## ws

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| spec | any | required | One bridge effect specification record |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Wrap a WebSocket operation specification for the browser bridge; this function opens no native socket |

Use `spec` as the inputs for this operation.

Effects: pure.

Wrap a WebSocket operation specification for the browser bridge; this function opens no native socket.

Build the ws effect record. The browser bridge consumes this record; this function itself does not change the page.

Return *[Set the ws field in the new record](table.put(in=None, at="ws", value=`spec`))*.

## clipboard

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| text | text | required | Text supplied by the caller |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Build a clipboard effect from the text; browser permissions and user-gesture requirements apply when consumed |

Use `text` as the inputs for this operation.

Effects: pure.

Build a clipboard effect from the text; browser permissions and user-gesture requirements apply when consumed.

Build the clipboard effect record. The browser bridge consumes this record; this function itself does not change the page.

We retain **`spec` = [Set the text field in the new record](table.put(in=None, at="text", value=`text`))**.
Return *[Set the clipboard field in the new record](table.put(in=None, at="clipboard", value=`spec`))*.

## download

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| body | any | required | Request body, or replacement text for document operations |
| filename | any | "download.txt" | Suggested download filename |
| mime | any | "text/plain;charset=utf-8" | Content MIME type |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Build the download effect record |

Use `body`, `filename`="download.txt", `mime`="text/plain;charset=utf-8" as the inputs for this operation.

Effects: pure.

Build the download effect record. The browser bridge consumes this record; this function itself does not change the page.

`spec` =

| body | filename | mime |
|------|----------|------|
| `body` | `filename` | `mime` |

Return *[Set the download field in the new record](table.put(in=None, at="download", value=`spec`))*.

## render_list

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| sel | text | required | CSS selector of the existing list container |
| items | list | required | Literal text or records with tag, text or children, class and attrs |
| tag | text | "li" | Element name consumed by the official bridge, normally li |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Pure render_list instruction for the official browser bridge |

Use `sel`, `items` and `tag`="li" to replace one dynamic list while preserving its parent layout. Each record can keep plain text and a data-id attribute for delegated events. The official bridge already supports this effect and escapes ordinary text, so a title containing quotes or angle brackets need not become an HTML string. Constructing this instruction does not update a DOM or prove that the selector exists. Only pass application-owned element names and attribute keys; the underlying bridge also accepts an explicit html escape hatch for specialized integrations. Initial page structure still belongs in the Document's component tree, and incoming business values should remain text records.

Records also accept tag and children, a list of the same text/control records. A li can retain a span label beside a button with data-id, so delegated deletion needs no author HTML. Do not combine children with text or html. Nesting is limited to 32 levels and 10000 records; element/attribute names are checked and event attributes or active URL schemes are rejected. Keep business values in text; the existing html field remains an explicit compatibility escape hatch.

For a row with a delete action, pass children as structured data. This pure call returns the instruction without loading a server plugin or performing browser IO; the real example uses the same shape in [browser-app](../examples/browser-app/static/app.mq.md). Arbitrary attributes, element names and legacy html remain author-controlled capabilities, so application data belongs only in text or scalar values. The renderer's structural checks do not turn explicit raw HTML into untrusted-content sanitization.

```sh
marqdo call lib/browser.mq.md render_list 'sel="#items"' 'items=[{"tag":"li","children":[{"tag":"span","text":"A <tag> & literal"},{"tag":"button","text":"Delete","attrs":{"type":"button","data-id":"1"}}]}]' --effects none --json
```

`spec` =

| tag | items |
|---|---|
| `tag` | `items` |

We **targets = [Select the existing list container](table.put(in=None, at=`sel`, value=`spec`))**.
Return *[Describe the native bridge list update](table.put(in=None, at="render_list", value=`targets`))*.
