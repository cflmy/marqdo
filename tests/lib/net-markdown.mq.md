---
title: lib/net markdown_parse smoke
import net:lib/net.mq.md
---

# main

*html = > net.markdown_parse text="# Title\n\nHello **world**."*
1. `html` != ""
  > print text=markdown-ok
2. *
  > print text=markdown-fail
