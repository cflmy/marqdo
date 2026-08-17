---
title: quantum custom gate zh from formula
description: $$ 矩阵 $$ → 门 矩阵= → 施加.
导入 量子:ext/quantum/量子.mq.md
import math:lib/math.mq.md
---

# main

`H矩阵` =
$$
\frac{1}{\sqrt{2}}\begin{pmatrix}1&1\\1&-1\end{pmatrix}
$$

*U = > 量子.门 矩阵=`H矩阵` 名=U*
*qc = > 量子.电路 比特数=1*
*qc = > `qc`.施加 门=`U` 比特=0*
*p = > `qc`.概率*
*p0 = p[^0]*
*lo = > math.div a=4 b=10*
*hi = > math.div a=6 b=10*

1. `p0` > `lo`
  1. `p0` < `hi`
    > print text=custom-ok
  2. *
    > print text=custom-fail
2. *
  > print text=custom-fail
