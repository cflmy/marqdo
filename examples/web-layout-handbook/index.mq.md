---
markup: occlusion
type: web
view: main
title: Marqdo layout handbook
route: /
import:
  home: ./components/index.mq.md
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | The value explicitly returned below; conditional returns follow the selected branch |

The page selects the layout root; follow components/index.mq.md for decomposition.
This main returns rows rather than HTML; view metadata opts into startup assembly.

After these steps, return *home.nodes()*.
