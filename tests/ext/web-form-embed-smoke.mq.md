---
title: web form embed smoke
description: Offline page.compose_form embeds form in main HTML (no listen).
> ext/web/web.mq.md
> lib/json.mq.md
---

# main

`fields` =

| 字段 | 标签 | 类型 | 必填 | 默认 |
|------|------|------|------|------|
| title | Title | text | true | |
| body | Body | textarea | false | |

*`f` = > web.form table=articles action=insert *
*`f` = > `f`.fields fields=`fields` *

*`page` = > web.page title="Compose" intro="<h1>Compose</h1>" *
*`page` = > `page`.compose_form id=article form=`f` *

*`fid` = > json.get value=`page` key=form_id *
1. `fid`
  > print text=form-id-ok
2. *
  > print text=form-id-fail

*`html` = > `page`.render *
1. `html`
  > print text=render-ok
2. *
  > print text=render-fail
