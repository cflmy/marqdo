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
