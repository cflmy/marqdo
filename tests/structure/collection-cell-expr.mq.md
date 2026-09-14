---
title: table cell expressions
description: Data cells evaluate like call-arg values (`var`, paths, hyphen prose).
---

# main

**secret = "sk-live"**
**name = "gpt-4o-mini"**

`cfg` =

| api_key | model | base |
|---------|-------|------|
| `secret` | `name` | https://api.openai.com/v1 |

> print text=[api_key](`cfg`)
> print text=[model](`cfg`)
> print text=[base](`cfg`)

`paths` =

| p |
|---|
| /chat/completions |

> print text=[1](`paths`)

`rec` =

| @ | role | content |
|---|------|---------|
| 1 | user | `secret` |

> print text=[content]([1](`rec`))

`ratios` =

| r |
|---|
| "1/5" |
| "16/9" |

> print text=[1](`ratios`)
> print text=[2](`ratios`)
