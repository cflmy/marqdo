---
title: lib/fs — files
description: Read and write text under the sandbox
import fs:lib/fs.mq.md
---

# main

Filesystem helpers from lib/fs. Bare path words must not contain `/` (division). Prefer a same-folder name or a variable.

**`ok` = [fs.exists] path="demo.txt"**

1. `ok`
  **打印 内容="demo-present"**
2. *
  **打印 内容="demo-missing"**
