---
markup: occlusion
title: SPA client with native records and structured browser effects
import:
  browser: lib/browser.mq.md
  table: lib/table.mq.md
---

The Document supplies native controls; this client supplies transitions between list, detail and edit. BrowserSession retains the entry state between callbacks. Stable ids keep selection separate from user text, and all dynamic list content is sent as records to the existing official bridge.

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Boot bindings, initial list, memory storage and panel effects |

This client keeps route, selection and records in the browser session's native Marqdo entry state. Titles are text; stable ids select records even when a title contains punctuation or is edited. The initial list is a row table rather than a newline string. Boot binds the existing controls, seeds the memory store and paints the first view immediately. Storage lasts for this client session; it is not a server database or durable reload persistence. Page geometry and labels live in separate Document components. Event callbacks only calculate state changes and return instructions for the official bridge.

We **route = "/"** and **cur_id = ""** and **next_id = 2** to start with no selected record.

`items` =

| @ | id | title |
|---|---|---|
| 1 | "1" | Alpha |
| 2 | "2" | Beta |

We **bindings = [Read the explicit event wiring](_bindings())** and **wired = [Describe those control bindings](table.put(in=None, at="wire", value=`bindings`))**.
We **painted = [Paint the initial list without changing history](spa_paint(records=`items`, path=`route`, selected_id=`cur_id`, update_url=False))**.
After boot, return *[Keep bindings and initial effects together](browser.merge(a=`wired`, b=`painted`))*.

## _bindings

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Explicit delegated event rows for existing native controls |

These rows connect the Document's stable ids to Marqdo callbacks. List clicks are delegated to li[data-id], so replacing list entries does not discard the listener. Save reads the existing input value, and browser back/forward supplies path to on_pop. The bridge owns event dispatch; this table does not contain author JavaScript or page HTML. Every referenced callback below has an actual parameter and return contract.

`bindings` =

| @ | 选择器 | 事件 | 调用 | 委托 | 值选择器 |
|---|---|---|---|---|---|
| 1 | "#nav-list" | click | go_list | | |
| 2 | "#nav-new" | click | go_new | | |
| 3 | "#items" | click | open_item | "li[data-id]" | |
| 4 | "#btn-edit" | click | go_edit | | |
| 5 | "#btn-back" | click | go_list | | |
| 6 | "#btn-save" | click | save_item | | "#edit-title" |
| 7 | "#btn-cancel" | click | go_list | | |
| 8 | window | popstate | on_pop | | |

Return *bindings*.

## on_pop

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| path | text | "" | Location path supplied by the browser's popstate event |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Panel, list and storage effects without pushing a second history entry |

Use `path`="" from the event payload. Route state uses a different name so assigning the event value changes shared state instead of shadowing a path parameter. An empty payload leaves the current route unchanged. Painting a history event deliberately avoids spa_goto: pressing Back must not create another forward history record.

1. `path` != ""
  We **route = [Retain the location supplied by the browser](path)**.
Return *[Repaint the restored route without another navigation](spa_paint(records=`items`, path=`route`, selected_id=`cur_id`, update_url=False))*.

## go_list

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | List panel effects and one navigation instruction |

The Items and Back controls share this transition. Clear the selected id so the list does not retain an editing target, switch the shared route to the list path, then paint through the common function. Stored item data remains unchanged. The browser bridge performs the returned navigation and DOM instructions after the state update.

We **route = "/"** and **cur_id = ""**.
Return *[Show the current list](spa_paint(records=`items`, path=`route`, selected_id=`cur_id`))*.

## go_new

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Empty editor effects and one navigation instruction |

Creating a record opens the editor without selecting an existing id. Save allocates an id only after nonempty text arrives, so merely opening or cancelling this panel does not insert a blank record. The common paint function puts an empty value in the native input and hides the other panels.

We **route = "/edit"** and **cur_id = ""**.
Return *[Open the empty editor](spa_paint(records=`items`, path=`route`, selected_id=`cur_id`))*.

## go_edit

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Editor effects for the existing selection |

Editing keeps the selected id and changes only the current panel. The paint function looks up the current title from native rows, so a renamed record is still found by its stable id. Navigation and rendering remain instructions for the bridge; this callback neither writes a file nor calls a server.

We **route = "/edit"**.
Return *[Show the selected record in the editor](spa_paint(records=`items`, path=`route`, selected_id=`cur_id`))*.

## open_item

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| data_id | text | "" | Stable data-id from the delegated list item |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Detail effects, or None when no record id was clicked |

Use `data_id`="" from the delegated event. A click outside a real list entry must not invent a selection. Keep the id separately from its displayed title; quoted text, angle brackets and later renames therefore do not alter the event identity. Actual text is selected when the detail panel is painted.

1. `data_id` != ""
  We **cur_id = [Retain the clicked record identity](data_id)** and **route = "/item"**.
  Return *[Show the selected record](spa_paint(records=`items`, path=`route`, selected_id=`cur_id`))*.
If no record was clicked, return *None*.

## save_item

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| value | text | "" | Plain title from the native editor input |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Saved detail effects, or a title-required text effect |

Use `value`="" from the Save binding. Empty text returns a validation message before changing data. For an existing id, replace that row's title while retaining all ids and row order. For a new record, allocate the next native integer id and convert it to event text. The resulting records are kept in the session and mirrored to the memory store by painting. No HTML is generated from a title and no external interpreter performs the array update. Cancel bypasses this function entirely.

1. `value` == ""
  Return *[Explain the required title](browser.set_text(sel="#log", text="title required"))*.
1. `cur_id` != ""
  We **updated = [Start the replacement record list](table.empty_list())**.
  - [Retain each record or replace only its title](for items as item)
    1. access(`item`, "id") == `cur_id`
      We **item = [Update the title without changing its id](table.put(in=`item`, at="title", value=`value`))**.
    We **updated = [Retain this row in its original order](table.append(list=`updated`, item=`item`))**.
  We **items = [Keep the completed native row list](updated)**.
2. *
  We **next_id = next_id + 1** and **cur_id = [Use the stable integer id as event text](str(next_id))**.

  `item` =

  | id | title |
  |---|---|
  | `cur_id` | `value` |

  We **items = [Append the new native record](table.append(list=`items`, item=`item`))**.
We **route = "/item"** after a successful save.
Return *[Display and mirror the saved record](spa_paint(records=`items`, path=`route`, selected_id=`cur_id`))*.

## _list_rows

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| records | list | required | Native rows with id and title |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | list | Bridge list items containing escaped text and stable data-id attributes |

Use `records` as the only list input. Each output item carries plain text plus an attribute record; the official bridge performs text escaping when it renders the list. The source never interpolates a title into an HTML tag. The delegated click id comes from the original record id, preserving selection independently of the displayed title.

We **rows = [Start the browser list descriptions](table.empty_list())**.
- [Describe every native record as a text item](for records as record)
  We **label = [Read its plain title](access(`record`, "title"))** and **id = [Read its stable identity](access(`record`, "id"))** and **attrs = [Retain the delegated event id](table.put(in=None, at="data-id", value=`id`))**.

  `row` =

  | text | attrs |
  |---|---|
  | `label` | `attrs` |

  We **rows = [Retain this structured list item](table.append(list=`rows`, item=`row`))**.
Return *rows*.

## spa_render

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Current panel, list, memory and navigation effects |

This named entry preserves the common rendering callback used by the demonstration. It delegates to the same paint function as direct transitions so panel selection, item text and memory storage stay consistent. No page string is compiled or evaluated here; the returned value is a normal bridge instruction record.

Return *[Paint the current native state](spa_paint(records=`items`, path=`route`, selected_id=`cur_id`))*.

## spa_paint

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| records | list | required | Native item rows to display and mirror |
| path | text | required | Current client route |
| selected_id | text | required | Stable selected id, or empty for a new item |
| update_url | bool | True | False for history events and initial boot; True for user navigation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | One complete structured effect map for the current native state |

Use `records`, `path`, `selected_id` and `update_url`=True for ordinary user navigation, and False when repainting an existing history entry. The selected row is found by id; all content remains plain text. Three panels receive explicit hidden flags, while title, detail and log text are described together. Browser merge replaces whole top-level effect keys, so combining those text targets first prevents later messages from discarding earlier text updates. List rows go through render_list, and the editor value changes only while that panel is active. Storage uses a record list in the memory scope. This function describes effects and performs no direct DOM, network or file access.

We **selected = [Find the native record by stable identity](table.find(list=`records`, key="id", value=`selected_id`))** and **body = ""** and **title = "Items"**.
1. `selected` != None
  We **body = [Read the current selected title](access(`selected`, "title"))**.
1. `path` == "/item"
  We **title = "Detail"**.
1. `path` == "/edit"
  We **title = "Edit"**.
We **list_flag = [Hide the list outside its route](table.put(in=None, at="hidden", value=(path != "/")))** and **detail_flag = [Show only the detail route](table.put(in=None, at="hidden", value=(path != "/item")))** and **edit_flag = [Show only the editor route](table.put(in=None, at="hidden", value=(path != "/edit")))**.

`visibility` =

| #view-list | #view-detail | #view-edit |
|---|---|---|
| `list_flag` | `detail_flag` | `edit_flag` |

`texts` =

| #title | #detail-body | #log |
|---|---|---|
| `title` | `body` | `path` |

We **ret = [Describe all text updates in one effect](browser.wrap(key="set_text", value=`texts`))** and **flags = [Describe the three panel visibility updates](browser.wrap(key="set_attr", value=`visibility`))** and **ret = [Keep independent text and attribute effects](browser.merge(a=`ret`, b=`flags`))**.
We **rows = [Describe record text without HTML](_list_rows(records=`records`))** and **listing = [Replace the existing list using structured items](browser.render_list(sel="#items", items=`rows`))** and **ret = [Retain the list update](browser.merge(a=`ret`, b=`listing`))**.
1. `path` == "/edit"
  We **edit = [Populate the native editor input](browser.set_value(sel="#edit-title", value=`body`))** and **ret = [Retain the input value update](browser.merge(a=`ret`, b=`edit`))**.
1. `update_url`
  We **navigation = [Navigate only for a user transition](browser.spa_goto(url=`path`))** and **ret = [Retain this history instruction](browser.merge(a=`ret`, b=`navigation`))**.

`persist` =

| op | key | value | scope |
|---|---|---|---|
| set | spa-items | `records` | memory |

We **storage = [Mirror the native records in memory](browser.storage(spec=`persist`))**.
Return *[Keep the completed paint and memory instructions](browser.merge(a=`ret`, b=`storage`))*.
