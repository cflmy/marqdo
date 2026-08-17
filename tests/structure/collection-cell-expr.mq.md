---
title: table cell expressions
description: Data cells evaluate like call-arg values (`var`, paths, hyphen prose).
---

# main

*secret = "sk-live"*
*name = "gpt-4o-mini"*

`cfg` =

| api_key | model | base |
|---------|-------|------|
| `secret` | `name` | https://api.openai.com/v1 |

> print text=`cfg`[^api_key]
> print text=`cfg`[^model]
> print text=`cfg`[^base]

`paths` =

| p |
|---|
| /chat/completions |

> print text=`paths`[^1]

`rec` =

| @ | role | content |
|---|------|---------|
| 1 | user | `secret` |

> print text=`rec`[^1][^content]
