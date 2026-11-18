---
markup: occlusion
title: Bytecode backend
description: `--backend bytecode` shares semantics with tree-walk
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

Milestone: bytecode prototype.

The same `.mq.md` runs on tree-walk or the bytecode VM:

  marqdo run FILE --backend tree

  marqdo run FILE --backend bytecode

Output matches on both backends:

We now **[Display this step's result](> print text="Bytecode backend: same program, alternate engine.")**.
