---
title: ext/data/form
description: Form Resource — fields, validate, submit (ADR 0007).
import plugin:lib/plugin.mq.md
import sys:lib/sys.mq.md
import table:lib/table.mq.md
---

## _load

**p = > plugin.native_path name="web"**
1. `p`
  > plugin.load path=`p`
2. *
  > print text=ext/data: native web plugin not found
  > sys.exit code=1
****

# form
    + `table`=None
    + `action`=insert
    + `id`=None

Field table + rules table; submit writes through `# db`.

> _load
*> web_form_new table=`table` action=`action` id=`id`*

## fields
    + `fields`

*> web_form_fields form=`self` fields=`fields`*

## rules
    + `rules`

*> web_form_rules form=`self` rules=`rules`*

## labels
    + `submit`="Submit"
    + `cancel`="cancel"
    + `cancel_href`=""

Localized submit / cancel copy for rendered forms.

*> web_form_labels form=`self` submit=`submit` cancel=`cancel` cancel_href=`cancel_href`*

## validate
    + `rules`=None
    + `data`

*> web_form_validate form=`self` rules=`rules` data=`data`*

## render
    + `id`=form
    + `data`=None
    + `errors`=None

*> web_form_render form=`self` id=`id` data=`data` errors=`errors`*

## submit
    + `data`
    + `db`

**url = [url](db)**
*> web_form_submit form=`self` data=`data` url=`url`*

