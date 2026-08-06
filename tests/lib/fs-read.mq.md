---
title: fs read/exists
> lib/fs.mq.md
---

# main

*`ok` = > exists path=hello.txt *

1. `ok`
  > print text=exists
2. *
  > print text=missing

*`t` = > read_text path=hello.txt *

> print text=`t`
