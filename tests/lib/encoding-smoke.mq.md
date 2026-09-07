---
title: lib/encoding smoke
import enc:lib/encoding.mq.md
---

# main

*b = > enc.base64_encode text="hello"*
> print text=`b`

*back = > enc.base64_decode text=`b`*
> print text=`back`

*h = > enc.hex_encode text="hi"*
> print text=`h`

*hb = > enc.hex_decode text=`h`*
> print text=`hb`
