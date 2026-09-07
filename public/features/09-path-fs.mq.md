---
title: Mid stdlib — path and fs
description: lib/path + fs copy/move/temp (Mid M2)
import path:lib/path.mq.md
import fs:lib/fs.mq.md
---

# Path algebra and file mid-layer

Join and normalize without plugins; copy/move stay sandboxed under the program directory.

# main

*`p` = > path.join a="out" b="demo.txt"*
*`n` = > path.normalize path="out/./x/../demo.txt"*
> fs.write_text path=`p` text="path-m2"
> fs.copy_file src=`p` dest="out/demo-copy.txt"
*`body` = > fs.read_text path="out/demo-copy.txt"*
> print text=`p`
> print text=`n`
> print text=`body`
1. `body` == "path-m2"
  > print text=mid-m2-ok
2. *
  > print text=mid-m2-fail
> fs.remove path=`p`
> fs.remove path="out/demo-copy.txt"
