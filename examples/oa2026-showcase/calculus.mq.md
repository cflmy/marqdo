---
title: 一份可执行的求导讲义
description: 公式、求导、求根、求值与绘图，在同一条叙述里完成。
import:
  math: lib/math.mq.md
---

# main

我们从一条曲线开始。下面这个公式既是排版出来的数学对象，
也是运行时可以直接求导、求根与绘图的`f`。

`f` =
$$
x^3 - 3*x
$$

对它求导，得到`df`；再让`df`在 x = 2 处取值，得到切线斜率。

**`df` = [math.diff] formula=`f` var="x"**
**`slope` = [math.eval] formula=`df` var="x" value=2**
> print text=`df`
> print text=`slope`

曲线与 x 轴的交点同样可以被机器找出来。这里不需要另写一段脚本。

**`roots` = [math.solve] formula=`f` var="x" min=-3 max=3**
> print text=`roots`

最后把公式画成图。图不是截图，而是本文档运行产生的派生物。

**`_svg` = [math.plot] formula=`f` var="x" min=-3 max=3 path="calculus-plot.svg"**
> print text=plot-ok
