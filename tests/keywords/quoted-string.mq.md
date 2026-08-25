---
title: quoted string literal
---

# main

*s = "a\nb"*
> print text=`s`

*s2 = "hex \x22 quote\x27 single"*
> print text=`s2`

*s3 = "\u{4e2d}\u{6587}"*
> print text=`s3`

*s4 = "keep \q verbatim"*
> print text=`s4`

*s5 = "\x41\x42\x43"*
> print text=`s5`