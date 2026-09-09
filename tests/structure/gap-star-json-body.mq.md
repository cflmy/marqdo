---
title: GAP-06/07 star + JSON array body keep following stmts
description: Nested args=`args` and text=["…"] must not truncate the ## body.
import json:lib/json.mq.md
---

# main

*a = > case_backtick*
*b = > case_json_array*
*c = > case_recover*
> print text=`a`
> print text=`b`
> print text=`c`

## case_backtick

`args` =

| x |
|---|
| ok |

*code = > len value=`args`*
*y = 1*
**y**

## case_json_array

*xs = > json.parse text=["scripts/legacy/web_search.py"]*
*n = > len value=`xs`*
**n**

## case_recover

故意少写闭合星号，后续语句仍须可执行：

*c = > json.parse text=[]`
*z = 7*
**z**
