---
title: fs mid2 M9 smoke
import fs:lib/fs.mq.md
---

# main

> fs.make_dirs path="m9-tree/nested"

> fs.write_text path="m9-tree/a.txt" text="A"

> fs.write_text path="m9-tree/nested/b.txt" text="B"

*st = > fs.stat path="m9-tree/a.txt"*
> print text=`st`[^size]
> print text=`st`[^is_file]
> print text=`st`[^is_dir]

*w = > fs.walk path="m9-tree"*
*n = > len `w`*
> print text=`n`
*p0 = > at value=`w` index=0*
*p1 = > at value=`w` index=1*
*p2 = > at value=`w` index=2*
> print text=`p0`
> print text=`p1`
> print text=`p2`

> fs.remove_tree path="m9-tree"

*gone = > fs.exists path="m9-tree"*
> print text=`gone`
