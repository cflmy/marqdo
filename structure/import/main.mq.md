---
title: Cross-file import
description: Frontmatter import bind:path.mq.md
import utils:utils.mq.md
---

# main

In the YAML header, `import utils:utils.mq.md` imports a sibling module and binds the library name `utils`.

Bind a return value, print it, then call greet:

*`y` = > utils.add_one n=41*

> print text=`y`

> utils.greet who=Marqdo
