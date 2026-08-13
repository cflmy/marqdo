---
title: lib/fs — files
description: Read and write text under the sandbox
import fs:lib/fs.mq.md
---

# main

Import lib/fs.mq.md. Functions: read_text, write_text, append_text, exists, list_dir, make_dir, remove (path / text params).

Bare path words must not contain a slash character (parsed as division). Prefer a same-folder name or a variable.

*`ok` = > fs.exists path=demo.txt *

1. `ok`
  > print text=demo-present
2. *
  > print text=demo-missing
