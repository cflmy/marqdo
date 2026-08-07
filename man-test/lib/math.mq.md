---
title: 手工构造的测试样例
description: AI禁止更改
> lib/math.mq.md
---

# main

`f` = 
$$
x^2 + 2
$$

`f` =
$$
x^2 - 2
$$

*`ty` = > type `f` *

> print text=`ty`

*`df` = > math.diff formula=`f` var=x *

> print text=`df`

*`roots` = > math.solve formula=`f` var=x *

> print text=`roots`

*`y` = > math.eval formula=`f` var=x value=3 *

> print text=`y`