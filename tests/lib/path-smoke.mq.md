---
title: lib/path smoke
import path:lib/path.mq.md
---

# main

*j = > path.join a="dir/sub" b="f.txt"*
> print text=`j`

*name = > path.file_name path=`j`*
> print text=`name`

*ext = > path.extension path=`j`*
> print text=`ext`

*par = > path.parent path=`j`*
> print text=`par`

*norm = > path.normalize path="a/./b/../c"*
> print text=`norm`

*abs = > path.is_absolute path="/tmp/x"*
> print text=`abs`

*parts = > path.split path="a/b/c"*
*pj = > join value=`parts` sep="|"*
> print text=`pj`
