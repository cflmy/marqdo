---
markup: occlusion
title: Cross-file import
description: Frontmatter nested import: map
import:
  utils: utils.mq.md
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

In the YAML header, nested `import:` with `utils: utils.mq.md` imports a sibling module and binds the library name `utils`.

Bind a return value, print it, then call greet:

We first **y = [Compute y](utils.add_one(n=41))** for the next step.

We now **[Display this step's result](> print text=`y`)**.

We now **[Call utils.greet](> utils.greet who="Marqdo")**.
