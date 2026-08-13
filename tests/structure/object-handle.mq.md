---
title: object handle smoke
import json:lib/json.mq.md
---

# counter
    + `n`

*`t` = > json.parse text={"p":"{\"n\":","s":"}"} *
*`qn` = > str `n` *
*`p` = > json.get value=`t` key=p *
*`s` = > json.get value=`t` key=s *
*`raw` = `p` + `qn` + `s` *
**> json.parse text=`raw`**

## bump

*`n` = > json.get value=`self` key=n *
*`n2` = `n` + 1*
*`t` = > json.parse text={"p":"{\"n\":","s":"}"} *
*`qn` = > str `n2` *
*`p` = > json.get value=`t` key=p *
*`s` = > json.get value=`t` key=s *
*`raw` = `p` + `qn` + `s` *
*`m` = > json.parse text=`raw` *
**`m`**

# main

*`c` = > counter n=3 *
*`ty` = > json.get value=`c` key=_type *
> print text=`ty`
*`n` = > json.get value=`c` key=n *
> print text=`n`
*`c2` = > `c`.bump *
*`n2` = > json.get value=`c2` key=n *
> print text=`n2`
