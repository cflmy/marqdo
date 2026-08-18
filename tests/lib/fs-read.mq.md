---
title: fs read/exists
import fs:lib/fs.mq.md
---

# main

*ok = > fs.exists path="hello.txt"*

1. `ok`
  > print text=exists
2. *
  > print text=missing

*t = > fs.read_text path="hello.txt"*

> print text=`t`
