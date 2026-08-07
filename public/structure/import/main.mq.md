---
title: Cross-file import
description: Frontmatter `> file.mq.md` merges modules
> utils.mq.md
---

# main

In the YAML header, `> utils.mq.md` imports a sibling module.

Bind a return value, print it, then call greet:

*`y` = > utils.add_one n=41*

> print text=`y`

> utils.greet who=Marqdo
