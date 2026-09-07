---
title: lib/re smoke
import re:lib/re.mq.md
---

# main

*ok = > re.is_match text="abc123" pattern="\\d+"*
> print text=`ok`

*m = > re.find text="ab12cd34" pattern="\\d+"*
> print text=`m`

*all = > re.find_all text="a1b22c333" pattern="\\d+"*
*joined = > join value=`all` sep=","*
> print text=`joined`

*rep = > re.replace text="a1b2c3" pattern="\\d+" with="X" count=2*
> print text=`rep`

*parts = > re.split text="a-b-c" pattern="-"*
*pj = > join value=`parts` sep="|"*
> print text=`pj`
