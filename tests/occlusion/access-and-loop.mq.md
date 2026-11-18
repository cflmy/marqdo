---
markup: occlusion
title: access + occlusion loop
import:
  json: lib/json.mq.md
---

# main

`表` = > json.parse text={"键":"值"}
`列` = > json.parse text=["一","二"]

**a = [取表项](access(`表`, "键"))**
**b = [取列表第二项](access(`列`, 2))**
> print text="a=" + `a`
> print text="b=" + `b`
- [篮子里的每个果子](遍历 `列` 为 `果`)
  > print text="果子=" + `果`
