---
markup: occlusion
title: S2 G02 BoundCall 路径绑定与动态标记
status: implemented
import:
  pricing: pricing.mq.md
---

## helper

动态目标演示用无参函数。

> print text=hi

# main

显式 import 路径必须进 BoundCall；`call_fn` 必须进 DynamicCall。

**小计 = [两件](pricing.商品小计(数量=2, 单价=30))**
> call_fn name=helper
> print text=`小计`
