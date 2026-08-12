---
title: quantum zh API smoke
description: Chinese 量子.电路 泡利Z sandwich.
> ext/quantum/量子.mq.md
> lib/math.mq.md
> lib/json.mq.md
---

# main

*`电路` = > 量子.电路 比特数=1 *
*`电路` = > `电路`.哈达玛 比特=0 *
*`电路` = > `电路`.泡利Z 比特=0 *
*`电路` = > `电路`.哈达玛 比特=0 *
*`概率` = > `电路`.概率 *
*`p1` = > json.get value=`概率` key="1" *
*`lo` = > math.div a=9 b=10 *

1. `p1` > `lo`
  > print text=zh-ok
2. *
  > print text=zh-fail
