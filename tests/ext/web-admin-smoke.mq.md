---
title: web admin form-from-schema smoke
description: Schema-derived admin forms share validate/submit (offline).
> ext/web/web.mq.md
> web-fixtures/db/articles.mq.md as articles
> lib/json.mq.md
---

# main

*`store` = > web.db url="sqlite:web-fixtures/data/admin-smoke.db" *
*`schema` = > articles.schema *
> `store`.init name=articles fields=`schema`
*`url` = > json.get value=`store` key=url *

*`info` = > web_db_table_info url=`url` table=articles *
*`cols` = > json.get value=`info` key=columns *
*`n` = > len value=`cols` *
1. `n` >= 2
  > print text=schema-ok
2. *
  > print text=schema-fail

*`fnew` = > web_form_from_schema url=`url` table=articles action=insert *
*`fields` = > json.get value=`fnew` key=fields *
*`nf` = > len value=`fields` *
1. `nf` >= 1
  > print text=new-form-ok
2. *
  > print text=new-form-fail

`row` =

| title | body |
|-------|------|
| Admin create | via schema form |

*`sub` = > web_form_submit form=`fnew` data=`row` url=`url` *
*`sok` = > json.get value=`sub` key=ok *
1. `sok`
  > print text=insert-ok
2. *
  > print text=insert-fail

*`got` = > `store`.get table=articles id=1 *
*`title` = > json.get value=`got` key=title *
1. `title` == "Admin create"
  > print text=get-ok
2. *
  > print text=get-fail

*`fedit` = > web_form_from_schema url=`url` table=articles action=update id=1 *
`patch` =

| title | body |
|-------|------|
| Admin edited | body updated |

*`up` = > web_form_submit form=`fedit` data=`patch` url=`url` *
*`uok` = > json.get value=`up` key=ok *
1. `uok`
  > print text=update-ok
2. *
  > print text=update-fail

*`got2` = > `store`.get table=articles id=1 *
*`title2` = > json.get value=`got2` key=title *
1. `title2` == "Admin edited"
  > print text=edit-ok
2. *
  > print text=edit-fail

`empty` =

| title | body |
|-------|------|
| | |

*`bad` = > web_form_submit form=`fnew` data=`empty` url=`url` *
*`bok` = > json.get value=`bad` key=ok *
1. `bok`
  > print text=required-should-fail
2. *
  > print text=required-ok
