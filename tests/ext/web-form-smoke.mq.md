---
title: web form smoke
description: Offline form fields / validate / submit (no listen).
import web:ext/web/web.mq.md
import articles:web-fixtures/db/articles.mq.md
import json:lib/json.mq.md
---

# main

*`store` = > web.db url="sqlite:web-fixtures/data/form-smoke.db" *
*`schema` = > articles.schema *
> `store`.init name=articles fields=`schema`

`fields` =

| 字段 | 标签 | 类型 | 必填 | 默认 |
|------|------|------|------|------|
| title | Title | text | true | |
| body | Body | textarea | false | |

`rules` =

| 字段 | 规则 | 消息 |
|------|------|------|
| title | required | Title is required |
| title | max:120 | Title is too long |

*`f` = > web.form table=articles action=insert *
*`f` = > `f`.fields fields=`fields` *
*`f` = > `f`.rules rules=`rules` *

`bad` =

| title | body |
|-------|------|
| | hello |

*`v` = > `f`.validate data=`bad` *
*`ok` = > json.get value=`v` key=ok *
1. `ok`
  > print text=validate-should-fail
2. *
  > print text=validate-ok

`good` =

| title | body |
|-------|------|
| From form | Body via submit |

*`sub` = > `f`.submit data=`good` db=`store` *
*`sok` = > json.get value=`sub` key=ok *
1. `sok`
  > print text=submit-ok
2. *
  > print text=submit-fail

*`rows` = > `store`.select table=articles limit=10 *
*`n` = > len value=`rows` *
1. `n` >= 1
  > print text=db-ok
2. *
  > print text=db-fail

*`html` = > `f`.render id=article *
1. `html`
  > print text=render-ok
2. *
  > print text=render-fail
