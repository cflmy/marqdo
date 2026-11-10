---
markup: occlusion
title: ext/data/form
description: Form Resource — fields, validate, submit (ADR 0007).
import:
  plugin: lib/plugin.mq.md
  sys: lib/sys.mq.md
  table: lib/table.mq.md
---

This leaf projects form schema and submission operations through the native web runtime. Read [the data entry](index.mq.md) before connecting a form to persistent storage. Schema validation and route authorization are different responsibilities: an accepted field value does not authenticate its submitter. Keep schema rows, validation failures and persistence results visible when changing this resource.

## _load

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | No business value on completion; a missing native plugin reports the cause and exits this run. |

Resolve and load the native web plugin before resource operations; missing binaries stop this run.

We retain **`p` = [Locate the installed native plugin](plugin.native_path(name="web"))**.
1. `p`
  We **[Load the native plugin required by this operation](plugin.load(path=`p`))**.
2. *
  We **[Print the prepared message](print(text="ext/data: native web plugin not found"))**.
  We **[End this run with the selected exit status](sys.exit(code=1))**.
Return *None*.


## reference

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | text | required | Exact id explicitly registered with mount |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Pure form_ref Resource for component.resource |

Use `id` to reference the same mounted form that handles submissions. This constructor does not create a form, load a plugin or register a route. Serving resolves the id after preparing the request's actual CSRF token and return path. An unknown id is a real request error; it cannot become an empty successful form.

The form's field/rule resource remains the single source for rendering and validation. Error responses retain the current CSRF token so the user can correct a field and resubmit. Authorization belongs to mount's optional role gate and the server's authentication policy, not the visibility of this node.

`resource` =

| _type | id |
|---|---|
| form_ref | `id` |

Return *resource* for explicit placement in the existing View schema.

## mount

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| app | any | required | Prepared web application with its actual database |
| id | text | required | Explicit system-form identifier used by reference |
| form | any | required | Form resource containing fields, rules and submission action |
| roles | any | None | None keeps public form behavior; otherwise gate the actual system form route |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Updated app with this form and optional server role gate |

Use `app`, `id`, `form` and `roles`=None before listening. Keep the returned app. The native system form route uses the same resource to validate and submit; component.resource only places its reference in the readable page layout. No user route is implicitly registered. System endpoints remain visible through web.inspect.

For a private form, provide roles explicitly so the actual /_form/id GET and POST path is gated as well as any containing page. Merely hiding a form or gating its presentation URL is insufficient. Configure authentication separately. A public creation example can retain roles=None. Existing sessions must submit the correct CSRF token; field-validation retries keep that token in the response.

We **[Load the existing form implementation](_load())** and **mounted = [Bind this form to its explicit system id](web_app_mount_form(app=`app`, id=`id`, form=`form`))**.
1. `roles` != None
  We **mounted = [Protect the actual system form endpoint](web_app_gate(app=`mounted`, path="/_form/" + id, roles=`roles`, permissions="", match="exact", on_deny="redirect", exclude=None))**.
Return *mounted* after the explicit form resource and policy are attached.

# form

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| table | any | None | Database table name or form submission table |
| action | any | "insert" | Form submission action such as insert or update |
| id | any | None | Identifier produced or selected by the corresponding resource operation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Field table + rules table; submit writes through `# db`. |

Use `table`=None, `action`="insert", `id`=None as the inputs for this operation.

Field table + rules table; submit writes through `# db`.

We **[Load the native plugin required by this operation](_load())**.
Return *[Field table + rules table; submit writes through # db.](web_form_new(table=`table`, action=`action`, id=`id`))*.

## fields

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| fields | any | required | Readable field definitions with the columns described by this resource |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Attach the readable field definitions to this form and return the updated resource. |

Use `fields` as the inputs for this operation.

Attach the readable field definitions to this form and return the updated resource.

Return *[Attach the readable field definitions to this form and return the updated resource.](web_form_fields(form=`self`, fields=`fields`))*.

## rules

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| rules | any | required | Readable validation-rule records |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Attach validation rules to this form without submitting user data. |

Use `rules` as the inputs for this operation.

Attach validation rules to this form without submitting user data.

Return *[Attach validation rules to this form without submitting user data.](web_form_rules(form=`self`, rules=`rules`))*.

## labels

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| submit | any | "Submit" | Displayed submit-button label |
| cancel | any | "cancel" | Displayed cancel-button label |
| cancel_href | any | "" | Navigation target of the cancel action |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Localized submit / cancel copy for rendered forms. |

Use `submit`="Submit", `cancel`="cancel", `cancel_href`="" as the inputs for this operation.

Localized submit / cancel copy for rendered forms.

Return *[Localized submit / cancel copy for rendered forms.](web_form_labels(form=`self`, submit=`submit`, cancel=`cancel`, cancel_href=`cancel_href`))*.

## validate

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| rules | any | None | Readable validation-rule records |
| data | any | required | Structured input supplied to this adapter or resource |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Validate supplied field values and return the native validation result; this does not submit to the database. |

Use `rules`=None, `data` as the inputs for this operation.

Validate supplied field values and return the native validation result; this does not submit to the database.

Return *[Validate supplied field values and return the native validation result; this does not submit to the database.](web_form_validate(form=`self`, rules=`rules`, data=`data`))*.

## render

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| id | any | "form" | Identifier produced or selected by the corresponding resource operation |
| data | any | None | Structured input supplied to this adapter or resource |
| errors | any | None | Field/error record returned by validation |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Render this form using its data and field errors; rendering is separate from database submission. |

Use `id`="form", `data`=None, `errors`=None as the inputs for this operation.

Render this form using its data and field errors; rendering is separate from database submission.

Return *[Render this form using its data and field errors; rendering is separate from database submission.](web_form_render(form=`self`, id=`id`, data=`data`, errors=`errors`))*.

## submit

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| data | any | required | Structured input supplied to this adapter or resource |
| db | any | required | Database resource created by the public database constructor |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | Submit validated form data through the selected database resource and configured action. |

Use `data`, `db` as the inputs for this operation.

Submit validated form data through the selected database resource and configured action.

We retain **`url` = [Read url field](access(`db`, "url"))**.
Return *[Submit validated form data through the selected database resource and configured action.](web_form_submit(form=`self`, data=`data`, url=`url`))*.
