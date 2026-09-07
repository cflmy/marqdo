---
title: fs copy/move/temp
import fs:lib/fs.mq.md
---

# main

> fs.write_text path="m2-src.txt" text="m2-body"

> fs.copy_file src="m2-src.txt" dest="m2-copy.txt"

*t = > fs.read_text path="m2-copy.txt"*
> print text=`t`

> fs.move src="m2-copy.txt" dest="m2-moved.txt"

*ok = > fs.exists path="m2-moved.txt"*
1. `ok`
  > print text=moved
2. *
  > print text=missing

*tmp = > fs.make_temp prefix="m2"*
> fs.write_text path=`tmp` text=temp-ok
*tt = > fs.read_text path=`tmp`*
> print text=`tt`

> fs.remove path="m2-src.txt"
> fs.remove path="m2-moved.txt"
> fs.remove path=`tmp`
