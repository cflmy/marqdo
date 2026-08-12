---
title: quantum author API zh smoke
description: 屏障 / 测量 / 追加 / 态 / 绘图.
> ext/quantum/量子.mq.md
---

# main

*`qc` = > 量子.电路 比特数=2 *
*`qc` = > `qc`.哈达玛 比特=0 *
*`qc` = > `qc`.屏障 *
*`qc` = > `qc`.控非 控制=0 目标=1 *
*`qc` = > `qc`.测量 *

*`st` = > `qc`.态 *
*`dim` = `st`[^dim] *
1. `dim` == 4
  > print text=state-ok
2. *
  > print text=state-fail

*`img` = > `qc`.绘图 路径=quantum-author-zh-draw.svg *
*`kind` = `img`[^kind] *
1. `kind` == circuit
  > print text=draw-ok
2. *
  > print text=draw-fail

*`other` = > 量子.电路 比特数=2 *
*`other` = > `other`.泡利X 比特=1 *
*`qc2` = > `qc`.追加 操作=`other` *
*`ops` = `qc2`[^ops] *
*`n` = > len `ops` *
1. `n` == 5
  > print text=append-ok
2. *
  > print text=append-fail
