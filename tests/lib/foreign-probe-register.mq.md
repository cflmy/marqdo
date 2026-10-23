---
markup: occlusion
title: foreign probe and register
import:
  foreign: lib/foreign.mq.md
---

# main

**探测 = > foreign.probe lang="node"**
> print text=[ok](access(`探测`, "ok"))
> print text=[lang](access(`探测`, "lang"))

`模板` =

| 步骤 |
|------|
| node |
| "-e" |
| {inline} |

**> foreign.register lang="tiny" template=`模板`**
**运行 = > foreign.invoke lang="tiny" source="console.log('tiny')"**
**运行输出 = > trim [stdout](access(`运行`, "stdout"))**
> print text=`运行输出`
**> foreign.unset lang="tiny"**
