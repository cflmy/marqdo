---
title: quantum amplitude damping zh smoke
description: 泡利X 后 振幅阻尼 p=1 全部落回 |0⟩.
导入 量子:ext/quantum/量子.mq.md
---

# main

*`qc` = > 量子.电路 比特数=1 *
*`qc` = > `qc`.泡利X 比特=0 *
*`qc` = > `qc`.噪声 种类=振幅阻尼 概率=1 *
*`r` = > `qc`.运行 次数=32 种子=7 *
*`c0` = `r`[^counts][^0] *

1. `c0` == 32
  > print text=amp-ok
2. *
  > print text=amp-fail
