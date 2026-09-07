---
title: table list mid smoke
import table:lib/table.mq.md
---

# main

*xs = > split value="3,1,2,1" sep=","*
*xs = > table.sort list=`xs`*
*s = > join value=`xs` sep=","*
> print text=`s`

*u = > table.unique list=`xs`*
*uj = > join value=`u` sep=","*
> print text=`uj`

*ch = > table.chunk list=`xs` size=2*
*n = > len `ch`*
> print text=`n`

*a = > split value="a,b" sep=","*
*b = > split value="1,2,3" sep=","*
*z = > table.zip a=`a` b=`b`*
*zn = > len `z`*
> print text=`zn`

*nested = > table.append list=None item=`a`*
*nested = > table.append list=`nested` item=`b`*
*flat = > table.flatten list=`nested`*
*fj = > join value=`flat` sep=","*
> print text=`fj`
