---
title: quantum noise zh smoke
description: 噪声 bitflip p=1 后 I 使全部射中 |1⟩.
导入 量子:ext/quantum/量子.mq.md
---

# main

*qc = > 量子.电路 比特数=1*
*qc = > `qc`.单位 比特=0*
*qc = > `qc`.噪声 种类="bitflip" 概率=1*
*r = > `qc`.运行 次数=32 种子=3*
*c1 = r[^counts][^1]*

1. `c1` == 32
  > print text=noise-ok
2. *
  > print text=noise-fail
