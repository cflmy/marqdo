---
title: inherit parent method
import json:lib/json.mq.md
---

# Greeter

## hello
    + `who`

*`msg` = "Hello, `who`!" *
**`msg`**

# Loud = > Greeter

# main

*`l` = > Loud *
*`ty` = > json.get value=`l` key=_type *
> print text=`ty`
*`m` = > `l`.hello who=world *
> print text=`m`
