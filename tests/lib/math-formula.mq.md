---
title: math formula pipeline
> lib/math.mq.md
---

# main

`f` =
$$
x^2 - 2
$$

*`ty` = > type `f` *

> print text=`ty`

*`df` = > diff formula=`f` var=x *

> print text=`df`

*`roots` = > solve formula=`f` var=x *

> print text=`roots`

*`y` = > eval formula=`f` var=x value=3 *

> print text=`y`
