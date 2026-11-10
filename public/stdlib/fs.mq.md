---
markup: occlusion
title: lib/fs — files
description: Read and write text under the sandbox
import:
  fs: lib/fs.mq.md
---

# main

| 返回 | 类型 | 说明 |
|---|---|---|
| result | any | None; execute this page's documented steps in order and observe their output or stated effects |

Filesystem helpers from lib/fs. Bare path words must not contain `/` (division). Prefer a same-folder name or a variable.

We first **ok = [Compute ok](fs.exists(path="demo.txt"))** for the next step.

1. `ok`
  **打印 内容="demo-present"**
2. *
  **打印 内容="demo-missing"**
