---
title: fs read/exists
> lib/fs.mq.md
---

# main

*`ok` = > exists path=hello.txt *

+ `ok`
  > print text=exists
+ *
  > print text=missing

*`t` = > read_text path=hello.txt *

> print text=`t`
