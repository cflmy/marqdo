---
title: object handle smoke
> lib/json.mq.md
---

# counter
    - n

*`t` = > parse text={"p":"{\"n\":","s":"}"} *
*`qn` = > str `n` *
*`p` = > get value=`t` key=p *
*`s` = > get value=`t` key=s *
*`raw` = `p` + `qn` + `s` *
**> parse text=`raw`**

## bump

*`n` = > get value=`self` key=n *
*`n2` = `n` + 1*
*`t` = > parse text={"p":"{\"n\":","s":"}"} *
*`qn` = > str `n2` *
*`p` = > get value=`t` key=p *
*`s` = > get value=`t` key=s *
*`raw` = `p` + `qn` + `s` *
*`m` = > parse text=`raw` *
**`m`**

# main

*`c` = > counter n=3 *
*`ty` = > get value=`c` key=_type *
> print text=`ty`
*`n` = > get value=`c` key=n *
> print text=`n`
*`c2` = > `c`.bump *
*`n2` = > get value=`c2` key=n *
> print text=`n2`
