---
title: lib/text mid smoke
import text:lib/text.mq.md
---

# main

*c = > text.contains text="Hello" sub="ell"*
> print text=`c`

*u = > text.to_upper text="ab"*
> print text=`u`

*r = > text.replace text="a-a-a" old="-" new="_" count=2*
> print text=`r`

*p = > text.pad text="x" width=4 fill="." align="right"*
> print text=`p`

*rep = > text.repeat text="ab" n=3*
> print text=`rep`
