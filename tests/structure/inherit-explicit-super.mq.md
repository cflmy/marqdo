---
title: inherit explicit parent constructor
import json:lib/json.mq.md
---

# Parent
    + `name`

*`out` = > json.parse text={"role":"parent"} *
*`out` = > json.set map=`out` key=name value=`name` *
**`out`**

# Child = > Parent
    + `name`

*`self` = > Parent name=`name` *
*`self` = > json.set map=`self` key=role value=child *
**`self`**

# main

*`c` = > Child name=Ada *
*`ty` = > json.get value=`c` key=_type *
*`role` = > json.get value=`c` key=role *
*`name` = > json.get value=`c` key=name *
> print text=`ty`
> print text=`role`
> print text=`name`
