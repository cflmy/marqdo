---
title: Bytecode backend
description: `--backend bytecode` shares semantics with tree-walk
---

# main

Milestone: bytecode prototype.

The same `.mq.md` runs on tree-walk or the bytecode VM:

  marqdo run FILE --backend tree

  marqdo run FILE --backend bytecode

Output matches on both backends:

> print text=Bytecode backend: same program, alternate engine.
