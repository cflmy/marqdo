---
title: math formula chain
> lib/math.mq.md
---

# main

`f` =
$$
x^3
$$

*`df` = > math.diff formula=`f` var=x *

*`ty` = > type `df` *

> print text=`ty`

*`df2` = > math.diff formula=`df` var=x *

> print text=`df2`

*`y` = > math.eval formula=`df` var=x value=2 *

> print text=`y`
