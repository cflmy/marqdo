---
markup: occlusion
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

> print text=[api_key](access(`cfg`, "api_key"))
> print text=[model](access(`cfg`, "model"))
> print text=[base](access(`cfg`, "base"))

`paths` =

| p |
|---|
| /chat/completions |

> print text=[1](access(`paths`, "1"))

`rec` =

| @ | role | content |
|---|------|---------|
| 1 | user | `secret` |

> print text=[content](access([1](access(`rec`, "1")), "content"))

`ratios` =

| r |
|---|
| "1/5" |
| "16/9" |

> print text=[1](access(`ratios`, "1"))
> print text=[2](access(`ratios`, "2"))
