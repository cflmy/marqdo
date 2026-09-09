---
title: Mid stdlib — fs make_dirs, walk, stat
description: Recursive dirs, walk, and file metadata (Mid2 M9)
import fs:lib/fs.mq.md
---

# Practical filesystem without plugins

`make_dirs` / `remove_tree` / `stat` / `walk` thicken `lib/fs` for scripts and tools.

# main

> fs.make_dirs path="m9-demo/sub"

> fs.write_text path="m9-demo/readme.txt" text="ok"

*st = > fs.stat path="m9-demo/readme.txt"*
> print text=`st`[^size]

*w = > fs.walk path="m9-demo"*
*n = > len `w`*
> print text=`n`

> fs.remove_tree path="m9-demo"
