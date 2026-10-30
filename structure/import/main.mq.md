---
markup: occlusion
title: Cross-file import
description: Frontmatter nested import: map
import:
  utils: utils.mq.md
---

# main

In the YAML header, nested `import:` with `utils: utils.mq.md` imports a sibling module and binds the library name `utils`.

Bind a return value, print it, then call greet:

**`y` = [utils.add_one] n=41**

> print text=`y`

> utils.greet who=Marqdo
