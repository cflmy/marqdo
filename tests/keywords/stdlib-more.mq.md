---
title: stdlib text and table
description: trim split join at type + lib imports
---

# main

*`raw` =   hi  *

*`t` = > trim `raw`*

> print text=`t`

*`parts` = > split value=a,b,c sep=,*

*`n` = > len `parts`*

> print text=`n`

*`mid` = > at value=`parts` index=1*

> print text=`mid`

*`joined` = > join value=`parts` sep=-*

> print text=`joined`

*`ty` = > type `parts`*

> print text=`ty`
